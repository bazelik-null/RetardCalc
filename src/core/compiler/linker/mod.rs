use crate::core::shared::bytecode::{Instruction, Operand};
use crate::core::shared::executable::Executable;
use std::collections::HashMap;

#[derive(Debug, Clone, Copy)]
pub enum Symbol {
    Label { offset: usize },
    DataSection { offset: usize, size: usize },
}

/// Tracks symbols during the linking phase.
#[derive(Debug, Clone)]
pub struct SymbolTable {
    symbols: HashMap<Operand, Symbol>,
}

impl SymbolTable {
    pub fn new() -> Self {
        SymbolTable {
            symbols: HashMap::new(),
        }
    }

    pub fn define_label(
        &mut self,
        label: Operand,
        instruction_offset: usize,
    ) -> Result<(), String> {
        if self.symbols.contains_key(&label) {
            return Err(format!("Symbol already defined: {:?}", label));
        }
        self.symbols.insert(
            label,
            Symbol::Label {
                offset: instruction_offset,
            },
        );
        Ok(())
    }

    pub fn define_data_section(
        &mut self,
        id: Operand,
        offset: usize,
        size: usize,
    ) -> Result<(), String> {
        if self.symbols.contains_key(&id) {
            return Err(format!("Symbol already defined: {:?}", id));
        }
        self.symbols
            .insert(id, Symbol::DataSection { offset, size });
        Ok(())
    }

    pub fn resolve(&self, symbol: Operand) -> Result<Symbol, String> {
        self.symbols
            .get(&symbol)
            .copied()
            .ok_or_else(|| format!("Unresolved symbol: {:?}", symbol))
    }

    pub fn has(&self, symbol: Operand) -> bool {
        self.symbols.contains_key(&symbol)
    }
}

impl Default for SymbolTable {
    fn default() -> Self {
        Self::new()
    }
}

/// A relocation request before linking.
#[derive(Debug, Clone, Copy)]
pub struct UnresolvedRelocation {
    pub instruction_offset: usize,
    pub symbol: Operand,
    pub addend: isize,
}

impl UnresolvedRelocation {
    pub fn new(instruction_offset: usize, symbol: Operand, addend: isize) -> Self {
        UnresolvedRelocation {
            instruction_offset,
            symbol,
            addend,
        }
    }
}

/// Tracks relocations before linking.
#[derive(Debug, Clone)]
pub struct RelocationQueue {
    relocations: Vec<UnresolvedRelocation>,
}

impl RelocationQueue {
    pub fn new() -> Self {
        RelocationQueue {
            relocations: Vec::new(),
        }
    }

    pub fn add(&mut self, relocation: UnresolvedRelocation) {
        self.relocations.push(relocation);
    }

    pub fn entries(&self) -> &[UnresolvedRelocation] {
        &self.relocations
    }

    pub fn count(&self) -> usize {
        self.relocations.len()
    }

    pub fn clear(&mut self) {
        self.relocations.clear();
    }
}

impl Default for RelocationQueue {
    fn default() -> Self {
        Self::new()
    }
}

/// Resolves symbols, computes offsets, and applies relocations.
#[derive(Debug, Clone)]
pub struct Linker {
    symbol_table: SymbolTable,
    relocation_queue: RelocationQueue,
    instructions: Vec<Instruction>,
    data: Vec<u8>,
}

impl Linker {
    pub fn new(instructions: Vec<Instruction>) -> Self {
        Linker {
            symbol_table: SymbolTable::new(),
            relocation_queue: RelocationQueue::new(),
            instructions,
            data: Vec::new(),
        }
    }

    pub fn define_label(
        &mut self,
        label: Operand,
        instruction_offset: usize,
    ) -> Result<(), String> {
        if instruction_offset > self.instructions.len() {
            return Err(format!(
                "Label points beyond instruction bounds: {}",
                instruction_offset
            ));
        }
        self.symbol_table.define_label(label, instruction_offset)
    }

    pub fn allocate_data(&mut self, id: Operand, bytes: &[u8]) -> Result<(), String> {
        let offset = self.data.len();
        let size = bytes.len();
        self.symbol_table.define_data_section(id, offset, size)?;
        self.data.extend_from_slice(bytes);
        Ok(())
    }

    pub fn request_relocation(
        &mut self,
        instruction_offset: usize,
        symbol: Operand,
        addend: isize,
    ) -> Result<(), String> {
        if instruction_offset >= self.instructions.len() {
            return Err(format!(
                "Instruction offset out of bounds: {}",
                instruction_offset
            ));
        }
        let relocation = UnresolvedRelocation::new(instruction_offset, symbol, addend);
        self.relocation_queue.add(relocation);
        Ok(())
    }

    /// Resolve all symbols and apply relocations.
    pub fn link(mut self, entry_point: Operand) -> Result<Executable, String> {
        // Apply all relocations
        for relocation in self.relocation_queue.entries().to_vec() {
            self.apply_relocation(relocation)?;
        }

        // Get entry point address
        let main_address = self.get_symbol_address(entry_point)?;

        Ok(Executable::new(self.instructions, main_address, self.data))
    }

    fn apply_relocation(&mut self, relocation: UnresolvedRelocation) -> Result<(), String> {
        // Validate instruction offset is within bounds
        if relocation.instruction_offset >= self.instructions.len() {
            return Err(format!(
                "Relocation instruction offset out of bounds: {}",
                relocation.instruction_offset
            ));
        }

        // Resolve the symbol
        let base = self.get_symbol_address(relocation.symbol)?;

        let addr = if relocation.addend >= 0 {
            base.checked_add(relocation.addend as usize)
                .ok_or_else(|| "Relocation addend overflow".to_string())?
        } else {
            base.checked_sub(-relocation.addend as usize)
                .ok_or_else(|| "Relocation addend underflow".to_string())?
        };

        // Patch the instruction operand
        self.instructions[relocation.instruction_offset].operand = addr as Operand;

        Ok(())
    }
    pub fn get_symbol_address(&self, symbol: Operand) -> Result<usize, String> {
        match self.symbol_table.resolve(symbol)? {
            Symbol::Label { offset } => Ok(offset),
            Symbol::DataSection { offset, .. } => Ok(offset),
        }
    }

    pub fn symbol_table(&self) -> &SymbolTable {
        &self.symbol_table
    }
}
