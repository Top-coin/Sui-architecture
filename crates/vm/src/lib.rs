use serde_json::Value;
use sui_core::{
    messages::ExecutionRequest,
    object::{ObjectData, ObjectID, Owner, SuiObject},
    transaction::TransactionKind,
};
use sui_storage::ObjectStore;

#[derive(Debug, Clone)]
pub struct ExecutionResult {
    pub gas_used: u64,
    pub touched_objects: Vec<SuiObject>,
    pub logs: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct MoveBytecode {
    pub instructions: Vec<MoveInstruction>,
}

#[derive(Debug, Clone)]
pub enum MoveInstruction {
    LoadConst(Value),
    CallFunction { module: String, function: String },
    Transfer { object_id: String, recipient: String },
    Return,
}

pub struct MoveVMExecutor {
    object_store: Option<Box<dyn ObjectStore>>,
}

impl MoveVMExecutor {
    pub fn new() -> Self {
        Self {
            object_store: None,
        }
    }

    pub fn with_object_store(store: Box<dyn ObjectStore>) -> Self {
        Self {
            object_store: Some(store),
        }
    }

    pub async fn execute(&self, request: &ExecutionRequest) -> ExecutionResult {
        match &request.tx.payload.kind {
            TransactionKind::Transfer { object, recipient } => {
                self.execute_transfer(object, recipient).await
            }
            TransactionKind::Call {
                package,
                module,
                function,
                arguments,
            } => {
                self.execute_move_call(package, module, function, arguments)
                    .await
            }
        }
    }

    async fn execute_transfer(&self, object: &ObjectID, recipient: &str) -> ExecutionResult {
        let mut touched_objects = Vec::new();
        let mut gas_used = 100;

        if let Some(store) = &self.object_store {
            if let Ok(Some(mut obj)) = store.get_object(&object.0).await {
                obj.owner = Owner::Address(recipient.to_string());
                obj.version += 1;
                gas_used += 400;
                if let Err(e) = store.put_object(obj.clone()).await {
                    return ExecutionResult {
                        gas_used,
                        touched_objects: vec![],
                        logs: vec![format!("Transfer failed: {}", e)],
                    };
                }
                touched_objects.push(obj);
            }
        } else {
            let new_obj = SuiObject::new(
                ObjectID::new(format!("{}-transferred", object.0)),
                Owner::Address(recipient.to_string()),
                ObjectData::Coin { balance: 1 },
            );
            touched_objects.push(new_obj);
            gas_used += 400;
        }

        ExecutionResult {
            gas_used,
            logs: vec![format!("Transfer executed: {} -> {}", object.0, recipient)],
            touched_objects,
        }
    }

    async fn execute_move_call(
        &self,
        _package: &ObjectID,
        module: &str,
        function: &str,
        arguments: &[Value],
    ) -> ExecutionResult {
        let bytecode = self.parse_move_call(module, function, arguments);
        let result = self.interpret_bytecode(&bytecode).await;

        ExecutionResult {
            gas_used: result.gas_used + 200,
            logs: vec![
                format!("Move call: {}::{}", module, function),
                format!("Arguments: {:?}", arguments),
                result.logs.join("; "),
            ],
            touched_objects: result.touched_objects,
        }
    }

    fn parse_move_call(&self, module: &str, function: &str, args: &[Value]) -> MoveBytecode {
        let mut instructions = Vec::new();

        for arg in args {
            instructions.push(MoveInstruction::LoadConst(arg.clone()));
        }

        instructions.push(MoveInstruction::CallFunction {
            module: module.to_string(),
            function: function.to_string(),
        });

        instructions.push(MoveInstruction::Return);

        MoveBytecode { instructions }
    }

    async fn interpret_bytecode(&self, bytecode: &MoveBytecode) -> ExecutionResult {
        let mut stack: Vec<Value> = Vec::new();
        let mut gas_used = 0;
        let mut logs = Vec::new();
        let mut touched_objects = Vec::new();

        for instruction in &bytecode.instructions {
            gas_used += 50;

            match instruction {
                MoveInstruction::LoadConst(value) => {
                    stack.push(value.clone());
                    logs.push(format!("Loaded constant: {:?}", value));
                }
                MoveInstruction::CallFunction { module, function } => {
                    let result = self.execute_function(module, function, &stack).await;
                    logs.extend(result.logs);
                    touched_objects.extend(result.touched_objects);
                    gas_used += result.gas_used;
                }
                MoveInstruction::Transfer { object_id, recipient } => {
                    if let Some(store) = &self.object_store {
                        if let Ok(Some(mut obj)) = store.get_object(object_id).await {
                            obj.owner = Owner::Address(recipient.clone());
                            obj.version += 1;
                            if store.put_object(obj.clone()).await.is_ok() {
                                touched_objects.push(obj);
                                logs.push(format!("Transferred {} to {}", object_id, recipient));
                            }
                        }
                    }
                }
                MoveInstruction::Return => {
                    logs.push("Function returned".to_string());
                    break;
                }
            }
        }

        ExecutionResult {
            gas_used,
            touched_objects,
            logs,
        }
    }

    async fn execute_function(&self, module: &str, function: &str, _stack: &[Value]) -> ExecutionResult {
        match (module, function) {
            ("coin", "transfer") => ExecutionResult {
                gas_used: 300,
                touched_objects: vec![],
                logs: vec!["Coin transfer executed".to_string()],
            },
            ("coin", "mint") => ExecutionResult {
                gas_used: 200,
                touched_objects: vec![SuiObject::new(
                    ObjectID::random(),
                    Owner::Address("mint-address".to_string()),
                    ObjectData::Coin { balance: 1000 },
                )],
                logs: vec!["Coin minted".to_string()],
            },
            _ => ExecutionResult {
                gas_used: 150,
                touched_objects: vec![],
                logs: vec![format!("Executed {module}::{function}")],
            },
        }
    }
}

impl Default for MoveVMExecutor {
    fn default() -> Self {
        Self::new()
    }
}
