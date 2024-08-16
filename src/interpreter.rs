use crate::parser::Node;
use std::collections::HashMap;
use crate::error::*;

#[derive(Debug, PartialEq, Clone)]
pub enum Value {
    String(String),
    Number(i32),
    Bool(bool),
}

type Frame = HashMap<String, Value>;
type Arguments = Node;
type Statements = Node;

#[derive(Debug)]
pub struct Interpreter {
    // Function Table:
    // Key - Function name
    // Value - Vec<Node> arguments, statements
    functions: HashMap<String, (Arguments, Statements)>,
    // Stack:
    // Each element in the stack is a function stack frame.
    // Crate a new stack frame on function entry.
    // Pop stack frame on function return.
    // Key - Variable name
    // Value - Variable value
    stack: Vec<Frame>,
}

impl Interpreter {
    pub fn new() -> Interpreter {
        //changed this to make the Interpreter mutable
        let mut interpreter = Interpreter {
            functions: HashMap::new(),
            stack: Vec::new(),
        };
        // we initialize the stack with an empty global frame by pushing an empty HashMap onto it.
        interpreter.stack.push(HashMap::new());
        // now return the initialized interpreter.
        interpreter
    }

    pub fn exec(&mut self, node: &Node) -> Result<Value, AsaErrorKind> {
        match node {
            Node::Program { children } => {
                let mut result = Ok(Value::Bool(true)); // a default value
                for n in children {
                    match n {
                        | Node::FunctionDefine { .. }
                        | Node::Expression { .. }
                        | Node::VariableDefine { .. }
                        | Node::String { .. }
                        | Node::Number { .. }
                        | Node::Bool { .. } => {
                            result = self.exec(n);
                        }
                        _ => unreachable!(),
                    }
                }
                result
            }

            // Evaluates a mathematical expression based on the elements in the children argument. If the expression is valid, the code evaluates it and returns a new Value object with the resulting value. If the expression is not valid, the code returns an error message.
            Node::MathExpression { name, children } => {
                //*DONE
                //easy way to ensure we need to even do a math expression
                if children.len() != 2 {
                    return Err(
                        AsaErrorKind::Generic(
                            "MathExpression must have exactly two children".to_string()
                        )
                    );
                }
                // evaluate the left and right operands
                let left_value = self.exec(&children[0])?;
                let right_value = self.exec(&children[1])?;

                // perform the mathematical operation based on the operator
                match (left_value, right_value) {
                    (Value::Number(lhs), Value::Number(rhs)) => {
                        match name.as_slice() {
                            b"add" => Ok(Value::Number(lhs + rhs)),
                            b"sub" => Ok(Value::Number(lhs - rhs)),
                            b"mul" => Ok(Value::Number(lhs * rhs)),
                            b"div" => Ok(Value::Number(lhs / rhs)),
                            // add more operators as needed, these are enough for now
                            _ =>
                                Err(
                                    AsaErrorKind::Generic(
                                        "Unsupported operation in Math Expression".to_string()
                                    )
                                ),
                            //anything else would fall under a wrong operation error ^
                        }
                    }
                    _ =>
                        Err(
                            AsaErrorKind::Generic(
                                "MathExpression operands must be numbers".to_string()
                            )
                        ),
                    //if we got here, then the operands used for Math Expression were not number types ^
                }
            }
            // Defines a function that takes some arguments and executes a program based on those arguments. The code first checks if the function exists, and if it does, it creates a new scope in which to execute the function's statements (push a new Frame onto the interpreter stack). The code then executes each statement in the function's statements list and returns the result of the function's execution. You will have to correlate each passed value with the apprpriate variable in the called function. If the wrong number or an wrong type of variable is passed, return an error. On success, insert the return value of the function (if any) into the appropriate entry of the caller's stack.
            Node::FunctionCall { name, children } => {
                //*DONE
                // convert the function name from bytes to string
                let function_name = String::from_utf8_lossy(&name);

                // retrieve the function definition from the hashmap and clone the arguments and body
                // using .as_ref to use a reference to the function name rather than getting ownership of the String
                // I cloned because the trait `Borrow<Cow<'_, str>>` is not implemented for `String
                let (func_args, func_body) = match
                    self.functions.get(function_name.as_ref()).cloned()
                {
                    Some((args, body)) => (args, body),
                    None => {
                        return Err(AsaErrorKind::UndefinedFunction);
                    }
                };

                // we create a new frame to store local variables and arguments
                let mut new_frame = HashMap::new();

                // we match the function arguments with the provided children
                if let Node::FunctionArguments { children: params } = func_args {
                    if params.len() != children.len() {
                        return Err(
                            AsaErrorKind::Generic(
                                format!(
                                    "Expected a total of {} arguments, instead got only {} arguments",
                                    params.len(),
                                    children.len()
                                )
                            )
                        );
                    }

                    // iterate over the function parameters and passed arguments
                    for (param, arg) in params.iter().zip(children.iter()) {
                        if let Node::Identifier { value } = param {
                            // convert parameter name from bytes to string
                            let param_name = String::from_utf8_lossy(value).into_owned();
                            // execute the argument expression and store its value in the frame
                            let arg_value = self.exec(arg)?;
                            new_frame.insert(param_name, arg_value);
                        } else {
                            return Err(
                                AsaErrorKind::Generic(
                                    "The parameter in the function's definition is not an identifier".to_string()
                                )
                            );
                        }
                    }
                } else {
                    //if we never got to match function arguments with the provided children
                    return Err(
                        AsaErrorKind::Generic("Function arguments were not provided".to_string())
                    );
                }
                // push the new frame onto the stack
                self.stack.push(new_frame);
                // then execute the function body
                let result = self.exec(&func_body);
                // pop the frame from the stack
                self.stack.pop();

                // return the result of the function execution
                result
            }
            // Defines a new function based on the elements in the children argument. The name of the function is retrieved from the node struct, the arguments are the first child, and the statements that define the function are the second child. A new key-value pair is then inserted into the functions table of the interprer. If the function was successfully defined, the code returns a Value object with a boolean value of true, otherwise an error is returned.
            Node::FunctionDefine { name, children } => {
                //TODO: FIX THIS FUNCTION DEFINE?
                // extract the function arguments and function statements
                let function_arguments = match &children[0] {
                    Node::FunctionArguments { children } =>
                        Node::FunctionArguments { children: children.clone() },
                    _ => {
                        return Err(AsaErrorKind::Generic("Invalid function arguments".to_string()));
                    }
                };
                let function_statements = match &children[1] {
                    Node::FunctionStatements { children } =>
                        Node::FunctionStatements { children: children.clone() },
                    _ => {
                        return Err(
                            AsaErrorKind::Generic("Invalid function statements".to_string())
                        );
                    }
                };
                //convert the function name from a vector to a string
                let function_name = String::from_utf8_lossy(name).to_string();
                // clone the function name to so we can check if it exists in the functions table
                let cloned_function_name = function_name.clone();

                // insert the function into the functions map
                self.functions.insert(function_name, (function_arguments, function_statements));
                if self.functions.contains_key(&cloned_function_name) {
                    Ok(Value::Bool(true))
                } else {
                    return Err(AsaErrorKind::UndefinedFunction);
                }
            }
            // Calls the exec() method on the first element in the children argument, which recursively evaluates the AST of the program being executed and returns the resulting value or error message.
            Node::FunctionReturn { children } => {
                //*DONE
                //pretty simple, just call the exec() on the first element and then it'll recursively evaluate from thereon
                self.exec(&children[0])
            }
            // Retrieves the value of the identifier from the current frame on the stack. If the variable is defined in the current frame, the code returns its value. If the variable is not defined in the current frame, the code returns an error message.
            Node::Identifier { value } => {
                //*DONE
                // we are converting the byte vector `value` to a `String` so we can find it in the hashmap
                let identifier = String::from_utf8(value.clone()).map_err(|_|
                    AsaErrorKind::Generic("Wrong sequence present in the identifier.".to_string())
                )?;

                // we check the current frame on the stack for the identifier.
                // if there is a frame there, we retrieve the value associated with the identifier.
                if let Some(frame) = self.stack.last() {
                    if let Some(_val) = frame.get(&identifier) {
                        let new_val = frame.get(&identifier).unwrap();
                        println!(
                            "Identifier '{}' was found with the value: {:?}",
                            identifier,
                            new_val
                        );
                        Ok(new_val.clone())
                        // if the identifier is found in the frame, return its value. if it is not found, we return a `UndefinedFunction` error
                    } else {
                        println!("Identifier '{}' was not found.", identifier);
                        Err(AsaErrorKind::UndefinedFunction)
                    }
                } else {
                    // if there is no frame available, we give a `UndefinedFunction` error showing that no frame is there
                    println!("No available frame for the '{}' identifier.", identifier);
                    Err(AsaErrorKind::UndefinedFunction)
                }
            }
            // Checks the type of the first element in the children argument and deciding what to do based on that type. If the type is a VariableDefine or FunctionReturn node, the code runs the run method on that node and returns the result.
            Node::Statement { children } => {
                //*DONE
                //if the first child node matches either VariableDefine or FunctionReturn we execute it and return result
                if let Node::VariableDefine { .. } | Node::FunctionReturn { .. } = &children[0] {
                    self.exec(&children[0])
                } else {
                    return Err(AsaErrorKind::Generic("The expression is undefined".to_string()));
                }
            }
            // Defines a new variable by assigning a name and a value to it. The name is retrieved from the first element of the children argument, and the value is retrieved by running the run method on the second element of the children argument. The key-value pair is then inserted into the last frame on the stack field of the current runtime object.
            Node::VariableDefine { children } => {
                //*DONE
                // make sure that there are exactly two children: identifier and value.
                if children.len() != 2 {
                    return Err(
                        AsaErrorKind::Generic(
                            "VariableDefine must have exactly two children".to_string()
                        )
                    );
                }

                // extract the identifier and value nodes.
                let identifier_node = &children[0];
                let value_node = &children[1];

                // we need to make sure  that the first child is an identifier.
                let variable_name = match identifier_node {
                    Node::Identifier { value } =>
                        String::from_utf8(value.clone()).map_err(|_|
                            AsaErrorKind::Generic(
                                "There are invalid characters in the variable name.".to_string()
                            )
                        )?,
                    _ => {
                        return Err(
                            AsaErrorKind::Generic(
                                "The first child of VariableDefine must be an identifier.".to_string()
                            )
                        );
                    }
                };

                // we then evaluate the value node to get the variable's value.
                let variable_value = self.exec(value_node)?;

                // insert the variable into the current frame on the stack.
                if let Some(current_frame) = self.stack.last_mut() {
                    current_frame.insert(variable_name, variable_value.clone());
                    Ok(variable_value)
                } else {
                    Err(
                        AsaErrorKind::Generic(
                            "There is no active frame available to define variable.".to_string()
                        )
                    )
                }
            }
            // Evaluate the child node using the exec() method.
            Node::Expression { children } => { self.exec(&children[0]) } //*DONE
            Node::Number { value } => { Ok(Value::Number(*value)) } //*DONE
            Node::String { value } => { Ok(Value::String(value.clone())) } //*DONE
            Node::Bool { value } => { Ok(Value::Bool(*value)) } //*DONE
            // Return an error message.
            x => {
                //*DONE
                Err(AsaErrorKind::Generic(format!("No supported node type: {:?}", x)))
            }
        }
    }

    pub fn start_main(&mut self, arguments: Vec<Node>) -> Result<Value, AsaErrorKind> {
        // This node is equivalent to the following Asa program source code:
        // "main()"
        // It calls the main function with a FunctionArguments node as input.
        let start_main = Node::FunctionCall { name: "main".into(), children: arguments };
        // Call the main function by running this code through the interpreter.
        self.exec(&start_main)
    }
}
