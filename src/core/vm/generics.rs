use crate::core::compiler::parser::tree::Type;
use crate::core::shared::bytecode::Opcode;
use crate::core::vm::error::VmError;
use crate::core::vm::memory::Value;
use crate::core::vm::{Num, VirtualMachine};

impl VirtualMachine {
    /// Generic binary numeric op
    pub fn numeric_binop<FI, FF>(&mut self, int_op: FI, float_op: FF) -> Result<(), VmError>
    where
        FI: Fn(i32, i32) -> i32,
        FF: Fn(f32, f32) -> f32,
    {
        let vb = self.memory.peek()?;
        let va = self.memory.peek()?;
        let na = self.value_to_num(va)?;
        let nb = self.value_to_num(vb)?;
        let res = match (na, nb) {
            (Num::Int(ai), Num::Int(bi)) => Num::Int(int_op(ai, bi)),
            (a, b) => Num::Float(float_op(a.to_f32(), b.to_f32())),
        };
        self.push_num(res)
    }

    /// Generic unary numeric op
    pub fn numeric_unary<F>(&mut self, float_op: F) -> Result<(), VmError>
    where
        F: Fn(f32) -> f32,
    {
        let value = self.memory.pop()?;
        let n = self.value_to_num(value)?;
        match n {
            Num::Int(i) => self.push_int(-i),
            Num::Float(f) => {
                let out = float_op(f);
                self.push_num(Num::Float(out))
            }
        }
    }

    /// Require an integer value (Value::Int or ref to Integer)
    pub fn require_int_value(&mut self, value: Value) -> Result<i32, VmError> {
        match value {
            Value::Int(i) => Ok(i),
            Value::Ref(addr) => {
                let (ty, data) = self.heap_type_and_data(addr)?;
                if ty != Type::Integer {
                    return Err(VmError::type_mismatch(
                        "integer",
                        format!("ref(0x{:x})", addr),
                    ));
                }
                if data.len() < 4 {
                    return Err(VmError::type_mismatch(
                        "integer",
                        format!("small data at 0x{:x}", addr),
                    ));
                }
                let mut arr = [0u8; 4];
                arr.copy_from_slice(&data[0..4]);
                Ok(i32::from_le_bytes(arr))
            }
        }
    }

    /// Compares strings and numerics
    pub fn compare_generic(&mut self, opcode: Opcode) -> Result<(), VmError> {
        let vb = self.memory.pop()?;
        let va = self.memory.pop()?;

        // Both refs and both strings
        if let (Value::Ref(a_addr), Value::Ref(b_addr)) = (&va, &vb) {
            let (ta, da) = self.heap_type_and_data(*a_addr)?;
            let (tb, db) = self.heap_type_and_data(*b_addr)?;
            if ta == Type::String && tb == Type::String {
                let sa = std::str::from_utf8(da).unwrap_or_default();
                let sb = std::str::from_utf8(db).unwrap_or_default();
                let res = match opcode {
                    Opcode::EQ => (sa == sb) as i32,
                    Opcode::NE => (sa != sb) as i32,
                    Opcode::LT => (sa < sb) as i32,
                    Opcode::GT => (sa > sb) as i32,
                    Opcode::LE => (sa <= sb) as i32,
                    Opcode::GE => (sa >= sb) as i32,
                    _ => unreachable!(),
                };
                return self.push_int(res);
            }
        }

        // Numeric
        let na = self.value_to_num(va)?;
        let nb = self.value_to_num(vb)?;
        let af = na.to_f32();
        let bf = nb.to_f32();
        let res = match opcode {
            Opcode::EQ => (af == bf) as i32,
            Opcode::NE => (af != bf) as i32,
            Opcode::LT => (af < bf) as i32,
            Opcode::GT => (af > bf) as i32,
            Opcode::LE => (af <= bf) as i32,
            Opcode::GE => (af >= bf) as i32,
            _ => unreachable!(),
        };
        self.push_int(res)
    }
}
