pub mod garbage_collector;
pub mod stack;

use crate::core::vm::error::VmError;
use crate::core::vm::memory::stack::StackFrame;
use crate::core::vm::number::Value;
use std::collections::HashMap;

/// A free block record
#[derive(Debug, Clone, Copy)]
struct FreeBlock {
    base: usize,
    size: usize,
}

/// Object metadata for registry
#[derive(Debug, Clone, Copy)]
pub struct ObjectMetadata {
    base: usize,
    size: usize,
    is_static: bool,
    marked: bool,
}

pub struct Memory {
    // Stack
    pub operand_stack: Vec<Value>,
    pub call_stack: Vec<StackFrame>,

    // Heap
    heap: Vec<u8>,
    pub next_free: usize,

    // Object registry. Maps base address to metadata
    object_registry: HashMap<usize, ObjectMetadata>,

    static_roots: Vec<usize>,

    // Free blocks available for reuse
    free_list: Vec<FreeBlock>,

    // Cached headers
    header_cache: HashMap<usize, (u32, bool)>, // (size, is_static)

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
            next_free: 0,
            object_registry: HashMap::new(),
            static_roots: Vec::new(),
            free_list: Vec::new(),
            header_cache: HashMap::new(),
            allocated_bytes_since_last_gc: 0,
            gc_threshold: heap_size / 10, // trigger GC after 10% allocated
            debug: false,
        }
    }

    fn record_event(&mut self, msg: String) {
        if self.debug {
            println!("  {}", msg);
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

    //
    // OBJECTS
    //

    /// Validate object header
    pub fn is_valid_object_header(&self, base: usize) -> bool {
        if base >= self.heap.len() || base + 16 > self.heap.len() {
            return false;
        }
        // Check if this base is in the registry
        self.object_registry.contains_key(&base)
    }

    /// Get object size from cache or read from heap
    pub fn get_object_size(&self, base: usize) -> Result<usize, VmError> {
        // Check cache first
        if let Some(&(size, _)) = self.header_cache.get(&base) {
            return Ok(size as usize);
        }

        // Fall back to reading from heap
        self.ensure_in_bounds(base, 4)?;
        let size = u32::from_le_bytes(
            self.heap[base..base + 4]
                .try_into()
                .map_err(|_| VmError::InvalidReference(base))?,
        ) as usize;

        if size == 0 || size > self.heap.len() {
            return Err(VmError::InvalidReference(base));
        }

        Ok(size)
    }

    /// Find allocation base for a pointer-like addr
    pub fn find_object_base(&self, addr: usize) -> Result<usize, VmError> {
        // First check if addr itself is a valid base
        if let Some(metadata) = self.object_registry.get(&addr) {
            return Ok(metadata.base);
        }

        // Check if addr falls within any registered object
        for (&base, metadata) in &self.object_registry {
            if addr >= base && addr < base + metadata.size {
                return Ok(base);
            }
        }

        Err(VmError::InvalidReference(addr))
    }

    //
    // ALLOC
    //

    /// Allocate space on the heap (first try free list, then bump).
    pub fn allocate(&mut self, size: usize, is_static: bool) -> Result<usize, VmError> {
        if size == 0 {
            return Err(VmError::ZeroAllocation);
        }

        self.check_and_collect_garbage(size)?;

        // Try free list first
        if let Some(addr) = self.allocate_from_free_list(size) {
            self.register_allocation(addr, size, is_static);
            return Ok(addr);
        }

        // Fall back to bump allocation
        let addr = self.allocate_from_bump(size)?;
        self.register_allocation(addr, size, is_static);
        Ok(addr)
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

    /// Register a newly allocated object in the metadata registry
    fn register_allocation(&mut self, addr: usize, size: usize, is_static: bool) {
        self.object_registry.insert(
            addr,
            ObjectMetadata {
                base: addr,
                size,
                is_static,
                marked: false,
            },
        );

        if is_static {
            self.static_roots.push(addr);
        }
    }

    /// Determine if GC should run and execute if necessary
    fn check_and_collect_garbage(&mut self, size: usize) -> Result<(), VmError> {
        let will_fit_free = size <= self.heap.len().saturating_sub(self.next_free);
        self.allocated_bytes_since_last_gc =
            self.allocated_bytes_since_last_gc.saturating_add(size);

        if !will_fit_free || self.allocated_bytes_since_last_gc >= self.gc_threshold {
            self.collect_garbage()?;
            self.allocated_bytes_since_last_gc = 0;
        }

        Ok(())
    }

    /// Allocate from the bump pointer if free list has no suitable block
    fn allocate_from_bump(&mut self, size: usize) -> Result<usize, VmError> {
        let available = self.heap.len().saturating_sub(self.next_free);
        if size > available {
            return Err(VmError::HeapExhausted {
                requested: size,
                available,
            });
        }

        let addr = self.next_free;
        self.next_free = self.next_free.saturating_add(size);
        Ok(addr)
    }

    //
    // FREE
    //

    /// Free an allocation
    fn free(&mut self, addr: usize, size: usize) -> Result<(), VmError> {
        self.unregister_allocation(addr);
        self.zero_memory(addr, size);

        // Handle tail coalescing (freed block is at end of bump pointer)
        if addr + size == self.next_free {
            self.coalesce_tail(addr);
        } else {
            // Add to free list with adjacent block coalescing
            self.add_to_free_list(addr, size);
        }

        Ok(())
    }

    /// Unregister an allocation and clear its metadata
    fn unregister_allocation(&mut self, addr: usize) {
        self.object_registry.remove(&addr);
        self.static_roots.retain(|&root| root != addr);
        self.header_cache.remove(&addr);
    }

    /// Zero out memory for a freed block
    fn zero_memory(&mut self, addr: usize, size: usize) {
        let end = addr.saturating_add(size).min(self.heap.len());
        self.heap[addr..end].fill(0);
    }

    /// Handle coalescing with free list when freeing at the tail
    fn coalesce_tail(&mut self, addr: usize) {
        self.next_free = addr;
        loop {
            let found_index = self
                .free_list
                .iter()
                .position(|fb| fb.base + fb.size == self.next_free);

            match found_index {
                Some(i) => {
                    let fb = self.free_list.swap_remove(i);
                    self.next_free = fb.base;
                }
                None => break,
            }
        }
    }

    /// Add freed block to free list
    fn add_to_free_list(&mut self, addr: usize, size: usize) {
        self.free_list.push(FreeBlock { base: addr, size });
    }

    /// Load a variable from the heap (RTTI + data)
    pub fn load_from_heap(&self, addr: usize) -> Result<(&[u8], &[u8]), VmError> {
        self.ensure_in_bounds(addr, 16)?;

        // Read total size from header
        let total_size = self.get_object_size(addr)?;
        self.ensure_in_bounds(addr, total_size)?;

        // Load RTTI (bytes 4-11)
        let rtti = &self.heap[addr + 4..addr + 12];

        // Load data len (bytes 12-15)
        let data_len =
            u32::from_le_bytes(self.heap[addr + 12..addr + 16].try_into().unwrap()) as usize;

        // Load data (bytes 16+)
        let data_offset = addr + 16;
        let data = &self.heap[data_offset..data_offset + data_len];

        Ok((rtti, data))
    }

    /// Load a variable RTTI from the heap
    pub fn load_type_from_heap(&self, addr: usize) -> Result<&[u8], VmError> {
        self.ensure_in_bounds(addr, 12)?;
        Ok(&self.heap[addr + 4..addr + 12])
    }

    /// Save a variable to the heap (RTTI + data). Returns address.
    pub fn save_to_heap(
        &mut self,
        rtti: &[u8],
        data: &[u8],
        is_static: bool,
    ) -> Result<usize, VmError> {
        if rtti.len() != 8 {
            return Err(VmError::RTTITooLarge(rtti.len()));
        }

        // Allocate heap object
        let total_size = 16 + data.len();
        let addr = self.allocate(total_size, is_static)?;

        // Write header
        self.heap[addr..addr + 4].copy_from_slice(&(total_size as u32).to_le_bytes()); // Size (4 bytes)
        self.heap[addr + 4..addr + 12].copy_from_slice(rtti); // RTTI (8 bytes)
        self.heap[addr + 12..addr + 16].copy_from_slice(&(data.len() as u32).to_le_bytes()); // Data length (4 bytes)

        // Cache header to avoid redundant reads
        self.header_cache
            .insert(addr, (total_size as u32, is_static));

        // Write data (bytes 16+)
        if !data.is_empty() {
            self.heap[addr + 16..addr + 16 + data.len()].copy_from_slice(data);
        }

        Ok(addr)
    }

    /// Load a pre-serialized object from executable data into heap.
    pub fn load_from_executable(&mut self, data: &[u8], is_static: bool) -> Result<usize, VmError> {
        // Allocate and copy data to heap
        let addr = self.allocate(data.len(), is_static)?;
        self.heap[addr..addr + data.len()].copy_from_slice(data);
        self.header_cache
            .insert(addr, (data.len() as u32, is_static));

        Ok(addr)
    }

    /// Get all static roots for GC scanning
    pub fn get_static_roots(&self) -> &[usize] {
        &self.static_roots
    }
}
