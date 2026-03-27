use crate::core::compiler::codegen::{CodeGenerator, Scope};
use crate::core::compiler::parser::tree::{Node, Parameter};
use crate::core::compiler::preprocessor::token::{LiteralValue, OperatorValue};
use crate::core::shared::bytecode::Opcode::*;
use lasso::Spur;

impl<'a> CodeGenerator<'a> {
    pub fn generate_node(&mut self, node: &Node) -> Result<(), String> {
        match node {
            Node::Literal(value) => self.gen_literal(value),
            Node::ArrayLiteral(_) => Err("Unimplemented".to_string()),
            Node::Identifier(value) => self.gen_identifier(*value),
            Node::Unary { rhs, op } => {
                self.generate_node(rhs.as_ref())?;
                self.gen_op(*op);
                Ok(())
            }
            Node::Binary { lhs, rhs, op } => {
                // Generate arguments and operator
                self.generate_node(lhs.as_ref())?;
                self.generate_node(rhs.as_ref())?;
                self.gen_op(*op);
                Ok(())
            }
            Node::Assignment { target, value } => self.gen_assignment(target, value),
            Node::Block(value) => self.gen_block(value),
            Node::If {
                condition,
                then_branch,
                else_branch,
            } => self.gen_if(condition, then_branch, else_branch),
            Node::While { condition, body } => self.gen_while(condition, body),
            Node::VariableDecl { name, value, .. } => self.gen_variable_decl(*name, value),
            Node::FunctionDecl {
                name, params, body, ..
            } => self.gen_function_decl(*name, params, body),
            Node::FunctionCall { name, args } => self.gen_function_call(name, args),
            Node::ArrayAccess { .. } => Err("Unimplemented".to_string()),
            Node::Return(value) => self.gen_return(value),
        }
    }

    fn gen_literal(&mut self, value: &LiteralValue) -> Result<(), String> {
        match value {
            // Push to stack as immediate
            LiteralValue::Integer(value) => {
                self.emit(PUSH, *value);
            }
            // Push to stack as immediate. For now store as int
            LiteralValue::Float(value) => {
                let value = *value as i32;
                self.emit(PUSH, value);
            }
            // Push to stack as immediate
            LiteralValue::Boolean(value) => {
                let value = *value as i32;
                self.emit(PUSH, value)
            }
            // Push to data section
            LiteralValue::String(value) => {
                let value = self.rodeo.resolve(value);
                self.insert_data(value)?;
            }
        }
        Ok(())
    }

    fn gen_identifier(&mut self, name: Spur) -> Result<(), String> {
        let local_id = self
            .scope
            .lookup(name)
            .ok_or_else(|| "Undefined variable".to_string())?;

        self.emit(LOAD_LOCAL, local_id);
        Ok(())
    }

    fn gen_assignment(&mut self, target: &Node, value: &Node) -> Result<(), String> {
        // Generate argument
        self.generate_node(value)?;

        // Handle different assignment targets
        match target {
            Node::Identifier(name) => {
                let local_id = self
                    .scope
                    .lookup(*name)
                    .ok_or_else(|| "Undefined variable".to_string())?;

                self.emit(STORE_LOCAL, local_id);
            }
            Node::ArrayAccess { .. } => {
                // TODO: Implement array element assignment
                todo!();
            }
            _ => {
                return Err("Invalid assignment target".to_string());
            }
        }

        Ok(())
    }

    fn gen_variable_decl(&mut self, name: Spur, value: &Node) -> Result<(), String> {
        let local_id = self.scope.allocate_local_id();
        self.scope.declare(name, local_id)?;

        self.generate_node(value)?;
        self.emit(STORE_LOCAL, local_id);

        Ok(())
    }

    fn gen_if(
        &mut self,
        condition: &Node,
        then_branch: &Node,
        else_branch: &Option<Box<Node>>,
    ) -> Result<(), String> {
        // Generate condition
        self.generate_node(condition)?;

        // Allocate labels for branches
        let else_label = self.allocate_label_id();
        let end_label = self.allocate_label_id();

        // Emit conditional jump to else branch
        let cond_jump_offset = self.instructions.len();
        self.emit(JMPF, else_label); // Jump if false
        self.request_branch_relocation(cond_jump_offset, else_label)?;

        // Generate then branch
        self.generate_node(then_branch)?;

        // Jump to end
        let then_jump_offset = self.instructions.len();
        self.emit(JMP, end_label);
        self.request_branch_relocation(then_jump_offset, end_label)?;

        // Define else label
        self.define_label(else_label)?;

        // Generate else branch if present
        if let Some(else_body) = else_branch {
            self.generate_node(else_body)?;
        }

        // Define end label
        self.define_label(end_label)?;

        Ok(())
    }

