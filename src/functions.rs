use super::variables;
use std::sync::{Arc, RwLock};
pub fn print(args: &[Arc<RwLock<variables::Primitive>>]) -> variables::Complex {
    println!(
        "{}",
        if let variables::Primitive::Literal(content) = &*args[0].read().unwrap() {
            &content
        } else {
            ""
        }
    );
    variables::Complex::Primitive(variables::Primitive::Nil)
}
pub fn stack123(_: &[Arc<RwLock<variables::Primitive>>]) -> variables::Complex {
    println!("you have called stack123.");
    variables::Complex::Stack(vec![
        variables::Primitive::Literal(String::from("1")),
        variables::Primitive::Literal(String::from("2")),
        variables::Primitive::Literal(String::from("3")),
    ])
}
