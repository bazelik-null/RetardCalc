use crate::core::compiler::parser::tree::Type;
use crate::core::vm::error::VmError;
use colored::Colorize;
use std::collections::{BTreeMap, HashSet};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Value {
    Int(i32),
    Ref(usize),
}

impl Value {
    pub fn as_int(&self) -> Result<i32, VmError> {
        match self {
            Value::Int(i) => Ok(*i),
            Value::Ref(_) => Err(VmError::TypeMismatch("reference", "integer".to_string())),
        }
    }

    pub fn as_ref(&self) -> Result<usize, VmError> {
        match self {
            Value::Ref(addr) => Ok(*addr),
            Value::Int(_) => Err(VmError::TypeMismatch("integer", "reference".to_string())),
        }
    }

    pub fn is_ref(&self) -> bool {
        matches!(self, Value::Ref(_))
    }
}

#[derive(Debug, Clone)]
pub struct StackFrame {
    pub locals: Vec<Value>,
    pub return_address: usize,
}

impl StackFrame {
    pub fn new(return_address: usize) -> Self {
        Self {
            locals: Vec::new(),
            return_address,
        }
    }
}

#[derive(Debug)]
pub struct Allocation {
    pub size: usize,
    pub marked: bool,
    pub is_static: bool,
}

/// A free block record
#[derive(Debug, Clone, Copy)]
struct FreeBlock {
    base: usize,
    size: usize,
}

pub struct Memory {
    // Stack
    pub operand_stack: Vec<Value>,
    pub call_stack: Vec<StackFrame>,

    // Heap
    heap: Vec<u8>,
    pub allocations: BTreeMap<usize, Allocation>, // Allocations ordered by base address bc of BTreeMap
    next_free: usize,

    // Free blocks available for reuse
    free_list: Vec<FreeBlock>,

    // Garbage Collector
    allocated_bytes_since_last_gc: usize,
    gc_threshold: usize,

    // Debugging
    pub debug: bool,
}

impl Memory {
    pub fn new(heap_size: usize) -> Self {
        Self {
            operand_stack: Vec::with_capacity(1024),
            call_stack: Vec::with_capacity(256),
            heap: vec![0u8; heap_size],
            allocations: BTreeMap::new(),
            next_free: 0,
            free_list: Vec::new(),
            allocated_bytes_since_last_gc: 0,
            gc_threshold: heap_size / 10, // trigger GC after 10% allocated
            debug: false,
        }
    }

    fn record_event(&mut self, msg: String) {
        if self.debug {
            println!("{}", format!("  [DEBUG]: {}", msg).red());
        }
    }

    fn ensure_in_bounds(&self, addr: usize, len: usize) -> Result<(), VmError> {
        let end = addr.checked_add(len).ok_or(VmError::OutOfBounds {
            addr,
            len,
            heap_size: self.heap.len(),
        })?;
        if end > self.heap.len() {
            return Err(VmError::OutOfBounds {
                addr,
                len,
                heap_size: self.heap.len(),
            });
        }
        Ok(())
    }

    fn read_u8(&self, addr: usize) -> Result<u8, VmError> {
        self.ensure_in_bounds(addr, 1)?;
        Ok(self.heap[addr])
    }

    fn read_u32_le(&self, addr: usize) -> Result<u32, VmError> {
        let b = self.read_bytes(addr, 4)?;
        Ok(u32::from_le_bytes(b.try_into().unwrap()))
    }

    /// Try to find a free block that fits. If found, return base and possibly shrink block.
    fn allocate_from_free_list(&mut self, size: usize) -> Option<usize> {
        for i in 0..self.free_list.len() {
            if self.free_list[i].size >= size {
                let base = self.free_list[i].base;
                if self.free_list[i].size == size {
                    self.free_list.swap_remove(i);
                } else {
                    self.free_list[i].base += size;
                    self.free_list[i].size -= size;
                }
                return Some(base);
            }
        }
        None
    }

    /// Allocate space on the heap (first try free_list, then bump).
    pub fn allocate(&mut self, size: usize, is_static: bool) -> Result<usize, VmError> {
        if size == 0 {
            return Err(VmError::ZeroAllocation);
        }

        // Try GC if we're low on space or threshold reached
        let will_fit_free = size <= self.heap.len().saturating_sub(self.next_free);
        self.allocated_bytes_since_last_gc =
            self.allocated_bytes_since_last_gc.saturating_add(size);
        if !will_fit_free || self.allocated_bytes_since_last_gc >= self.gc_threshold {
            self.collect_garbage()?;
            self.allocated_bytes_since_last_gc = 0;
        }

        // Try free list
        if let Some(addr) = self.allocate_from_free_list(size) {
            self.allocations.insert(
                addr,
                Allocation {
                    size,
                    marked: false,
                    is_static,
                },
            );
            self.record_event(format!(
                "ALLOC  base=0x{:06x} size={} static={}",
                addr, size, is_static
            ));
            return Ok(addr);
        }

        // Bump heap
        let available = self.heap.len().saturating_sub(self.next_free);
        if size > available {
            return Err(VmError::HeapExhausted {
                requested: size,
                available,
            });
        }
        let addr = self.next_free;
        self.next_free = self.next_free.saturating_add(size);
        self.allocations.insert(
            addr,
            Allocation {
                size,
                marked: false,
                is_static,
            },
        );
        self.record_event(format!(
            "ALLOC  base=0x{:06x} size={} static={}",
            addr, size, is_static
        ));

        Ok(addr)
    }

