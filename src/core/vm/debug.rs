use crate::core::shared::types::Type;
use crate::core::vm::error::VmError;
use crate::core::vm::number::Value;
use crate::core::vm::VirtualMachine;
use colored::Colorize;
use std::collections::{HashMap, HashSet, VecDeque};

type ReachableAndReferrers = (HashSet<usize>, HashMap<usize, Vec<String>>);

struct HeapBlockInfo {
    base: usize,
    size: usize,
    is_static: bool,
    marked: bool,
    reachable: bool,
    type_name: String,
    referrers: Vec<String>,
}

// Formatting helpers to reduce repetition
struct Fmt;
impl Fmt {
    fn addr(addr: usize) -> String {
        format!("0x{:06x}", addr).cyan().to_string()
    }

    fn sep(color: impl Fn(&str) -> colored::ColoredString) -> String {
        color(&"═".repeat(70)).to_string()
    }

    fn sep_dash(color: impl Fn(&str) -> colored::ColoredString) -> String {
        color(&"─".repeat(70)).to_string()
    }

    fn bytes(b: usize) -> String {
        format!("{:>6} Bytes", b)
    }

    fn tag_status(block: &HeapBlockInfo) -> String {
        if block.is_static {
            "STATIC".bright_magenta().bold().to_string()
        } else if block.marked {
            "MARKED".bright_green().to_string()
        } else if block.reachable {
            "LIVE".green().to_string()
        } else {
            "UNREACHABLE".red().bold().to_string()
        }
    }

    fn tag_type(block: &HeapBlockInfo) -> String {
        block.type_name.bright_yellow().to_string()
    }
}

impl VirtualMachine {
    pub fn run_debug(&mut self) -> Result<(), VmError> {
        self.debug = true;
        self.memory.debug = true;
        self.memory.push_frame(0);

        println!("\n {}", "VM EXECUTION".bold().cyan());
        println!(" {}", Fmt::sep(|s| s.cyan()));

        while !self.halted && self.pc < self.instructions.len() {
            self.print_debug_line()?;
            self.step()?;
        }

        self.memory.collect_garbage()?;

        println!(" {}", Fmt::sep(|s| s.cyan()));
        self.print_memory_report()?;
        println!(" {}\n", "END OF EXECUTION".bold().cyan());
        Ok(())
    }

