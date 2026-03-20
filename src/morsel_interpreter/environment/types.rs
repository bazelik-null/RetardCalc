use crate::morsel_interpreter::environment::value::Value;
use std::str::FromStr;

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub enum Type {
    // Real types
    Integer,
    Float,
    String,
    Boolean,
    // Utility types
    Any,  // Unsafe
    Null, // The null literal value (nullable type)
    Unit, // Statements, assignments, functions with no return
}

impl Type {
    pub fn of(value: &Value) -> Self {
        match value {
            Value::Integer(_) => Type::Integer,
            Value::Float(_) => Type::Float,
            Value::String(_) => Type::String,
            Value::Boolean(_) => Type::Boolean,
            Value::Null => Type::Null,
        }
    }

    /// Check if this type is compatible with another (for implicit conversions)
    pub fn is_compatible_with(&self, other: &Type) -> bool {
        match (self, other) {
            (a, b) if a == b => true,
            // Type::Any accepts anything
            (_, Type::Any) => true,
            (Type::Any, _) => true,

            (Type::Integer, Type::Float) => true,
            (Type::Float, Type::Integer) => true,
            _ => false,
        }
    }

    /// Check if explicit conversion is allowed
    pub fn can_convert_to(&self, target: &Type) -> bool {
        match (self, target) {
            (a, b) if a == b => true,
            // Type::Any accepts anything
            (_, Type::Any) => true,
            (Type::Any, _) => true,
            // Only allow numeric conversions (explicit casts)
            (Type::Integer, Type::Float) | (Type::Float, Type::Integer) => true,
            // Only allow conversion TO string (from any type)
            (_, Type::String) => true,
            // Null can be assigned to any type (optional types)
            (Type::Null, _) => true,
            _ => false,
        }
    }
}

impl FromStr for Type {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "int" => Ok(Type::Integer),
            "float" => Ok(Type::Float),
            "string" => Ok(Type::String),
            "bool" => Ok(Type::Boolean),
            "null" => Ok(Type::Null),
            "unit" => Ok(Type::Unit),
            "any" => Ok(Type::Any),
            _ => Err(format!("Unknown type: {}", s)),
        }
    }
}

impl std::fmt::Display for Type {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Type::Integer => write!(f, "int"),
            Type::Float => write!(f, "float"),
            Type::String => write!(f, "string"),
            Type::Boolean => write!(f, "bool"),
            Type::Null => write!(f, "null"),
            Type::Unit => write!(f, "unit"),
            Type::Any => write!(f, "any"),
        }
    }
}