    /// Find allocation base for a pointer-like addr.
    /// Uses ordered BTreeMap to find the greatest allocation base <= addr and checks range.
    pub fn allocation_base_for(&self, addr: usize) -> Result<usize, VmError> {
        if let Some((&base, alloc)) = self.allocations.range(..=addr).next_back()
            && addr < base + alloc.size
        {
            return Ok(base);
        }
        Err(VmError::InvalidReference(addr))
    }

    /// Free an allocation (internal)
    fn free_allocation(&mut self, addr: usize) -> Result<(), VmError> {
        let alloc = self
            .allocations
            .remove(&addr)
            .ok_or(VmError::InvalidReference(addr))?;
        let size = alloc.size;

        // Zero memory
        let end = addr.saturating_add(size).min(self.heap.len());
        for b in &mut self.heap[addr..end] {
            *b = 0;
        }

        // If this allocation is at the heap tail, move next_free back and try to collapse contiguous freed suffix.
        if addr + size == self.next_free {
            self.next_free = addr;
            // See if we can shrink further by consuming any free_list blocks that directly precede next_free.
            loop {
                // Find any free block that ends exactly at next_free
                let mut found_index: Option<usize> = None;
                for (i, fb) in self.free_list.iter().enumerate() {
                    if fb.base + fb.size == self.next_free {
                        found_index = Some(i);
                        break;
                    }
                }
                if let Some(i) = found_index {
                    let fb = self.free_list.swap_remove(i);
                    self.next_free = fb.base;
                } else {
                    break;
                }
            }
        } else {
            // Insert into free list and try to coalesce adjacent free blocks
            let mut new_fb = FreeBlock { base: addr, size };
            // Try to merge with existing blocks if adjacent
            let mut i = 0;
            while i < self.free_list.len() {
                let fb = self.free_list[i];
                // fb is directly before new_fb
                if fb.base + fb.size == new_fb.base {
                    new_fb.base = fb.base;
                    new_fb.size += fb.size;
                    self.free_list.swap_remove(i);
                    continue;
                }
                // fb is directly after new_fb
                if new_fb.base + new_fb.size == fb.base {
                    new_fb.size += fb.size;
                    self.free_list.swap_remove(i);
                    continue;
                }
                i += 1;
            }
            self.free_list.push(new_fb);
        }

        self.record_event(format!("FREE   base=0x{:06x} size={}", addr, size));

        Ok(())
    }

    /// Read bytes from heap
    pub fn read_bytes(&self, addr: usize, len: usize) -> Result<&[u8], VmError> {
        self.ensure_in_bounds(addr, len)?;
        Ok(&self.heap[addr..addr + len])
    }

    /// Write bytes to heap
    pub fn write_bytes(&mut self, addr: usize, data: &[u8]) -> Result<(), VmError> {
        self.ensure_in_bounds(addr, data.len())?;
        self.heap[addr..addr + data.len()].copy_from_slice(data);
        Ok(())
    }

    /// Push a value
    pub fn push(&mut self, value: Value) -> Result<(), VmError> {
        self.operand_stack.push(value);
        Ok(())
    }

    /// Pop a value
    pub fn pop(&mut self) -> Result<Value, VmError> {
        let value = self.operand_stack.pop().ok_or(VmError::StackUnderflow)?;
        Ok(value)
    }

    /// Peek at the top of the operand stack
    pub fn peek(&self) -> Result<Value, VmError> {
        self.operand_stack
            .last()
            .copied()
            .ok_or(VmError::StackUnderflow)
    }

    /// Push a call frame
    pub fn push_frame(&mut self, return_address: usize) {
        self.call_stack.push(StackFrame::new(return_address));
    }

    /// Pop a call frame
    pub fn pop_frame(&mut self) -> Result<StackFrame, VmError> {
        let frame = self.call_stack.pop().ok_or(VmError::CallStackUnderflow)?;
        Ok(frame)
    }

    /// Get current frame (mutable)
    pub fn current_frame_mut(&mut self) -> Result<&mut StackFrame, VmError> {
        self.call_stack.last_mut().ok_or(VmError::NoActiveFrame)
    }

    /// Get current frame (immutable)
    pub fn current_frame(&self) -> Result<&StackFrame, VmError> {
        self.call_stack.last().ok_or(VmError::NoActiveFrame)
    }

    /// Set a local variable
    pub fn set_local(&mut self, index: usize, value: Value) -> Result<(), VmError> {
        let frame = self.current_frame_mut()?;
        if index >= frame.locals.len() {
            frame.locals.resize(index + 1, Value::Int(0));
        }
        frame.locals[index] = value;
        Ok(())
    }

