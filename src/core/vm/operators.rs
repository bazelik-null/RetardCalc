use crate::core::compiler::parser::tree::Type;
use crate::core::shared::builtin_func::SysCallId;
use crate::core::vm::error::VmError;
use crate::core::vm::memory::Value;
use crate::core::vm::{Num, VirtualMachine};
use std::io;
use std::io::Write;

impl VirtualMachine {
    /// Numeric add or string concat (string if either operand is string)
    pub fn op_add(&mut self) -> Result<(), VmError> {
        let vb = self.memory.pop()?;
        let va = self.memory.pop()?;

        // Determine types
        let ta = self.get_type(&va)?;
        let tb = self.get_type(&vb)?;

        // If numbers
        if matches!(ta, Some(Type::Integer) | Some(Type::Float))
            && matches!(tb, Some(Type::Integer) | Some(Type::Float))
        {
            return self.numeric_binop(|x, y| x.wrapping_add(y), |x, y| x + y);
        }

        // If strings
        let sa = self.value_to_string(&va)?;
        let sb = self.value_to_string(&vb)?;

        let (rtti, data) = self.concat_string(&sa, &sb)?;
        let addr = self.memory.save_to_heap(&rtti, &data, false)?;
        self.push_ref(addr)?;

        Ok(())
    }

    fn concat_string(&self, sa: &str, sb: &str) -> Result<(Vec<u8>, Vec<u8>), VmError> {
        let mut combined = Vec::with_capacity(sa.len() + sb.len());
        combined.extend_from_slice(sa.as_bytes());
        combined.extend_from_slice(sb.as_bytes());
        self.build_data(combined, Type::String)
    }

    pub fn op_sub(&mut self) -> Result<(), VmError> {
        self.numeric_binop(|x, y| x.wrapping_sub(y), |x, y| x - y)
    }

    pub fn op_mul(&mut self) -> Result<(), VmError> {
        self.numeric_binop(|x, y| x.wrapping_mul(y), |x, y| x * y)
    }

    pub fn op_div(&mut self) -> Result<(), VmError> {
        let vb = self.memory.pop()?;
        let va = self.memory.pop()?;

        // Check divisor
        self.check_divisor(va)?;
        self.check_divisor(vb)?;

        // Push back and perform division
        self.memory.push(va)?;
        self.memory.push(vb)?;
        self.numeric_binop(|x, y| x / y, |x, y| x / y)
    }

    pub fn op_rem(&mut self) -> Result<(), VmError> {
        let vb = self.memory.pop()?;
        let va = self.memory.pop()?;

        // Check divisor
        self.check_divisor(va)?;
        self.check_divisor(vb)?;

        // Push back and perform mod
        self.memory.push(va)?;
        self.memory.push(vb)?;
        self.numeric_binop(|x, y| x % y, |x, y| x % y)
    }

    fn check_divisor(&mut self, value: Value) -> Result<(), VmError> {
        if let Ok(value) = self.value_to_num(value) {
            match value {
                Num::Int(0) => Err(VmError::type_mismatch("non-zero", "divisor")),
                Num::Float(0.0) => Err(VmError::type_mismatch("non-zero", "divisor")),
                _ => Ok(()),
            }
        } else {
            Err(VmError::type_mismatch("numeric", "divisor"))
        }
    }

    pub fn op_pow(&mut self) -> Result<(), VmError> {
        let vb = self.memory.pop()?;
        let va = self.memory.pop()?;

        // Determine types
        let na_res = self.value_to_num(va);
        let nb_res = self.value_to_num(vb);

        if let (Ok(Num::Int(ai)), Ok(Num::Int(bi))) = (na_res, nb_res) {
            // if exponent negative, fallback to float pow
            if bi >= 0 {
                let exp = bi as u32;
                let res = ai.wrapping_pow(exp);
                return self.push_int(res);
            }
        }

        // Fallback to float powf
        self.memory.push(va)?;
        self.memory.push(vb)?;
        self.numeric_binop(|x, y| (x as f32).powf(y as f32) as i32, |x, y| x.powf(y))
    }

    pub fn op_neg(&mut self) -> Result<(), VmError> {
        self.numeric_unary(|x| -x)
    }

    // Bitwise ops (require integer)
    pub fn op_and(&mut self) -> Result<(), VmError> {
        let b = self.memory.pop()?;
        let a = self.memory.pop()?;
        let bi = self.require_int_value(b)?;
        let ai = self.require_int_value(a)?;
        self.push_int(ai & bi)
    }

    pub fn op_or(&mut self) -> Result<(), VmError> {
        let b = self.memory.pop()?;
        let a = self.memory.pop()?;
        let bi = self.require_int_value(b)?;
        let ai = self.require_int_value(a)?;
        self.push_int(ai | bi)
    }

    pub fn op_xor(&mut self) -> Result<(), VmError> {
        let b = self.memory.pop()?;
        let a = self.memory.pop()?;
        let bi = self.require_int_value(b)?;
        let ai = self.require_int_value(a)?;
        self.push_int(ai ^ bi)
    }

