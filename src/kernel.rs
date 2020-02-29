use super::{exec, structures, variables};
use rand::Rng;
use std::collections::HashMap;
use std::sync::{mpsc, Arc, RwLock};
use std::thread;
const PACKAGE_IDENTIFIER: usize = 32;
const THREAD_MAX_NUM: usize = 8;
const THREAD_IDENTIFIER_LENGTH: usize = 31;
#[derive(Debug)]
pub struct ExpressionPackage {
    identifier: [u8; PACKAGE_IDENTIFIER],
    expression: Arc<RwLock<structures::Expression>>,
    variable: Arc<RwLock<HashMap<String, Arc<RwLock<variables::Primitive>>>>>,
}
impl ExpressionPackage {
    pub fn from_function(
        function: &structures::Function,
        mut input: Vec<variables::Primitive>,
    ) -> Vec<Self> {
        let mut self_vector = Vec::new();
        //First, find input variables
        let mut variable_hashmap = HashMap::new();
        for variable_name in &function.input {
            println!("AAA{}", variable_name);
            variable_hashmap.insert(
                variable_name.to_string(),
                Arc::new(RwLock::new(input.remove(0))),
            );
        }
        let arc_variable_hashmap = Arc::new(RwLock::new(variable_hashmap));
        let mut rng = rand::thread_rng();
        for expression in &function.process {
            //Generate a random identifier
            let mut identifier = [0; PACKAGE_IDENTIFIER];
            for identifier_byte in identifier.iter_mut() {
                *identifier_byte = rng.gen();
            }
            self_vector.push(Self {
                identifier,
                expression: Arc::clone(expression),
                variable: Arc::clone(&arc_variable_hashmap),
            })
        }
        self_vector
    }
    pub fn from_expression(
        expression: std::sync::Arc<std::sync::RwLock<structures::Expression>>,
        input: HashMap<String, variables::Primitive>,
    ) -> Self {
        let mut rng = rand::thread_rng();
        let mut identifier = [0; PACKAGE_IDENTIFIER];
        for identifier_byte in identifier.iter_mut() {
            *identifier_byte = rng.gen();
        }
        let mut input_hashmap = HashMap::new();
        for (variable_name, variable_content) in input {
            input_hashmap.insert(variable_name, Arc::new(RwLock::new(variable_content)));
        }
        Self {
            identifier,
            expression,
            variable: Arc::new(RwLock::new(input_hashmap)),
        }
    }
    pub fn from_vec_expression(
        expression_vec: Vec<std::sync::Arc<std::sync::RwLock<structures::Expression>>>,
        input: HashMap<String, variables::Primitive>,
    ) -> Vec<Self> {
        let mut self_vector = Vec::new();
        let mut rng = rand::thread_rng();
        let mut input_hashmap = HashMap::new();
        for (variable_name, variable_content) in input {
            input_hashmap.insert(variable_name, Arc::new(RwLock::new(variable_content)));
        }
        let arc_variable_hashmap = Arc::new(RwLock::new(input_hashmap));
        for expression in expression_vec {
            let mut identifier = [0; PACKAGE_IDENTIFIER];
            for identifier_byte in identifier.iter_mut() {
                *identifier_byte = rng.gen();
            }
            self_vector.push(Self {
                identifier,
                expression,
                variable: Arc::clone(&arc_variable_hashmap),
            })
        }
        self_vector
    }
}
pub enum Message {
    Query([u8; PACKAGE_IDENTIFIER]),
    Package(ExpressionPackage),
    QueryResult(QueryResult),
    NewScript(String, structures::Script),
    Complete,
    CompleteWithPackage(Vec<ExpressionPackage>),
    PackageReceived,
    Exit,
}
enum QueryResult {
    Running,
    Completed,
}
enum ThreadState {
    Idle,
    Busy,
}
pub struct Kernel {
    sender: mpsc::Sender<Message>,
    receiver: mpsc::Receiver<Message>,
    handle: thread::JoinHandle<()>,
}
struct Thread {
    identifier: [u8; THREAD_IDENTIFIER_LENGTH],
    sender: mpsc::Sender<Message>,
    receiver: mpsc::Receiver<Message>,
    state: ThreadState,
    handle: thread::JoinHandle<()>,
}
impl Kernel {
    pub fn new() -> Self {
        let (sender, rx) = mpsc::channel();
        let (tx, receiver) = mpsc::channel();
        log::trace!("Kernel created");
        let handle = thread::Builder::new()
            .name("Kernel".to_string())
            .spawn(move || {
                //Reserved sender. currently there's no use for it.
                tx.send(Message::Complete).unwrap();
                let script_hashmap = Arc::new(RwLock::new(HashMap::new()));
                let mut thread_vec: Vec<Thread> = Vec::new();
                let mut assigned_job_identifier_hashmap: HashMap<[u8; THREAD_IDENTIFIER_LENGTH], [u8; PACKAGE_IDENTIFIER]> = HashMap::new();
                let mut completed_job_identifier_list: Vec<[u8; PACKAGE_IDENTIFIER]> = Vec::new();
                let mut assign_queue = Vec::new();
                loop {
                    let message_from_main_thread_warped = rx.try_recv();
                    if let Ok(message_from_main_thread) = message_from_main_thread_warped {
                        match message_from_main_thread {
                            Message::NewScript(script_name, script) => {
                                if script_hashmap
                                    .write()
                                    .unwrap()
                                    .insert(script_name, script)
                                    .is_some()
                                {
                                    log::warn!("Script name already registered. Rewritting...");
                                };
                            }
                            Message::Package(assign_package) => {
                                log::info!("Kernel received a package.");
                                assign_queue.push(assign_package);
                            }
                            Message::Exit => {
                                log::info!("Kernel stopping...");
                                for thread in thread_vec {
                                    thread.sender.send(Message::Exit).unwrap();
                                    thread.handle.join().unwrap();
                                }
                                log::warn!("Kernel stopped.");
                                return;
                            }
                            _ => log::warn!("Kernel received an unsupported message"),
                        };
                    } else if let Err(std::sync::mpsc::TryRecvError::Disconnected) =
                        message_from_main_thread_warped
                    {
                        log::error!("Kernel will stop ungracefully.");
                        return;
                    }
                    //Refresh states for threads
                    for thread in &mut thread_vec {
                        let message_from_thread_warped = thread.receiver.try_recv();
                        if let Ok(message_from_thread) = message_from_thread_warped {
                            match message_from_thread {
                                Message::Complete => {
                                    thread.state = ThreadState::Idle;
                                    log::trace!("Pushing job identifier from thread identifier {:?}", thread.identifier);
                                    completed_job_identifier_list.push(assigned_job_identifier_hashmap.remove(&thread.identifier).unwrap());
                                },
                                Message::CompleteWithPackage(package_vector) => {
                                    for assign_package in package_vector {
                                        assign_queue.push(assign_package);
                                    }
                                    thread.state = ThreadState::Idle;
                                    completed_job_identifier_list.push(assigned_job_identifier_hashmap.remove(&thread.identifier).unwrap());
                                }
                                Message::Package(package) => {
                                    assign_queue.push(package);
                                    thread.sender.send(Message::PackageReceived).unwrap();
                                }
                                Message::Query(package_id) => {
                                    let mut is_completed = Message::QueryResult(QueryResult::Running);
                                    for i in &completed_job_identifier_list {
                                        if &package_id == i {
                                            is_completed = Message::QueryResult(QueryResult::Completed);
                                        }
                                    }
                                    thread.sender.send(is_completed).unwrap();
                                }
                                _ => panic!("Kernel received an unsupported message"),
                            }
                        }
                    }
                    //Assign packages
                    let mut idle_thread_vec: Vec<&Thread> = Vec::new();
                    for thread in &thread_vec {
                        if let ThreadState::Idle = thread.state {
                            idle_thread_vec.push(thread);
                        }
                    }
                    let idle_thread_num = idle_thread_vec.len();
                    let mut create_thread_count = 0;
                    if idle_thread_num < assign_queue.len() {
                        log::info!("Idle thread not enough.");
                        if THREAD_MAX_NUM
                            > thread_vec.len() - idle_thread_vec.len() + assign_queue.len()
                        {
                            log::info!("Create thread will fix it.");
                            create_thread_count = assign_queue.len() - idle_thread_vec.len();
                        } else if THREAD_MAX_NUM > thread_vec.len() {
                            log::info!("Creating new threads...");
                            create_thread_count = THREAD_MAX_NUM - thread_vec.len();
                        }
                        for thread_num in 0..create_thread_count {
                            //Create thread
                            let thread_name = format!("Thread {}", thread_vec.len() + thread_num);
                            let (sender, rx) = mpsc::channel();
                            let (tx, receiver) = mpsc::channel();
                            let script_map = Arc::clone(&script_hashmap);
                            let thread_handle = thread::Builder::new()
                                .name(thread_name)
                                .spawn(move || {
                                    //Some
                                    let builtin_hashmap = exec::builtin_hashmap();
                                    loop {
                                        let message_wrapped = rx.try_recv();
                                        if let Ok(message) = message_wrapped {
                                            match message {
                                                Message::Package(package) => {
                                                    log::trace!("Execute package");
                                                    //Variable collection
                                                    let mut variable_vector = Vec::new();
                                                    let variable_hashmap =
                                                        package.variable.read().unwrap();
                                                    for variable_name in &package
                                                        .expression
                                                        .read()
                                                        .unwrap()
                                                        .variables
                                                    {
                                                        if variable_name.starts_with('"')
                                                            && variable_name.ends_with('"')
                                                        {
                                                            log::warn!(
                                                                "Variable {} seems to be a string.",
                                                                variable_name
                                                            );
                                                            variable_vector.push(Arc::new(
                                                                RwLock::new(
                                                                    variables::Primitive::Literal(
                                                                        variable_name[1
                                                                            ..variable_name.len()
                                                                                - 1]
                                                                            .to_string(),
                                                                    ),
                                                                ),
                                                            ));
                                                        } else {
                                                            variable_vector.push(Arc::clone(
                                                                variable_hashmap
                                                                    .get(variable_name)
                                                                    .unwrap(),
                                                            ));
                                                        }
                                                    }
                                                    let mut result = variables::Complex::Primitive(
                                                        variables::Primitive::Nil,
                                                    );
                                                    //Operation
                                                    match &package
                                                        .expression
                                                        .read()
                                                        .unwrap()
                                                        .operation
                                                    {
                                                        structures::Operation::Builtin(
                                                            builtin_command,
                                                        ) => {
                                                            let op_function = builtin_hashmap
                                                                .get(builtin_command)
                                                                .unwrap();
                                                            result = op_function(&variable_vector);
                                                        }
                                                        structures::Operation::External(
                                                            script_name,
                                                            function_name,
                                                        ) => {
                                                            //Transfowm variable vector
                                                            let mut new_variable_vec = Vec::new();
                                                            for variable in variable_vector {
                                                                new_variable_vec.push(
                                                                    variable
                                                                        .read()
                                                                        .unwrap()
                                                                        .clone(),
                                                                );
                                                            }
                                                            let script = script_map.read().unwrap();
                                                            let function = script
                                                                .get(script_name)
                                                                .unwrap()
                                                                .get(function_name)
                                                                .unwrap();
                                                            let expression_pack =
                                                                ExpressionPackage::from_function(
                                                                    function,
                                                                    new_variable_vec,
                                                                );
                                                            let result_variable = expression_pack[expression_pack.len()].variable.clone();
                                                            for single_expression in expression_pack
                                                            {
                                                                let identifier =
                                                                    single_expression.identifier;
                                                                tx.send(Message::Package(
                                                                    single_expression,
                                                                ))
                                                                .unwrap();
                                                                let mut sented = false;
                                                                while !sented {
                                                                    if let Ok(msg) = rx.try_recv() {
                                                                        if let Message::PackageReceived = msg {
                                                                            sented = true;
                                                                        }
                                                                    }
                                                                }
                                                                let mut completed = false;
                                                                while !completed {
                                                                    tx.send(Message::Query(identifier)).unwrap();
                                                                    let mut sented = false;
                                                                    while !sented {
                                                                        if let Ok(msg) = rx.try_recv() {
                                                                            if let Message::QueryResult(result) = msg {
                                                                                sented = true;
                                                                                if let QueryResult::Completed = result {
                                                                                    completed = true;
                                                                                }
                                                                            }
                                                                        }
                                                                    }
                                                                }
                                                            }
                                                            //Get result variable
                                                            if let Some(variable_name) = &function.output {
                                                                result = variables::Complex::Primitive(result_variable.read().unwrap().get(variable_name).unwrap().read().unwrap().clone());
                                                            };
                                                        }
                                                    };
                                                    //To
                                                    let mut thread_result = Message::Complete;
                                                    match &package.expression.read().unwrap().to {
                                                        structures::ExpressionTo::ToVar(variable_name) => {
                                                            package.variable.write().unwrap().insert(variable_name.to_string(), if let variables::Complex::Primitive(variable_content) = result {
                                                                Arc::new(RwLock::new(variable_content))
                                                            } else {
                                                                log::error!("Unexpected variable type");
                                                                return;
                                                            });
                                                        }
                                                        structures::ExpressionTo::ToBlock(block) => {
                                                            let mut blocks: Vec<ExpressionPackage> = Vec::new();
                                                            match result {
                                                                variables::Complex::Primitive(single_result) => {
                                                                    let mut variable_hmap = HashMap::new();
                                                                    for (variable_name, variable_content) in &*package.variable.read().unwrap() {
                                                                        variable_hmap.insert(variable_name.to_string(), variable_content.read().unwrap().clone());
                                                                    }
                                                                    variable_hmap.insert(String::from("this"), single_result);
                                                                    for pack in ExpressionPackage::from_vec_expression(block.to_vec(), variable_hmap) {
                                                                        blocks.push(pack);
                                                                    }
                                                                },
                                                                variables::Complex::Stack(stack_result) => {
                                                                    for single_result in stack_result {
                                                                        let mut variable_hmap = HashMap::new();
                                                                        for (variable_name, variable_content) in &*package.variable.read().unwrap() {
                                                                            variable_hmap.insert(variable_name.to_string(), variable_content.read().unwrap().clone());
                                                                        }
                                                                        variable_hmap.insert(String::from("this"), single_result);
                                                                        for pack in ExpressionPackage::from_vec_expression(block.to_vec(), variable_hmap) {
                                                                            blocks.push(pack);
                                                                        }
                                                                    }
                                                                },
                                                            }
                                                            thread_result = Message::CompleteWithPackage(blocks);
                                                        },
                                                        structures::ExpressionTo::Nil => {},
                                                    }
                                                    tx.send(thread_result).unwrap();
                                                }
                                                Message::Exit => {
                                                    log::warn!("Thread stopping");
                                                    return;
                                                }
                                                _ => {
                                                    log::warn!("Unexpected message");
                                                }
                                            }
                                        } else if let Err(mpsc::TryRecvError::Disconnected) =
                                            message_wrapped
                                        {
                                            log::error!("Thread will stop ungracefully.");
                                            return;
                                        }
                                    }
                                })
                                .unwrap();
                            let thread_identifier = {
                                let mut rng = rand::thread_rng();
                                let mut identifier = [0; THREAD_IDENTIFIER_LENGTH];
                                for identifier_byte in identifier.iter_mut() {
                                    *identifier_byte = rng.gen();
                                }
                                identifier
                            };
                            thread_vec.push(Thread {
                                identifier: thread_identifier,
                                sender,
                                receiver,
                                handle: thread_handle,
                                state: ThreadState::Idle,
                            });
                        }
                    }
                    let mut idle_thread_vec: Vec<&Thread> = Vec::new();
                    for thread in &thread_vec {
                        if let ThreadState::Idle = thread.state {
                            idle_thread_vec.push(thread);
                        }
                    }
                    for thread in idle_thread_vec {
                        if !assign_queue.is_empty() {
                            let package = assign_queue.remove(0);
                            log::trace!("Assigned thread identifier: {:?}, job identifier: {:?}", thread.identifier, package.identifier);
                            assigned_job_identifier_hashmap.insert(thread.identifier, package.identifier);
                            thread
                                .sender
                                .send(Message::Package(package))
                                .unwrap();
                        }
                    }
                }
            })
            .unwrap();
        Self {
            sender,
            receiver,
            handle,
        }
    }
    pub fn send_message(&self, msg: Message) {
        self.sender.send(msg);
    }
    pub fn stop(self) {
        self.sender.send(Message::Exit);
        self.handle.join().unwrap();
    }
}
