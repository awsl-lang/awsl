use std::collections::HashMap;
use std::sync::{Arc, RwLock};
pub type Script = Arc<HashMap<String, Function>>;
/// Create a script from a `&str`
///
/// Please delete all whitespace before creating it
pub fn new_script(script_code: &str) -> Script {
    let mut current_offset = 0;
    let mut added_offset = 0;
    let mut current_beacket = 0;
    let mut definition = "";
    let mut block;
    let mut function: HashMap<String, Function> = HashMap::new();
    while script_code.len() > current_offset + added_offset {
        added_offset += 1;
        if &script_code[current_offset + added_offset - 1..current_offset + added_offset] == "{" {
            current_beacket += 1;
            if current_beacket == 1 {
                definition = &script_code[current_offset..added_offset + current_offset - 1];
                current_offset += added_offset;
                added_offset = 0;
            }
        } else if &script_code[current_offset + added_offset - 1..current_offset + added_offset]
            == "}"
        {
            current_beacket -= 1;
            if current_beacket == 0 {
                block = &script_code[current_offset..added_offset + current_offset - 1];
                let block_into_function = Function::from_str(
                    definition,
                    Expression::from_char(&block.chars().collect::<Vec<char>>()[..]),
                )
                .unwrap();
                function.insert(block_into_function.0.to_string(), block_into_function.1);
                current_offset += added_offset;
                added_offset = 0;
            }
        }
    }
    Arc::new(function)
}
#[derive(Debug)]
/// `Function` is a set of process instruction (`Expression`) and variables definition (both input and outbut)
pub struct Function {
    /// Function's input variable name
    pub input: Vec<String>,
    /// How to process input variable
    pub process: Vec<Arc<RwLock<Expression>>>,
    /// Function's output variable (if any)
    pub output: Option<String>,
}
impl Function {
    /// Create a function from string and `CommandBlock`
    ///
    /// `function definition` convention: NAME<RESULT>(VARIABLE)
    ///
    /// , where `RESULT` and `VARIABLE` *may* leave blanked, and
    ///
    /// - `NAME` should not contain `<`,
    /// - `RESULT` should not contain `>`
    /// - space between `RESULT` and `VARIABLE` should be exactly two character long ( For example, `<(`)
    fn from_str(
        function_definition: &str,
        function_content: Vec<Expression>,
    ) -> Result<(&str, Self), &'static str> {
        if function_definition.find('<').is_none() {
            return Err("Unable to find '<' in function definition");
        } else if function_definition[function_definition.find('<').unwrap()..]
            .find('>')
            .is_none()
        {
            return Err("Unable to find '>' after function name definition");
        }
        let name = &function_definition[..function_definition.find('<').unwrap()];
        let result = &function_definition[function_definition.find('<').unwrap() + 1
            ..function_definition[function_definition.find('<').unwrap()..]
                .find('>')
                .unwrap()
                + function_definition.find('<').unwrap()];
        let variables = &function_definition
            [function_definition.rfind('(').unwrap() + 1..function_definition.rfind(')').unwrap()];
        let mut process = Vec::new();
        for expression in function_content {
            process.push(Arc::new(RwLock::new(expression)));
        }
        Ok((
            name,
            Self {
                input: if variables.is_empty() {
                    Vec::new()
                } else {
                    variables
                        .split(',')
                        .map(|s| s.to_string())
                        .collect::<Vec<String>>()
                },
                process,
                output: if result.is_empty() {
                    None
                } else {
                    Some(result.to_string())
                },
            },
        ))
    }
}
#[derive(Debug)]
pub struct Expression {
    /// What is the expression
    pub operation: Operation,
    /// What variables should be sent to operation
    pub variables: Vec<String>,
    /// What's next / Should result stored
    pub to: ExpressionTo,
}
impl Expression {
    pub fn from_char(script_code: &[char]) -> Vec<Self> {
        // Grammar:
        // VARIABLE = COMMAND;
        // COMMAND => {ANOTHERBLOCK}
        // COMMAND;
        let mut self_vec: Vec<Self> = Vec::new();
        let mut command_detail = String::new();
        let mut to = ExpressionTo::Nil;
        let mut char_offset = 0;
        while script_code.len() > char_offset {
            while (script_code[char_offset] != '=') && (script_code[char_offset] != ';') {
                command_detail.push(script_code[char_offset]);
                char_offset += 1;
            }
            if script_code[char_offset] == '=' {
                if script_code[char_offset + 1] == '>' {
                    // COMMAND => {ANOTHERBOCK}
                    let mut code_offset = 2;
                    let mut branches_count = 1;
                    while branches_count != 0 {
                        if script_code[char_offset + 1 + code_offset] == '{' {
                            branches_count += 1;
                        } else if script_code[char_offset + 1 + code_offset] == '}' {
                            branches_count -= 1;
                        }
                        code_offset += 1;
                    }
                    let mut to_vec = Vec::new();
                    for i in
                        Self::from_char(&script_code[char_offset + 3..char_offset + code_offset])
                    {
                        to_vec.push(Arc::new(RwLock::new(i)));
                    }
                    to = ExpressionTo::ToBlock(to_vec);
                    char_offset += code_offset - 1;
                } else {
                    to = ExpressionTo::ToVar(command_detail);
                    command_detail = String::new();
                    char_offset -= 1;
                }
                char_offset += 1;
            } else {
                let op = &command_detail[..command_detail.find('(').unwrap()];
                let variable = &command_detail
                    [command_detail.find('(').unwrap() + 1..command_detail.rfind(')').unwrap()];
                self_vec.push(Self {
                    operation: Operation::from_str(&op).unwrap(),
                    to,
                    variables: if variable.is_empty() {
                        Vec::new()
                    } else {
                        variable
                            .split(',')
                            .map(|s| s.to_string())
                            .collect::<Vec<String>>()
                    },
                });
                command_detail = String::new();
                to = ExpressionTo::Nil;
            }
            char_offset += 1;
        }
        self_vec
    }
}
#[derive(Debug)]
pub enum Operation {
    Builtin(String),
    External(String, String),
}
impl Operation {
    //Parse a string into operation
    pub fn from_str(command_str: &str) -> Result<Self, &'static str> {
        // Command should be:
        // COMMAND[@PARENT]
        let mut parent: Option<String> = None;
        let command = if command_str.find('@').is_some() {
            //COMMAND@PARENT
            parent = Some(command_str[command_str.find('@').unwrap() + 1..].to_string());
            &command_str[..command_str.find('@').unwrap()]
        } else {
            //COMMAND
            command_str
        };
        if command_str.is_empty() {
            return Err("Empty command instruction");
        }
        match parent {
            Some(par) => Ok(Self::External(par, command.to_string())),
            None => Ok(Self::Builtin(command.to_string())),
        }
    }
}
#[derive(Debug)]
pub enum ExpressionTo {
    ToVar(String),
    ToBlock(Vec<Arc<RwLock<Expression>>>),
    Nil,
}