    fn gen_while(&mut self, condition: &Node, body: &Node) -> Result<(), String> {
        // Allocate labels
        let loop_label = self.allocate_label_id();
        let exit_label = self.allocate_label_id();

        // Define loop start label
        self.define_label(loop_label)?;

        // Generate condition
        self.generate_node(condition)?;

        // Jump to exit if condition is false
        let cond_jump_offset = self.instructions.len();
        self.emit(JMPF, exit_label);
        self.request_branch_relocation(cond_jump_offset, exit_label)?;

        // Generate loop body
        self.generate_node(body)?;

        // Jump back to loop start
        let loop_jump_offset = self.instructions.len();
        self.emit(JMP, loop_label);
        self.request_branch_relocation(loop_jump_offset, loop_label)?;

        // Define exit label
        self.define_label(exit_label)?;

        Ok(())
    }

    fn gen_function_decl(
        &mut self,
        name: Spur,
        params: &[Parameter],
        body: &Node,
    ) -> Result<(), String> {
        // Look up the function metadata (assigned in pass 1)
        let func_label = self
            .functions
            .get(&name)
            .ok_or_else(|| "Function not found in metadata".to_string())?
            .label;

        // Define label for function start
        self.define_label(func_label)?;

        // Create new scope for function with current scope as parent
        let parent_scope = std::mem::take(&mut self.scope);
        self.scope = Scope::with_parent(parent_scope);

        // Declare parameters as local variables
        for (idx, param) in params.iter().enumerate() {
            let local_id = idx as i32;
            self.scope.declare(param.name, local_id)?;
        }

        // Update next_local_id to account for parameters
        self.scope.next_local_id = params.len() as i32;

        // Generate function body
        self.generate_node(body)?;

        // Emit return instruction if not present (implicit return)
        self.emit(RET, 0);

        // Restore parent scope
        let func_scope = std::mem::take(&mut self.scope);
        if let Some(parent) = func_scope.parent {
            self.scope = *parent;
        }

        // Reset local_id counter for next function
        self.scope.next_local_id = 0;

        Ok(())
    }

    fn gen_function_call(&mut self, name: &Node, args: &[Node]) -> Result<(), String> {
        // Generate arguments in order (they'll be pushed onto the stack)
        for arg in args {
            self.generate_node(arg)?;
        }

        // Extract function name from Node
        let func_name = match name {
            Node::Identifier(func_name) => *func_name,
            _ => return Err("Function name must be an identifier".to_string()),
        };

        // Look up function label
        let func_label = self
            .scope
            .lookup(func_name)
            .ok_or_else(|| "Undefined function".to_string())?;

        // Emit call instruction with the function label
        let call_offset = self.instructions.len();
        self.emit(CALL, func_label);
        self.request_branch_relocation(call_offset, func_label)?;

        Ok(())
    }

    fn gen_return(&mut self, value: &Option<Box<Node>>) -> Result<(), String> {
        // Generate return value if present
        if let Some(val) = value {
            self.generate_node(val)?;
        } else {
            // Push 0 as default return value
            self.emit(PUSH, 0);
        }

        // Emit return instruction
        self.emit(RET, 0);

        Ok(())
    }

    fn gen_block(&mut self, block: &[Node]) -> Result<(), String> {
        for n in block.iter() {
            self.generate_node(n)?;
        }

        Ok(())
    }

    fn gen_op(&mut self, op: OperatorValue) {
        match op {
            OperatorValue::Plus => self.emit(ADD, 0),
            OperatorValue::Minus => self.emit(SUB, 0),
            OperatorValue::Multiply => self.emit(MUL, 0),
            OperatorValue::Divide => self.emit(DIV, 0),
            OperatorValue::Modulo => self.emit(REM, 0),
            OperatorValue::Power => self.emit(POW, 0),
            OperatorValue::Equal => self.emit(EQ, 0),
            OperatorValue::NotEqual => self.emit(NE, 0),
            OperatorValue::Not => self.emit(NOT, 0),
            OperatorValue::Greater => self.emit(GT, 0),
            OperatorValue::Less => self.emit(LT, 0),
            OperatorValue::GreaterEqual => self.emit(GE, 0),
            OperatorValue::LessEqual => self.emit(LE, 0),
            OperatorValue::And => self.emit(AND, 0),
            OperatorValue::Or => self.emit(OR, 0),
        }
    }
}
