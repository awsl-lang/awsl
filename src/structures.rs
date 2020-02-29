use std::collections::HashMap;
use std::sync::{Arc, RwLock};
pub type Script = Arc<HashMap<String, Function>>;
/// `Function` is a set of process instruction (`Expression`) and variables definition (both input and outbut)
pub struct Function {
    /// Function's input variable name
    pub input: Vec<String>,
    /// How to process input variable
    pub process: Vec<Arc<RwLock<Expression>>>,
    /// Function's output variable (if any)
    pub output: Option<String>,
}
pub struct Expression {
    /// What is the expression
    pub operation: Operation,
    /// What variables should be sent to operation
    pub variables: Vec<String>,
    /// What's next / Should result stored
    pub to: ExpressionTo,
}
pub enum Operation {
    Builtin(String),
    External(String, String),
}
pub enum ExpressionTo {
    ToVar(String),
    ToBlock(Arc<RwLock<Expression>>),
    Nil,
}
