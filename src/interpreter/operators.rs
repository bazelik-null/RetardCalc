use std::fmt;

#[derive(Debug, Default, Copy, Clone, PartialEq)]
pub enum OperatorType {
    // Arithmetic
    Add,      // x + y
    Subtract, // x - y
    Multiply, // x * y
    Divide,   // x / y
    // Exponents
    Exponent, // x ^ y
    Sqrt,     // √ x
    Log,      // x log(y) [where x is base, y is argument]
    Ln,       // ln(x)
    // Trigonometry
    Cos,  // cos(x)
    Sin,  // sin(x)
    Tan,  // tan(x)
    Acos, // arccos(x)
    Asin, // arcsin(x)
    Atan, // arctan(x)
    // Misc
    Negate, // -x
    Modulo, // x % y
    Abs,    // abs(x)
    Round,  // round(x)
    // Brackets
    LBracket, // (
    RBracket, // )

    #[default]
    Unknown,
}

impl OperatorType {
    pub fn is_additive(&self) -> bool {
        matches!(self, OperatorType::Add | OperatorType::Subtract)
    }

    pub fn is_multiplicative(&self) -> bool {
        matches!(
            self,
            OperatorType::Multiply
                | OperatorType::Divide
                | OperatorType::Log
                | OperatorType::Modulo
        )
    }

    pub fn is_exponentiation(&self) -> bool {
        matches!(self, OperatorType::Exponent)
    }

    pub fn is_unary(&self) -> bool {
        matches!(
            self,
            OperatorType::Negate
                | OperatorType::Sqrt
                | OperatorType::Ln
                | OperatorType::Cos
                | OperatorType::Sin
                | OperatorType::Tan
                | OperatorType::Acos
                | OperatorType::Asin
                | OperatorType::Atan
                | OperatorType::Abs
                | OperatorType::Round
        )
    }
}

impl fmt::Display for OperatorType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            // Arithmetic
            OperatorType::Add => write!(f, "+"),
            OperatorType::Subtract => write!(f, "-"),
            OperatorType::Multiply => write!(f, "*"),
            OperatorType::Divide => write!(f, "/"),
            // Exponents
            OperatorType::Exponent => write!(f, "^"),
            OperatorType::Sqrt => write!(f, "√"),
            OperatorType::Log => write!(f, "log"),
            OperatorType::Ln => write!(f, "ln"),
            // Trigonometry
            OperatorType::Cos => write!(f, "cos"),
            OperatorType::Sin => write!(f, "sin"),
            OperatorType::Tan => write!(f, "tan"),
            OperatorType::Acos => write!(f, "arccos"),
            OperatorType::Asin => write!(f, "arcsin"),
            OperatorType::Atan => write!(f, "arctan"),
            // Misc
            OperatorType::Negate => write!(f, "-"),
            OperatorType::Modulo => write!(f, "%"),
            OperatorType::Abs => write!(f, "abs"),
            OperatorType::Round => write!(f, "round"),
            // Brackets
            OperatorType::LBracket => write!(f, "("),
            OperatorType::RBracket => write!(f, ")"),

            OperatorType::Unknown => write!(f, "?"),
        }
    }
}