    /// Get a local variable from the current frame
    pub fn get_local(&self, index: usize) -> Result<Value, VmError> {
        let frame = self.current_frame()?;
        frame
            .locals
            .get(index)
            .copied()
            .ok_or(VmError::LocalOutOfBounds(index))
    }

    /// Load a variable from the heap (RTTI + data)
    pub fn load_from_heap(&self, addr: usize) -> Result<(&[u8], &[u8]), VmError> {
        // Read RTTI length
        let rtti_len = self.read_u8(addr)? as usize;
        let rtti_start = addr + 1;
        // Read RTTI
        let rtti = self.read_bytes(rtti_start, rtti_len)?;
        // Read data length
        let data_len_start = rtti_start + rtti_len;
        let data_len = self.read_u32_le(data_len_start)? as usize;
        // Read data
        let data_start = data_len_start + 4;
        let data = self.read_bytes(data_start, data_len)?;

        Ok((rtti, data))
    }

    /// Save a variable to the heap (RTTI + data). Returns address.
    pub fn save_to_heap(
        &mut self,
        rtti: &[u8],
        data: &[u8],
        is_static: bool,
    ) -> Result<usize, VmError> {
        if rtti.len() > 255 {
            return Err(VmError::RTTITooLarge(rtti.len()));
        }
        // Allocate block
        let total_size = 1 + rtti.len() + 4 + data.len();
        let addr = self.allocate(total_size, is_static)?;
        // Write RTTI length
        self.heap[addr] = rtti.len() as u8;
        let rtti_start = addr + 1;
        // Write RTTI bytes
        self.heap[rtti_start..rtti_start + rtti.len()].copy_from_slice(rtti);
        // Write data length
        let data_len_start = rtti_start + rtti.len();
        let data_len_bytes = (data.len() as u32).to_le_bytes();
        self.heap[data_len_start..data_len_start + 4].copy_from_slice(&data_len_bytes);
        // Write data
        let data_start = data_len_start + 4;
        self.heap[data_start..data_start + data.len()].copy_from_slice(data);

        Ok(addr)
    }

    /// Scan an object's data for potential references.
    fn scan_object_references(&self, base: usize, work: &mut Vec<usize>) -> Result<(), VmError> {
        let (rtti, data) = self.load_from_heap(base)?;

        // Parse type from RTTI
        let (type_info, _) = Type::from_bytes(rtti).map_err(VmError::InvalidRTTI)?;

        // Get pointer offsets from type
        let offsets = type_info.pointer_offsets();

        for offset in offsets {
            if offset + 8 <= data.len() {
                let mut bytes = [0u8; 8];
                bytes.copy_from_slice(&data[offset..offset + 8]);
                let candidate = usize::from_le_bytes(bytes);

                if let Ok(candidate_base) = self.allocation_base_for(candidate) {
                    work.push(candidate_base);
                }
            }
        }

        Ok(())
    }

    /// Tracing garbage collector
    pub fn collect_garbage(&mut self) -> Result<(), VmError> {
        if self.debug {
            self.record_event(format!(
                "GC START: allocations={} next_free=0x{:06x}",
                self.allocations.len(),
                self.next_free
            ));
        }

        // Reset marks
        for alloc in self.allocations.values_mut() {
            alloc.marked = false;
        }

        let mut work: Vec<usize> = Vec::new();
        let mut visited: HashSet<usize> = HashSet::new();

        // Add roots: operand stack
        for v in &self.operand_stack {
            if let Value::Ref(addr) = *v
                && let Ok(base) = self.allocation_base_for(addr)
            {
                work.push(base);
            }
        }

        // Add roots: call stack locals
        for frame in &self.call_stack {
            for v in &frame.locals {
                if let Value::Ref(addr) = *v
                    && let Ok(base) = self.allocation_base_for(addr)
                {
                    work.push(base);
                }
            }
        }

        // Add roots: static allocations
        for (&base, alloc) in &self.allocations {
            if alloc.is_static {
                work.push(base);
            }
        }

        // Mark phase
        while let Some(base) = work.pop() {
            if visited.contains(&base) {
                continue;
            }
            visited.insert(base);

            if let Some(alloc_meta) = self.allocations.get_mut(&base) {
                alloc_meta.marked = true;
            } else {
                continue;
            }

            self.scan_object_references(base, &mut work)?;
        }

        // Sweep phase: free unmarked, non-static allocations
        let mut bases_to_free: Vec<usize> = Vec::new();
        for (&base, alloc) in &self.allocations {
            if !alloc.marked && !alloc.is_static {
                bases_to_free.push(base);
            }
        }

        // Free each
        for base in bases_to_free {
            self.free_allocation(base)?;
        }

        if self.debug {
            self.record_event(format!(
                "GC END: allocations={} next_free=0x{:06x}",
                self.allocations.len(),
                self.next_free
            ));
        }

        Ok(())
    }

    pub fn peek_stack(&self) -> &[Value] {
        &self.operand_stack
    }
}