    fn print_debug_line(&self) -> Result<(), VmError> {
        let addr = Fmt::addr(self.pc);
        let instr = if self.pc < self.instructions.len() {
            format!("{:<30}", self.instructions[self.pc].to_string())
        } else {
            format!("{:<30}", "OUT OF BOUNDS".red().bold().to_string())
        };

        let stack = self.format_stack_display()?;
        let stack_display = format!("{:<50}", stack);

        println!("  {}  {}  {}", addr, instr, stack_display);
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
                Value::Imm(n) => format!("Imm<{}>", n).green().to_string(),
                Value::Ref(addr) => format!("Ref<0x{:X}>", addr).bright_cyan().to_string(),
                Value::StackRef { local_index, .. } => format!("StackRef<0x{:X}>", local_index)
                    .bright_blue()
                    .to_string(),
            })
            .collect();

        Ok(format!("[{}]", items.join(",")))
    }

    fn compute_reachable_and_referrers(&self) -> Result<ReachableAndReferrers, VmError> {
        let mut work: VecDeque<usize> = VecDeque::new();
        let mut marked: HashSet<usize> = HashSet::new();
        let mut referrers: HashMap<usize, Vec<String>> = HashMap::new();

        // Add base to worklist
        let mut add_root = |base: usize, label: Option<String>| {
            if marked.insert(base) {
                work.push_back(base);
            }
            if let Some(lbl) = label {
                referrers.entry(base).or_default().push(lbl);
            }
        };

        // Process operand stack
        for (i, v) in self.memory.operand_stack.iter().enumerate() {
            if let Value::Ref(addr) = v {
                if let Ok(base) = self.memory.find_object_base(*addr) {
                    add_root(base, Some(format!("stack[{}]", i)));
                }
            }
        }

        // Process call stack locals
        for (fi, frame) in self.memory.call_stack.iter().enumerate() {
            for (li, v) in frame.locals.iter().enumerate() {
                if let Value::Ref(addr) = v {
                    if let Ok(base) = self.memory.find_object_base(*addr) {
                        add_root(base, Some(format!("frame[{}].local[{}]", fi, li)));
                    }
                }
            }
        }

        // Process static allocations
        let mut addr = 0;
        while addr < self.memory.next_free {
            if self.memory.is_valid_object_header(addr) {
                if let Ok(true) = self.memory.is_static(addr) {
                    add_root(addr, Some("static".to_string()));
                }
                addr += self.memory.get_object_size(addr).unwrap_or(1);
            } else {
                addr += 1;
            }
        }

        // Mark reachable objects
        while let Some(base) = work.pop_front() {
            if let Ok((_rtti, data)) = self.memory.load_from_heap(base) {
                if let Ok((type_info, _)) = Type::from_bytes(_rtti) {
                    for offset in type_info.pointer_offsets() {
                        if offset + 8 <= data.len() {
                            let candidate = usize::from_le_bytes(
                                data[offset..offset + 8].try_into().unwrap_or([0; 8]),
                            );

                            if let Ok(candidate_base) = self.memory.find_object_base(candidate) {
                                referrers
                                    .entry(candidate_base)
                                    .or_default()
                                    .push(format!("heap[{:06x}]+{}", base, offset));

                                if marked.insert(candidate_base) {
                                    work.push_back(candidate_base);
                                }
                            }
                        }
                    }
                }
            }
        }

        Ok((marked, referrers))
    }

    pub fn print_heap_view(&self) -> Result<(), VmError> {
        println!("\n {}", "HEAP VIEW".bold().magenta());
        println!(" {}", Fmt::sep_dash(|s| s.magenta()));

        let blocks = self.get_heap_blocks_with_referrers()?;

        if blocks.is_empty() {
            println!("  {} (no allocations)", "empty".bright_black());
            return Ok(());
        }

        let total_allocated: usize = blocks.iter().map(|b| b.size).sum();

        for block in &blocks {
            println!(
                "  {} {}  {} {}",
                Fmt::addr(block.base),
                Fmt::bytes(block.size),
                Fmt::tag_type(block),
                Fmt::tag_status(block)
            );

            if !block.referrers.is_empty() {
                let display: Vec<_> = block.referrers.iter().take(3).cloned().collect();
                println!("    refs: {}", display.join(", "));
                if block.referrers.len() > 3 {
                    println!("    ... +{} more", block.referrers.len() - 3);
                }
            }
        }

        println!(" {}", Fmt::sep_dash(|s| s.magenta()));
        println!(
            "  Total: {} bytes in {} blocks",
            total_allocated.to_string().bright_cyan(),
            blocks.len().to_string().bright_cyan()
        );
        println!(" {}\n", Fmt::sep_dash(|s| s.magenta()));

        Ok(())
    }

    fn get_heap_blocks_with_referrers(&self) -> Result<Vec<HeapBlockInfo>, VmError> {
        let (reachable, referrers_map) = self.compute_reachable_and_referrers()?;
        let mut blocks: Vec<HeapBlockInfo> = Vec::new();
        let mut addr = 0;

        while addr < self.memory.next_free {
            if self.memory.is_valid_object_header(addr) {
                let size = self.memory.get_object_size(addr)?;
                let is_static = self.memory.is_static(addr)?;
                let marked = self.memory.is_marked(addr)?;
                let type_name = self
                    .heap_get_type_and_data(addr)
                    .map(|(ty, _)| format!("{:?}", ty))
                    .unwrap_or_else(|_| "?".to_string());

                blocks.push(HeapBlockInfo {
                    base: addr,
                    size,
                    is_static,
                    marked,
                    reachable: reachable.contains(&addr),
                    type_name,
                    referrers: referrers_map.get(&addr).cloned().unwrap_or_default(),
                });

                addr += size;
            } else {
                addr += 1;
            }
        }

        Ok(blocks)
    }

    fn print_memory_report(&self) -> Result<(), VmError> {
        println!("\n {}", "MEMORY REPORT".bold().yellow());
        println!(" {}", Fmt::sep(|s| s.yellow()));

        let blocks = self.get_heap_blocks_with_referrers()?;
        let orphaned: Vec<_> = blocks
            .iter()
            .filter(|b| !b.reachable && !b.is_static)
            .collect();
        let active: Vec<_> = blocks
            .iter()
            .filter(|b| b.reachable || b.is_static)
            .collect();

        self.print_block_section("Active", &active)?;
        self.print_block_section("Orphaned", &orphaned)?;

        let total_allocated: usize = blocks.iter().map(|b| b.size).sum();
        let total_leaked: usize = orphaned.iter().map(|b| b.size).sum();

        println!("\n {}", "Summary".bold());
        println!("  Total: {} bytes", total_allocated.to_string().cyan());
        println!(
            "  Leaked: {} bytes",
            if total_leaked == 0 {
                total_leaked.to_string().green()
            } else {
                total_leaked.to_string().red()
            }
        );
        println!(" {}\n", Fmt::sep(|s| s.yellow()));

        Ok(())
    }

    fn print_block_section(&self, label: &str, blocks: &[&HeapBlockInfo]) -> Result<(), VmError> {
        let color = if label == "Active" {
            |s: &str| s.green().bold()
        } else {
            |s: &str| s.red().bold()
        };

        println!(" {} allocations:", color(label));

        if blocks.is_empty() {
            let msg = if label == "Active" {
                "none"
            } else {
                "No leaks!"
            };
            println!("  {}", msg.bright_black());
            return Ok(());
        }

        for block in blocks {
            let tag = if block.is_static {
                "STATIC".bright_magenta()
            } else if block.marked {
                "MARKED".bright_green()
            } else {
                "LIVE".green()
            };

            println!(
                "  {} {} [{}]",
                Fmt::addr(block.base),
                Fmt::bytes(block.size),
                tag
            );
        }

        Ok(())
    }

    pub fn check_memory_leaks(&self) -> Result<usize, VmError> {
        let blocks = self.get_heap_blocks_with_referrers()?;
        Ok(blocks
            .iter()
            .filter(|b| !b.reachable && !b.is_static)
            .count())
    }

    pub fn diagnose_fragmentation(&self) -> Result<(), VmError> {
        println!("\n {}", "FRAGMENTATION ANALYSIS".bold().yellow());
        println!(" {}", Fmt::sep_dash(|s| s.yellow()));

        let mut addr = 0;
        let mut gaps = Vec::new();
        let mut last_end = 0;

        while addr < self.memory.next_free {
            if self.memory.is_valid_object_header(addr) {
                if addr > last_end {
                    gaps.push((last_end, addr - last_end));
                }
                let size = self.memory.get_object_size(addr)?;
                last_end = addr + size;
                addr += size;
            } else {
                addr += 1;
            }
        }

        if gaps.is_empty() {
            println!("  {} No fragmentation", "✓".green());
        } else {
            println!("  {} {} gaps:", "⚠".yellow(), gaps.len());
            let total_gap: usize = gaps.iter().map(|(_, size)| size).sum();

            for (start, size) in gaps {
                println!(
                    "    {} → {} : {} B",
                    Fmt::addr(start),
                    Fmt::addr(start + size),
                    size
                );
            }

            let pct = (total_gap as f64 / self.memory.next_free as f64) * 100.0;
            println!("  Total: {} bytes ({:.1}%)", total_gap, pct);
        }

        println!(" {}\n", Fmt::sep_dash(|s| s.yellow()));
        Ok(())
    }
}
