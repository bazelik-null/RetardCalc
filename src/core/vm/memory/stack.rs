use crate::core::vm::error::VmError;
use crate::core::vm::memory::Memory;
use crate::core::vm::number::Value;

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

impl Memory {
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
            frame.locals.resize(index + 1, value);
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

    /// Create a stack reference to a local variable
    pub fn create_stack_ref(&self, local_index: usize) -> Result<Value, VmError> {
        let frame_count = self.call_stack.len();
        if frame_count == 0 {
            return Err(VmError::NoActiveFrame);
        }

        let frame_index = frame_count - 1; // Current frame
        let frame = &self.call_stack[frame_index];

        if local_index >= frame.locals.len() {
            return Err(VmError::LocalOutOfBounds(local_index));
        }

        Ok(Value::StackRef {
            frame_index,
            local_index,
        })
    }

    /// Dereference a stack reference and get the value
    pub fn dereference_stack_ref(
        &self,
        frame_index: usize,
        local_index: usize,
    ) -> Result<Value, VmError> {
        if frame_index >= self.call_stack.len() {
            return Err(VmError::LocalOutOfBounds(frame_index));
        }

        let frame = &self.call_stack[frame_index];
        frame
            .locals
            .get(local_index)
            .copied()
            .ok_or(VmError::LocalOutOfBounds(local_index))
    }

    /// Update a value through a stack reference
    pub fn set_through_stack_ref(
        &mut self,
        frame_index: usize,
        local_index: usize,
        value: Value,
    ) -> Result<(), VmError> {
        if frame_index >= self.call_stack.len() {
            return Err(VmError::LocalOutOfBounds(frame_index));
        }

        let frame = &mut self.call_stack[frame_index];
        if local_index >= frame.locals.len() {
            frame.locals.resize(local_index + 1, value);
        }
        frame.locals[local_index] = value;
        Ok(())
    }

    /// Check if a stack reference is still valid
    pub fn is_stack_ref_valid(&self, frame_index: usize, local_index: usize) -> bool {
        frame_index < self.call_stack.len()
            && local_index < self.call_stack[frame_index].locals.len()
    }

    /// Dereference any value that might be a stack reference
    pub fn resolve_value(&self, value: Value) -> Result<Value, VmError> {
        match value {
            Value::StackRef {
                frame_index,
                local_index,
            } => {
                if !self.is_stack_ref_valid(frame_index, local_index) {
                    return Err(VmError::InvalidReference(frame_index));
                }
                self.dereference_stack_ref(frame_index, local_index)
            }
            other => Ok(other),
        }
    }

    pub fn peek_stack(&self) -> &[Value] {
        &self.operand_stack
    }
}
