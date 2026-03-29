use crate::core::vm::error::VmError;
use crate::core::vm::memory::Value;
use crate::core::vm::VirtualMachine;
use colored::Colorize;
use std::collections::{HashMap, HashSet, VecDeque};

type ReachableAndReferrers = (HashSet<usize>, HashMap<usize, Vec<String>>);

struct HeapBlockInfo {
    base: usize,
    size: usize,
    is_static: bool,
    reachable: bool,
    type_name: String,
    referrers: Vec<String>,
}

impl VirtualMachine {
    pub fn run_debug(&mut self) -> Result<(), VmError> {
        self.debug = true;
        self.memory.debug = true;
        self.memory.push_frame(0);

        println!("\n {}", "VM EXECUTION".bold().cyan());
        println!(" {}", "═".repeat(75).cyan());

        while !self.halted && self.pc < self.instructions.len() {
            if self.debug {
                self.print_debug_line()?;
            }
            self.step()?;
        }

        self.memory.collect_garbage()?; // Collect garbage on exit

        println!(" {}", "═".repeat(75).cyan());
        self.print_memory_report()?;
        println!(" {}\n", "END OF EXECUTION".bold().cyan());
        Ok(())
    }

    fn print_debug_line(&self) -> Result<(), VmError> {
        if self.pc >= self.instructions.len() {
            println!(
                "  {}  {}",
                format!("0x{:06x}", self.pc).cyan(),
                "OUT OF BOUNDS".red().bold()
            );
            return Ok(());
        }

        let instr = self.instructions[self.pc];
        let stack_display = self.format_stack_display()?;

        println!(
            "  {}  {} {}",
            format!("0x{:06x}", self.pc).cyan(),
            instr,
            stack_display
        );

        Ok(())
    }

    fn format_stack_display(&self) -> Result<String, VmError> {
        let stack = self.memory.peek_stack();
        if stack.is_empty() {
            return Ok(format!("[{}]", "empty".bright_black()));
        }

        let items: Vec<String> = stack
            .iter()
            .map(|v| match v {
                Value::Int(n) => format!("Int<{}>", n).green().to_string(),
                Value::Ref(addr) => format!("Ref<0x{:x}>", addr).bright_cyan().to_string(),
            })
            .collect();

        Ok(format!("[{}]", items.join(", ")))
    }

    fn compute_reachable_and_referrers(&self) -> Result<ReachableAndReferrers, VmError> {
        let mut work: VecDeque<usize> = VecDeque::new();
        let mut marked: HashSet<usize> = HashSet::new();
        let mut referrers: HashMap<usize, Vec<String>> = HashMap::new();

        let mut push_base = |base: usize,
                             label: Option<String>,
                             work: &mut VecDeque<usize>,
                             marked: &mut HashSet<usize>| {
            if marked.insert(base) {
                work.push_back(base);
            }
            if let Some(lbl) = label {
                referrers.entry(base).or_default().push(lbl);
            }
        };

        // Helper to process a Value that might be a ref
        let mut process_val = |val: &Value, label: String| {
            if let Value::Ref(addr) = *val
                && let Ok(base) = self.memory.allocation_base_for(addr)
            {
                push_base(base, Some(label), &mut work, &mut marked);
            }
        };

        // Roots: operand stack
        for (i, v) in self.memory.operand_stack.iter().enumerate() {
            process_val(v, format!("operand_stack[{}]", i));
        }

        // Roots: call stack locals
        for (fi, frame) in self.memory.call_stack.iter().enumerate() {
            for (li, v) in frame.locals.iter().enumerate() {
                process_val(v, format!("frame[{}].local[{}]", fi, li));
            }
        }

        // Roots: static allocations
        for (&base, alloc) in &self.memory.allocations {
            if alloc.is_static {
                push_base(base, Some("static".to_string()), &mut work, &mut marked);
            }
        }

        // Marking: scan each object's bytes and treat words as potential pointers
        let bytes = 8;
        while let Some(base) = work.pop_front() {
            if let Ok((_rtti, data)) = self.memory.load_from_heap(base) {
                let mut i = 0;
                while i + bytes <= data.len() {
                    let mut buf = [0u8; 8];
                    buf.copy_from_slice(&data[i..i + bytes]);
                    let candidate = usize::from_le_bytes(buf);
                    if let Ok(candidate_base) = self.memory.allocation_base_for(candidate) {
                        referrers
                            .entry(candidate_base)
                            .or_default()
                            .push(format!("heap[0x{:06x}]+{}", base, i));
                        if marked.insert(candidate_base) {
                            work.push_back(candidate_base);
                        }
                    }
                    i += bytes;
                }
            }
        }

        Ok((marked, referrers))
    }

