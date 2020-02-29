use super::{functions, variables};
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
pub type BuiltInFunction = dyn Fn(&[Arc<RwLock<variables::Primitive>>]) -> variables::Complex;
pub fn builtin_hashmap() -> HashMap<String, Box<BuiltInFunction>> {
    let mut result: HashMap<String, Box<BuiltInFunction>> = HashMap::new();
    let built_in_commands_list = vec![
        BuiltInCmd {
            name: "print".to_string(),
            function: Box::new(functions::print),
        },
        BuiltInCmd {
            name: "stack123".to_string(),
            function: Box::new(functions::stack123),
        },
    ];
    for i in built_in_commands_list {
        result.insert(i.name, i.function);
    }
    result
}
struct BuiltInCmd {
    name: String,
    function: Box<BuiltInFunction>,
}
