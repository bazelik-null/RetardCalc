use crate::core::shared::types::Type;
use crate::core::vm::error::VmError;
use crate::core::vm::memory::Memory;
use crate::core::vm::number::Value;
use std::collections::HashSet;
use std::time::Instant;

const MAX_TRACE_ITERATIONS: usize = 1_000_000;

impl Memory {
    /// Check if object is marked for collection
    pub fn is_marked(&self, addr: usize) -> Result<bool, VmError> {
        self.object_registry
            .get(&addr)
            .map(|m| m.marked)
            .ok_or(VmError::InvalidReference(addr))
    }

    /// Set mark on object
    fn set_marked(&mut self, addr: usize, marked: bool) -> Result<(), VmError> {
        if let Some(metadata) = self.object_registry.get_mut(&addr) {
            metadata.marked = marked;
            Ok(())
        } else {
            Err(VmError::InvalidReference(addr))
        }
    }

    /// Check if object is static
    pub fn is_static(&self, addr: usize) -> Result<bool, VmError> {
        self.object_registry
            .get(&addr)
            .map(|m| m.is_static)
            .ok_or(VmError::InvalidReference(addr))
    }

    /// Scan object for references to other objects
    fn scan_object_references(&self, base: usize, work: &mut Vec<usize>) -> Result<(), VmError> {
        let (rtti, data) = self.load_from_heap(base)?;

        // Parse type from RTTI to get pointer offsets
        let (type_info, _) = Type::from_bytes(rtti).map_err(VmError::InvalidRTTI)?;
        let offsets = type_info.pointer_offsets();

        // Check each pointer offset in the object's data
        for offset in offsets {
            if offset + 8 <= data.len() {
                let mut bytes = [0u8; 8];
                bytes.copy_from_slice(&data[offset..offset + 8]);
                let candidate = usize::from_le_bytes(bytes);

                // Try to find the base of this reference
                if let Ok(candidate_base) = self.find_object_base(candidate) {
                    work.push(candidate_base);
                }
            }
        }

        Ok(())
    }

    pub fn collect_garbage(&mut self) -> Result<(), VmError> {
        let gc_start = Instant::now();
        self.log_gc_start();

        // Phase 1: Mark reachable objects
        let marked_count = self.mark_phase()?;

        // Phase 2: Sweep unreachable objects
        let swept_count = self.sweep_phase()?;

        // Optimize free list
        self.consolidate_free_list();

        self.log_gc_completion(gc_start, marked_count, swept_count);
        Ok(())
    }

    /// Phase 1: Mark all reachable objects
    fn mark_phase(&mut self) -> Result<usize, VmError> {
        // Reset all mark bits
        for metadata in self.object_registry.values_mut() {
            metadata.marked = false;
        }

        let mut work_queue: Vec<usize> = Vec::new();
        let mut visited: HashSet<usize> = HashSet::new();

        // Collect all root references
        self.collect_roots(&mut work_queue);

        // Trace reachable objects
        self.trace_reachable_objects(&mut work_queue, &mut visited)?;

        Ok(visited.len())
    }

    /// Collect all root references from stack and static data
    fn collect_roots(&self, work_queue: &mut Vec<usize>) {
        // Roots from operand stack
        for value in &self.operand_stack {
            if let Value::Ref(ref_addr) = *value
                && let Ok(base) = self.find_object_base(ref_addr)
            {
                work_queue.push(base);
            }
        }

        // Roots from call stack locals
        for frame in &self.call_stack {
            for value in &frame.locals {
                if let Value::Ref(ref_addr) = *value
                    && let Ok(base) = self.find_object_base(ref_addr)
                {
                    work_queue.push(base);
                }
            }
        }

        // Roots from static objects
        for &static_root in self.get_static_roots() {
            work_queue.push(static_root);
        }
    }

    /// Trace all reachable objects from work queue
    fn trace_reachable_objects(
        &mut self,
        work_queue: &mut Vec<usize>,
        visited: &mut HashSet<usize>,
    ) -> Result<(), VmError> {
        let mut iterations = 0;

        while let Some(object_base) = work_queue.pop() {
            iterations += 1;
            if iterations > MAX_TRACE_ITERATIONS {
                return Err(VmError::GcError(
                    "Mark phase exceeded maximum iterations (possible infinite loop)".to_string(),
                ));
            }

            // Mark as visited and process only if not already seen
            if visited.insert(object_base) {
                self.set_marked(object_base, true)?;
                self.scan_object_references(object_base, work_queue)?;
            }
        }

        Ok(())
    }

    /// Phase 2: Sweep unmarked objects
    fn sweep_phase(&mut self) -> Result<usize, VmError> {
        let mut objects_to_free: Vec<(usize, usize)> = Vec::new();

        // Identify unmarked, non-static objects
        for (&addr, metadata) in self.object_registry.iter() {
            if !metadata.marked && !metadata.is_static {
                objects_to_free.push((addr, metadata.size));
            }
        }

        let swept_count = objects_to_free.len();

        // Free unmarked objects
        for (addr, size) in objects_to_free {
            if let Err(e) = self.free(addr, size)
                && self.debug
            {
                self.record_event(format!(
                    "[GC]: Warning: failed to free object at 0x{:06x}: {:?}",
                    addr, e
                ));
            }
        }

        Ok(swept_count)
    }

    fn consolidate_free_list(&mut self) {
        if self.free_list.len() <= 1 {
            return;
        }

        // Sort by base then merge adjacent blocks in one pass
        self.free_list.sort_by_key(|fb| fb.base);

        let mut write = 0;
        for read in 0..self.free_list.len() {
            if write == 0 {
                self.free_list[write] = self.free_list[read];
                write += 1;
                continue;
            }

            let last = self.free_list[write - 1];
            let cur = self.free_list[read];

            if last.base + last.size == cur.base {
                self.free_list[write - 1].size = last.size + cur.size;
            } else {
                if write != read {
                    self.free_list[write] = cur;
                }
                write += 1;
            }
        }
        self.free_list.truncate(write);
    }

    fn log_gc_start(&mut self) {
        self.record_event(format!(
            "[GC]: Starting garbage collection: Heap used=0x{:06x}",
            self.next_free
        ));
    }

    fn log_gc_completion(&mut self, gc_start: Instant, marked_count: usize, swept_count: usize) {
        let duration_ms = gc_start.elapsed().as_millis();
        self.record_event(format!(
            "[GC]: Marked {} objects | Swept {} objects",
            marked_count, swept_count
        ));
        self.record_event(format!(
            "[GC]: Completed: Heap used=0x{:06x}; Duration={}ms;",
            self.next_free, duration_ms
        ));
    }
}
