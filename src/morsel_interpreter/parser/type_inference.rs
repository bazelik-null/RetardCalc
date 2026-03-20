use crate::morsel_interpreter::environment::types::Type;
use crate::morsel_interpreter::parser::ast_node::Node;
use crate::morsel_interpreter::parser::builder::AstBuilder;

impl<'a> AstBuilder<'a> {
    /// Infer the type of node by recursively analyzing its structure
    pub fn infer_type_from_node(&self, node: &Node) -> Result<Type, String> {
        match node {
            Node::Literal(value) => Ok(value.type_of()),

            Node::Reference(name) => self
                .symbol_table
                .variables
                .lookup(name)
                .map(|var| var.type_annotation)
                .ok_or_else(|| format!("Variable '{}' is not defined", name)),

            Node::Call { name, args } => self.infer_call_type(name, args),

            Node::Block(statements) => self.infer_block_type(statements),

            // Statements return Unit type
            Node::Assignment { .. } | Node::LetBinding { .. } | Node::FuncBinding() => {
                Ok(Type::Unit)
            }
        }
    }

    /// Infer type of function call or operator application
    #[inline]
    fn infer_call_type(&self, name: &str, args: &[Node]) -> Result<Type, String> {
        match (name, args.len()) {
            // Unary operators
            ("-" | "!", 1) => {
                let operand_type = self.infer_type_from_node(&args[0])?;
                self.infer_unary_op_type(name, &operand_type)
            }
            // Binary operators
            (
                "+" | "-" | "*" | "/" | "%" | "^" | "==" | "!=" | "<" | "<=" | ">" | ">=" | "&&"
                | "||",
                2,
            ) => {
                let left_type = self.infer_type_from_node(&args[0])?;
                let right_type = self.infer_type_from_node(&args[1])?;
                self.infer_binary_op_type(name, &left_type, &right_type)
            }
            // Function call (any name with any arity)
            _ => {
                let func = self
                    .symbol_table
                    .functions
                    .lookup(name)
                    .ok_or_else(|| format!("Function '{}' is not defined", name))?;
                Ok(func.metadata.return_type)
            }
        }
    }

    /// Infer type of block (last expression determines type)
    #[inline]
    fn infer_block_type(&self, statements: &[Node]) -> Result<Type, String> {
        statements
            .last()
            .map(|last| {
                if matches!(last, Node::LetBinding { .. } | Node::FuncBinding()) {
                    Ok(Type::Unit)
                } else {
                    self.infer_type_from_node(last)
                }
            })
            .unwrap_or(Ok(Type::Unit))
    }

    /// Infer type of binary operation with type compatibility check
    fn infer_binary_op_type(
        &self,
        op: &str,
        left_type: &Type,
        right_type: &Type,
    ) -> Result<Type, String> {
        match op {
            // Arithmetic operators
            "+" | "-" | "*" | "/" | "%" | "^" => {
                self.infer_arithmetic_op_type(op, left_type, right_type)
            }
            // Comparison operators always return boolean
            "==" | "!=" | "<" | "<=" | ">" | ">=" => Ok(Type::Boolean),
            // Logical operators require boolean operands
            "&&" | "||" => {
                if *left_type == Type::Boolean && *right_type == Type::Boolean {
                    Ok(Type::Boolean)
                } else {
                    Err(format!(
                        "Operator '{}' requires boolean operands, got {} and {}",
                        op, left_type, right_type
                    ))
                }
            }
            _ => Err(format!("Unknown binary operator: {}", op)),
        }
    }

    /// Infer type of arithmetic operations
    fn infer_arithmetic_op_type(
        &self,
        op: &str,
        left_type: &Type,
        right_type: &Type,
    ) -> Result<Type, String> {
        // Type compatibility check
        if !left_type.is_compatible_with(right_type) {
            return Err(format!(
                "Type mismatch in '{}': {} and {}",
                op, left_type, right_type
            ));
        }

        match left_type {
            Type::Integer | Type::Float => Ok(*left_type),
            Type::String if op == "+" => Ok(Type::String), // String concatenation
            _ => Err(format!(
                "Operator '{}' cannot be applied to type {}",
                op, left_type
            )),
        }
    }

    /// Infer type of unary operation
    fn infer_unary_op_type(&self, op: &str, operand_type: &Type) -> Result<Type, String> {
        match op {
            // Negation requires numeric type
            "-" => match operand_type {
                Type::Integer | Type::Float => Ok(*operand_type),
                _ => Err(format!(
                    "Unary operator '-' cannot be applied to type {}",
                    operand_type
                )),
            },
            // Logical NOT requires boolean type
            "!" => {
                if *operand_type == Type::Boolean {
                    Ok(Type::Boolean)
                } else {
                    Err(format!(
                        "Operator '!' requires boolean operand, got {}",
                        operand_type
                    ))
                }
            }
            _ => Err(format!("Unknown unary operator: {}", op)),
        }
    }
}
