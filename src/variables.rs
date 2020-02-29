#[derive(Debug)]
pub enum Primitive {
    Nil,
    Literal(String),
}
impl Clone for Primitive {
    fn clone(&self) -> Self {
        match &self {
            Primitive::Nil => Self::Nil,
            Primitive::Literal(i) => Self::Literal(i.clone()),
        }
    }
}
pub enum Complex {
    Primitive(Primitive),
    Stack(Vec<Primitive>),
}
impl Clone for Complex {
    fn clone(&self) -> Self {
        match &self {
            Complex::Primitive(primitive_variable) => Self::Primitive(primitive_variable.clone()),
            Complex::Stack(stack_variables) => {
                let mut stack_result = Vec::new();
                for primitive_variable in stack_variables {
                    stack_result.push(primitive_variable.clone());
                }
                Self::Stack(stack_result)
            }
        }
    }
}
