mod exec;
mod functions;
mod kernel;
mod structures;
mod variables;
fn main() {
    println!("AAA");
}
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
            threads.send_package(i);
        }
        thread::sleep(Duration::from_secs(5));
        threads.stop();
    }
    #[test]
    fn new_script() {
        pretty_env_logger::init();
        let threads = kernel::Kernel::new();
        let script = structures::new_script(
            "main<>(){print(\"abc\");stack123()=>{print(\"fgh\");};print(\"cda\");}",
        );
        let package =
            kernel::ExpressionPackage::from_function(script.get("main").unwrap(), Vec::new());
        log::trace!("{:?}", package);
        for i in package {
            threads.send_package(i);
        }
        thread::sleep(Duration::from_secs(5));
        threads.stop();
    }
}
