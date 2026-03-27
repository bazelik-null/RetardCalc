pub mod generator;

use crate::core::compiler::linker::Linker;
use crate::core::compiler::parser::tree::Node;
use crate::core::shared::bytecode::Opcode::*;
use crate::core::shared::bytecode::{Instruction, Opcode, Operand};
use crate::core::shared::executable::Executable;
use lasso::{Rodeo, Spur};
use std::collections::HashMap;

pub struct Scope {
    variables: HashMap<Spur, Operand>,
    parent: Option<Box<Scope>>,
}

impl Default for Scope {
    fn default() -> Self {
        Self::new()
    }
}

impl Scope {
    pub fn new() -> Self {
        Self {
            variables: HashMap::new(),
            parent: None,
        }
    }

    pub fn with_parent(parent: Scope) -> Self {
        Self {
            variables: HashMap::new(),
            parent: Some(Box::new(parent)),
        }
    }

    pub fn declare(&mut self, name: Spur, local_id: Operand) -> Result<(), String> {
        if self.variables.contains_key(&name) {
            return Err("Variable already declared in this scope".to_string());
        }
        self.variables.insert(name, local_id);
        Ok(())
    }

    pub fn lookup(&self, name: Spur) -> Option<Operand> {
        self.variables
            .get(&name)
            .copied()
            .or_else(|| self.parent.as_ref().and_then(|p| p.lookup(name)))
    }
}

/// Metadata about a function for the first pass
struct FunctionMetadata {
    label: Operand,
}

pub struct CodeGenerator<'a> {
    rodeo: &'a Rodeo,
    instructions: Vec<Instruction>,
    next_data_id: Operand,
    scope: Scope,
    next_local_id: Operand,
    next_label_id: Operand,

    // Track symbols and relocations during generation
    pending_data: Vec<(Operand, Vec<u8>)>,
    pending_labels: Vec<(Operand, usize)>,
    pending_relocations: Vec<(usize, Operand, isize)>,

    // Store function metadata for the second pass
    functions: HashMap<Spur, FunctionMetadata>,
}

impl<'a> CodeGenerator<'a> {
    pub fn new(rodeo: &'a Rodeo) -> Self {
        Self {
            rodeo,
            instructions: vec![],
            next_data_id: 0,
            scope: Scope::new(),
            next_local_id: 0,
            next_label_id: 0,
            pending_data: vec![],
            pending_labels: vec![],
            pending_relocations: vec![],
            functions: HashMap::new(),
        }
    }

    /// Compile bytecode from AST nodes and produce a linked executable.
    pub fn compile(mut self, nodes: &[Node]) -> Result<Executable, String> {
        // Collect all function declarations and assign labels (first pass)
        self.collect_functions(nodes)?;

        // Get main function label
        let entry_name = self
            .rodeo
            .get("main")
            .ok_or_else(|| "Main function not found.".to_string())?;
        let entry_point = self
            .scope
            .lookup(entry_name)
            .ok_or_else(|| "Main function not found.".to_string())?;

        // Generate code for each node (second pass)
        for n in nodes.iter() {
            self.generate_node(n)?;
        }

        // Push final HALT instruction
        self.emit(HALT, 0);

        // Create linker with all instructions
        let mut linker = Linker::new(self.instructions);

        // Define all labels
        for (label, offset) in self.pending_labels {
            linker.define_label(label, offset)?;
        }

        // Allocate all data
        for (data_id, bytes) in self.pending_data {
            linker.allocate_data(data_id, &bytes)?;
        }

        // Request all relocations
        for (instr_offset, symbol, addend) in self.pending_relocations {
            linker.request_relocation(instr_offset, symbol, addend)?;
        }

        // Link and produce executable
        let executable = linker.link(entry_point)?;

        Ok(executable)
    }

    /// Scan all nodes and collect function declarations
    fn collect_functions(&mut self, nodes: &[Node]) -> Result<(), String> {
        for node in nodes {
            if let Node::FunctionDecl { name, .. } = node {
                let label = self.allocate_label_id();
                self.functions.insert(*name, FunctionMetadata { label });
                // Declare the function in the global scope
                self.scope.declare(*name, label)?;
            }
        }
        Ok(())
    }

    /// Allocate data and request relocation for the next instruction.
    fn insert_data<T: AsRef<[u8]>>(&mut self, value: T) -> Result<(), String> {
        let bytes = value.as_ref();

        // Allocate a data ID
        let data_id = self.allocate_data_id();

        // Emit instruction that will reference this data
        self.emit(PUSH, data_id);

        // Record the instruction offset
        let instruction_offset = self.instructions.len() - 1;

        // Request relocation from the linker
        self.pending_data.push((data_id, bytes.to_vec()));
        self.pending_relocations
            .push((instruction_offset, data_id, 0));

        Ok(())
    }

    /// Define a label at the current instruction offset.
    fn define_label(&mut self, label: Operand) -> Result<(), String> {
        let instruction_offset = self.instructions.len();
        self.pending_labels.push((label, instruction_offset));
        Ok(())
    }

    /// Request a relocation for a branch/jump instruction.
    fn request_branch_relocation(
        &mut self,
        instruction_offset: usize,
        target_label: Operand,
    ) -> Result<(), String> {
        self.pending_relocations
            .push((instruction_offset, target_label, 0));
        Ok(())
    }

    fn allocate_data_id(&mut self) -> Operand {
        let id = self.next_data_id;
        self.next_data_id += 1;
        id
    }

    fn allocate_local_id(&mut self) -> Operand {
        let id = self.next_local_id;
        self.next_local_id += 1;
        id
    }

    fn allocate_label_id(&mut self) -> Operand {
        let id = self.next_label_id;
        self.next_label_id += 1;
        id
    }

    fn emit(&mut self, opcode: Opcode, operand: Operand) {
        self.instructions.push(Instruction::new(opcode, operand));
    }
}