    pub fn print_heap_view(&self) -> Result<(), VmError> {
        println!("\n {}", "HEAP VIEW".bold().magenta());
        println!(" {}", "─".repeat(75).magenta());

        let blocks = self.get_heap_blocks_with_referrers()?;

        if blocks.is_empty() {
            println!("  {} (no allocations)", "empty".bright_black());
            return Ok(());
        }

        let total_allocated: usize = blocks.iter().map(|b| b.size).sum();

        for block in &blocks {
            let status = if block.is_static {
                "STATIC".bright_magenta().bold()
            } else if block.reachable {
                "LIVE".green()
            } else {
                "UNREACHABLE".red().bold()
            };

            println!(
                "  {} size: {:>6} bytes  type: {}  {}",
                format!("0x{:06x}", block.base).cyan(),
                block.size,
                block.type_name.bright_yellow(),
                status
            );

            if !block.referrers.is_empty() {
                let display: Vec<_> = block.referrers.iter().take(4).cloned().collect();
                println!("    refs from: {}", display.join(", "));
                if block.referrers.len() > 4 {
                    println!("    ... and {} more", block.referrers.len() - 4);
                }
            }
        }

        println!(" {}", "─".repeat(75).magenta());
        println!(
            "  Total: {} bytes allocated across {} blocks",
            total_allocated.to_string().bright_cyan(),
            blocks.len().to_string().bright_cyan()
        );
        println!(" {}\n", "─".repeat(75).magenta());

        Ok(())
    }

    fn get_heap_blocks_with_referrers(&self) -> Result<Vec<HeapBlockInfo>, VmError> {
        let (reachable, referrers_map) = self.compute_reachable_and_referrers()?;

        let mut blocks: Vec<HeapBlockInfo> = self
            .memory
            .allocations
            .iter()
            .map(|(&base, alloc)| {
                let type_name = self
                    .heap_type_and_data(base)
                    .map(|(ty, _)| format!("{:?}", ty))
                    .unwrap_or_else(|_| "unknown".to_string());

                HeapBlockInfo {
                    base,
                    size: alloc.size,
                    is_static: alloc.is_static,
                    reachable: reachable.contains(&base),
                    type_name,
                    referrers: referrers_map.get(&base).cloned().unwrap_or_default(),
                }
            })
            .collect();

        blocks.sort_by_key(|b| b.base);
        Ok(blocks)
    }

    fn print_memory_report(&self) -> Result<(), VmError> {
        println!("\n {}", "MEMORY REPORT".bold().yellow());
        println!(" {}", "═".repeat(75).yellow());

        let blocks = self.get_heap_blocks_with_referrers()?;
        let orphaned: Vec<_> = blocks
            .iter()
            .filter(|b| !b.reachable && !b.is_static)
            .collect();
        let active: Vec<_> = blocks
            .iter()
            .filter(|b| b.reachable || b.is_static)
            .collect();

        println!(" {} allocations:", "Active".bold().green());
        if active.is_empty() {
            println!("  {}", "none".bright_black());
        } else {
            for block in &active {
                let tag = if block.is_static { "static" } else { "live" };
                println!(
                    "  {} {} bytes ({})",
                    format!("0x{:06x}", block.base).green(),
                    block.size,
                    tag
                );
            }
        }

        println!("\n {} allocations:", "Orphaned".bold().red());
        if orphaned.is_empty() {
            println!("  {}", "No memory leaks detected!".green());
        } else {
            let leaked_bytes: usize = orphaned.iter().map(|b| b.size).sum();
            println!(
                "  {} bytes in {} blocks",
                leaked_bytes.to_string().red(),
                orphaned.len().to_string().red()
            );
            for block in &orphaned {
                println!(
                    "    {} {} bytes (type: {})",
                    format!("0x{:06x}", block.base).red(),
                    block.size,
                    block.type_name.bright_yellow()
                );
                if !block.referrers.is_empty() {
                    println!("      refs from: {}", block.referrers.join(", "));
                } else {
                    println!("      refs from: (none)");
                }
            }
        }

        let total_allocated: usize = blocks.iter().map(|b| b.size).sum();
        let total_leaked: usize = orphaned.iter().map(|b| b.size).sum();

        println!("\n {}", "Summary".bold());
        println!(
            "  Total allocated: {} bytes",
            total_allocated.to_string().cyan()
        );
        println!(
            "  Total leaked: {} bytes",
            if total_leaked == 0 {
                total_leaked.to_string().green()
            } else {
                total_leaked.to_string().red()
            }
        );
        println!(" {}\n", "═".repeat(75).yellow());

        Ok(())
    }

    pub fn check_memory_leaks(&self) -> Result<usize, VmError> {
        let blocks = self.get_heap_blocks_with_referrers()?;
        Ok(blocks
            .iter()
            .filter(|b| !b.reachable && !b.is_static)
            .count())
    }
}
