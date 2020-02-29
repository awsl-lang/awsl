mod exec;
mod functions;
mod kernel;
mod structures;
mod variables;
fn main() {}
#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;
    use std::time::Duration;
    #[test]
    fn exec() {
        pretty_env_logger::init();
        let function = structures::Function {
            input: Vec::new(),
            process: vec![std::sync::Arc::new(std::sync::RwLock::new(
                structures::Expression {
                    operation: structures::Operation::Builtin(String::from("print")),
                    variables: vec![String::from("\"Hello world\"")],
                    to: structures::ExpressionTo::Nil,
                },
            ))],
            output: None,
        };
        let threads = kernel::Kernel::new();
        let package = kernel::ExpressionPackage::from_function(&function, Vec::new());
        for i in package {
            threads.send_message(kernel::Message::Package(i));
        }
        thread::sleep(Duration::from_secs(5));
        threads.stop();
    }
}