    pub fn op_not(&mut self) -> Result<(), VmError> {
        let a = self.memory.pop()?;
        let ai = self.require_int_value(a)?;
        self.push_int(!ai)
    }

    pub fn op_sla(&mut self) -> Result<(), VmError> {
        let b = self.memory.pop()?;
        let a = self.memory.pop()?;
        let bi = self.require_int_value(b)? as u32;
        let ai = self.require_int_value(a)?;
        self.push_int(ai.wrapping_shl(bi))
    }

    pub fn op_sra(&mut self) -> Result<(), VmError> {
        let b = self.memory.pop()?;
        let a = self.memory.pop()?;
        let bi = self.require_int_value(b)? as u32;
        let ai = self.require_int_value(a)?;
        self.push_int((ai as u32).wrapping_shr(bi) as i32)
    }

    /// Pop reference address (an address to a heap object) and push value or ref
    pub fn op_load(&mut self) -> Result<(), VmError> {
        let addr = self.pop_ref()?;
        let (ty, data) = self.heap_type_and_data(addr)?;
        match ty {
            Type::Integer => {
                if data.len() < 4 {
                    return Err(VmError::type_mismatch(
                        "integer",
                        format!("small data at 0x{:x}", addr),
                    ));
                }
                let mut arr = [0u8; 4];
                arr.copy_from_slice(&data[0..4]);
                let val = i32::from_le_bytes(arr);
                self.push_int(val)?;
            }
            // For Float/String/Complex types push the reference to the heap object
            Type::Float
            | Type::String
            | Type::Reference(_)
            | Type::Array(_)
            | Type::FixedArray(_, _) => {
                self.push_ref(addr)?;
            }
            _ => return Err(VmError::type_mismatch("loadable", format!("{:?}", ty))),
        }
        Ok(())
    }

    /// Pop value, address, and write to target based on type
    pub fn op_store(&mut self) -> Result<(), VmError> {
        let val = self.memory.pop()?;
        let addr = self.pop_ref()?;

        // If value is an int and target expects integer, write payload
        if let Value::Int(i) = val {
            let (ty, _data) = self.heap_type_and_data(addr)?;
            if ty != Type::Integer {
                return Err(VmError::type_mismatch(
                    "integer target",
                    format!("{:?}", ty),
                ));
            }
            // Write RTTI+payload: create buffer of RTTI then 4 bytes payload
            let mut buf = Type::Integer.to_bytes();
            buf.extend_from_slice(&i.to_le_bytes());
            self.memory.write_bytes(addr, &buf)?;
            return Ok(());
        }

        // If val is a ref, copy the source object's RTTI+data into destination
        if let Value::Ref(src_addr) = val {
            let (rtti_src, data_src) = self.memory.load_from_heap(src_addr)?;
            let mut buf = Vec::with_capacity(rtti_src.len() + data_src.len());
            buf.extend_from_slice(rtti_src);
            buf.extend_from_slice(data_src);
            self.memory.write_bytes(addr, &buf)?;
            return Ok(());
        }

        Err(VmError::type_mismatch("storable", "value"))
    }

    pub fn op_syscall(&mut self, id: u8) -> Result<(), VmError> {
        // Convert operant into ID
        let id = SysCallId::from_u8(id).map_err(|e| VmError::type_mismatch("syscall id", e))?;

        // Pop args count
        let argc_val = self.memory.pop()?;
        let argc = match argc_val {
            Value::Int(i) if i >= 0 => i as usize,
            Value::Int(_) => {
                return Err(VmError::type_mismatch(
                    "non-negative integer",
                    format!("arg count {:?}", argc_val),
                ));
            }
            Value::Ref(_) => {
                return Err(VmError::type_mismatch(
                    "integer",
                    format!("arg count {:?}", argc_val),
                ));
            }
        };

        // Pop arguments
        let mut args = Vec::with_capacity(argc);
        for _ in 0..argc {
            args.push(self.memory.pop()?);
        }
        args.reverse();

        // Call syscalls
        match id {
            SysCallId::Print => self.op_print(&args),
            SysCallId::Println => self.op_println(&args),
            SysCallId::Input => self.op_input(&args),
        }?;

        Ok(())
    }

    fn op_print(&mut self, args: &[Value]) -> Result<(), VmError> {
        for val in args {
            let s = self.value_to_string(val)?;
            print!("{}", s);
            io::stdout().flush().unwrap();
        }
        Ok(())
    }

    fn op_println(&mut self, args: &[Value]) -> Result<(), VmError> {
        for val in args {
            let s = self.value_to_string(val)?;
            println!("{}", s);
        }
        Ok(())
    }

    fn op_input(&mut self, args: &[Value]) -> Result<(), VmError> {
        // Print prompt
        self.op_print(args)?;

        // Get line
        let mut input = String::new();
        io::stdin()
            .read_line(&mut input)
            .map_err(|e| VmError::runtime(e.to_string()))?;

        // Save data to heap
        let (rtti, data) = self.build_data(input, Type::String)?;
        let addr = self.memory.save_to_heap(&rtti, &data, false)?;

        // Push reference to stack
        self.push_ref(addr)?;

        Ok(())
    }
}
