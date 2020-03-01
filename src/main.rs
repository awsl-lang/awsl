mod exec;
mod functions;
mod kernel;
mod structures;
mod variables;
fn main() {
    let cli_config = clap::App::new("awsl")
        .version("0.9 Alpha")
        .author("moelife-coder <61054382+moelife-coder@users.noreply.github.com>")
        .about("A awsl-lang runtime executer")
        .arg(
            clap::Arg::with_name("run")
                .takes_value(true)
                .help("Name of the script that should be executed")
                .short("r"),
        )
        .arg(
            clap::Arg::with_name("load")
                .short("l")
                .takes_value(true)
                .multiple(true)
                .help("Name of the script that should be loaded"),
        )
        .arg(
            clap::Arg::with_name("function_name")
                .short("n")
                .takes_value(true)
                .multiple(false)
                .help("Name of the function that should be executed"),
        )
        .arg(
            clap::Arg::with_name("arguments")
                .short("a")
                .takes_value(true)
                .multiple(true)
                .help("Arguments for the function"),
        )
        .get_matches();
    let script_file = cli_config.value_of("run").unwrap();
    let loaded_script_file = match cli_config.values_of("load") {
        None => Vec::new(),
        Some(i) => i.collect(),
    };
    let function_name = match cli_config.value_of("function_name") {
        None => {
            log::warn!("No function name specified. Treated as \"main\"");
            "main"
        }
        Some(i) => i,
    };
    let threads = kernel::Kernel::new();
    if !loaded_script_file.is_empty() {
        for i in loaded_script_file {
            let mut script = std::fs::read_to_string(i).unwrap();
            script.retain(|c| !c.is_whitespace());
            let script_structure = structures::new_script(&script);
            threads.send_message(kernel::Message::NewScript(i.to_string(), script_structure));
        }
    }
    let vars = match cli_config.values_of("arguments") {
        Some(i) => {
            log::trace!("Accepting values from command line interface - {:?}", i);
            i.collect()
        }
        None => Vec::new(),
    };
    let mut main_script = std::fs::read_to_string(script_file).unwrap();
    main_script.retain(|c| !c.is_whitespace());
    let main_script_structure = structures::new_script(&main_script);
    let mut variables_primitive = Vec::new();
    for i in vars {
        variables_primitive.push(variables::Primitive::Literal(i.to_string()));
    }
    let packages = kernel::ExpressionPackage::from_function(
        main_script_structure.get(function_name).unwrap(),
        variables_primitive,
    );
    for i in packages {
        threads.send_package(i);
    }
    threads.grace_stop();
}
#[cfg(test)]
mod tests {
    use super::*;
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
        threads.grace_stop();
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
        for i in package {
            threads.send_package(i);
        }
        threads.grace_stop();
    }
}
