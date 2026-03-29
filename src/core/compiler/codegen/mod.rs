pub mod generator;

use crate::core::compiler::linker::Linker;
use crate::core::compiler::parser::tree::{Node, Type};
use crate::core::shared::bytecode::Opcode::*;
use crate::core::shared::bytecode::{Instruction, Opcode, Operand};
use crate::core::shared::executable::Executable;
use lasso::{Rodeo, Spur};
use std::collections::HashMap;

pub struct Scope {
    variables: HashMap<Spur, (Operand, Type)>,
    parent: Option<Box<Scope>>,
    next_local_id: Operand,
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
            next_local_id: 0,
        }
    }

    pub fn with_parent(parent: Scope) -> Self {
        Self {
            variables: HashMap::new(),
            parent: Some(Box::new(parent)),
            next_local_id: 0,
        }
    }

    pub fn declare(&mut self, name: Spur, local_id: Operand, var_type: Type) -> Result<(), String> {
        if self.variables.contains_key(&name) {
            return Err("Variable already declared in this scope".to_string());
        }
        self.variables.insert(name, (local_id, var_type));
        Ok(())
    }

    pub fn lookup(&self, name: Spur) -> Option<(Operand, Type)> {
        self.variables
            .get(&name)
            .cloned()
            .or_else(|| self.parent.as_ref().and_then(|p| p.lookup(name)))
    }

    pub fn allocate_local_id(&mut self) -> Operand {
        let id = self.next_local_id;
        self.next_local_id += 1;
        id
    }

    pub fn get_type(&self, id: Spur) -> Option<Type> {
        self.lookup(id).map(|(_, var_type)| var_type)
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
            next_label_id: 0,      // Label namespace: 0 - 99,999
            next_data_id: 100_000, // Data namespace: 100,000 - i32 limit
            scope: Scope::new(),
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
        let executable = linker.link(entry_point.0)?;

        Ok(executable)
    }

    /// Scan all nodes and collect function declarations
    fn collect_functions(&mut self, nodes: &[Node]) -> Result<(), String> {
        for node in nodes {
            if let Node::FunctionDecl {
                name, return_type, ..
            } = node
            {
                let label = self.allocate_label_id();
                let return_type = return_type.clone().unwrap_or(Type::Void);

                self.functions.insert(*name, FunctionMetadata { label });
                // Declare the function in the global scope
                self.scope.declare(*name, label, return_type)?;
            }
        }
        Ok(())
    }

    /// Allocate data and request relocation for the next instruction.
    /// Layout: `[rtti_len (1 byte) | rtti_bytes | data_len (4 bytes) | data_bytes]`
    fn insert_data<T: AsRef<[u8]>>(&mut self, value: T, rtti: Type) -> Result<(), String> {
        let bytes = value.as_ref();
        let rtti_bytes = rtti.to_bytes();

        // Allocate a data ID
        let data_id = self.allocate_data_id();

        // Emit instruction that will reference this data
        self.emit(PUSH_HEAP_REF, data_id);

        // Record the instruction offset
        let instruction_offset = self.instructions.len() - 1;

        // Build the complete data block with header
        let mut data_block = Vec::new();

        // Construct header
        let rtti_len = rtti_bytes.len() as u8;
        let data_len = bytes.len() as u32;
        data_block.push(rtti_len);
        data_block.extend_from_slice(&rtti_bytes);
        data_block.extend_from_slice(&data_len.to_le_bytes());
        data_block.extend_from_slice(bytes);

        // Push data
        self.pending_data.push((data_id, data_block));

        // Request relocation from the linker
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

    fn allocate_label_id(&mut self) -> Operand {
        let id = self.next_label_id;
        self.next_label_id += 1;
        id
    }

    fn emit(&mut self, opcode: Opcode, operand: Operand) {
        self.instructions.push(Instruction::new(opcode, operand));
    }
}
