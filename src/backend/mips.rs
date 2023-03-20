use std::fs::OpenOptions;
use std::io::prelude::*;
use std::io::{self, BufRead};
use std::error::Error;
use std::collections::HashMap;
use std::sync::atomic::{AtomicUsize, Ordering};

use crate::frontend::intermediate_gen::{IntermediateInstr, Argument};
use crate::frontend::semantics::{SymbolTable, SymbolTableRow};
use crate::frontend::ast::Type;


#[allow(dead_code)]
#[derive(Debug)]
struct VariableTableRow {
    identifier: usize,
    offset:usize
}


/**
 * Calculates the size required for the function frame. Used when invoking a function.
 */
fn get_frame_size(function_id:&str, symbol_table:&SymbolTable) -> u64 {
    let mut frame_size = 0;
    for symbol in &symbol_table.rows {
        match symbol {
            SymbolTableRow::Variable {primitive_type, function_id: fid, ..} => {
                if fid != function_id {
                    continue;
                }

                // add the size in bytes of the datatype to the frame size
                match primitive_type {
                    Type::Float => frame_size += 4,
                    Type::Double => frame_size += 8,
                    Type::Char => frame_size += 4,
                    Type::Byte => frame_size += 4,
                    Type::Integer => frame_size += 4,
                    Type::Long => frame_size += 8,
                    Type::Boolean => frame_size += 4,
                    Type::String => todo!(),
                    Type::Void => panic!("Type void cannot be stored on the stack")
                }
            },

            _ => {}
        }
    }

    frame_size
}


fn get_target_code(architecture:&str, instr:&str, op_type:Option<&str>, arguments:Vec<String>) -> String {
    let mut file = OpenOptions::new().read(true).open("src/backend/target_code.json").expect("Could not read target_code.json");
    let mut json = String::new();
    file.read_to_string(&mut json).unwrap();
    let json:serde_json::Value = serde_json::from_str(&json).expect("Could not parse JSON from target_code.json");

    let target_code:Vec<String> = match op_type {
        Some(op_type) => {
            serde_json::to_string(&json[architecture][instr][op_type]).unwrap().split("\",").map(|item| {
                item.replace("[", "").replace("]", "").replace("\"", "").trim().to_string().replace("\\t", "\t")
            }).collect()
        },

        None => {
            serde_json::to_string(&json[architecture][instr]).unwrap().split("\",").map(|item| {
                item.replace("[", "").replace("]", "").replace("\"", "").trim().to_string().replace("\\t", "\t")
            }).collect()
        }
    };
    let mut target_code = target_code.join("\n");

    let required_arg_count = target_code.matches("{}").count();
    if required_arg_count != arguments.len() {
        panic!("Instruction {} takes {} arguments, but {} were provided", instr, required_arg_count, arguments.len());
    }

    for arg in arguments {
        target_code = target_code.replacen("{}", &arg, 1);
    }

    target_code += "\n";
    target_code
}


#[allow(dead_code)]
fn add_library(library_name:&str) -> Vec<String> {
    let file = OpenOptions::new().read(true).open(format!("src/backend/{}.asm", library_name)).unwrap();
    let lines:Vec<String> = io::BufReader::new(file).lines().map(|l| l.unwrap()).collect();
    lines
}


/**
 * Derives the next label from a static variable. Label is in the format `_t_<hex>` where `<hex>` is a
 * hexadecimal number uniquely identifying the label. 
 * 
 * For example, we start at "_t_1", then "_t_2", and the 32nd is "_t_20".
 */
fn get_next_label() -> String {
    static NEXT_LABEL:AtomicUsize = AtomicUsize::new(1);
    let next_label = NEXT_LABEL.fetch_add(1, Ordering::Relaxed);
    format!("_t_{:x}", next_label)
}


/**
 * Generates the final MIPS assembly code that can then be compiled to native binary using a separate tool.
 */
pub fn generate_mips(intermediate_code:Vec<IntermediateInstr>, filename:&str, symbol_table:&SymbolTable) -> Result<(), Box<dyn Error>> {
    let mut file = OpenOptions::new().write(true).truncate(true).create(true).open(filename)?;

    let mut text_section:Vec<String> = vec![String::from(".data:")];
    let mut mips_instrs:Vec<String> = vec![String::from("\n\n.text:")];

    let mut stack_id_offset_map: HashMap<usize, usize> = HashMap::new();
    let mut current_var_offset:usize = 0;
    let mut stack_types:Vec<Type> = vec![];

    mips_instrs.push("\tj main # start program execution\n\n".to_owned());
    // mips_instrs.append(&mut add_library("math64_mips"));

    for instr in intermediate_code {
        match instr {
            IntermediateInstr::FuncStart(name) => {
                let frame_size = get_frame_size(&name, symbol_table);
                mips_instrs.push(get_target_code("mips", "start_func", None, vec![name, frame_size.to_string()]));
            },

            IntermediateInstr::FuncEnd(name) => {
                if name == "main" {
                    mips_instrs.push(get_target_code("mips", "end_main", None, vec![]));
                } else {
                    mips_instrs.push(get_target_code("mips", "end_func", None, vec![name]));
                }
            },

            // Push an integer to the stack, use registers $t0 and $t2 to allow for future implementation of long datatype
            IntermediateInstr::Push(_, var) => {
                match var {
                    Argument::Integer(value) => {
                        stack_types.push(Type::Integer);
                        mips_instrs.push(get_target_code("mips", "push", Some("int"), vec![value.to_string()]));
                    },

                    Argument::Long(value) => {
                        stack_types.push(Type::Long);
                        let upper_bits:u64 = (value as u64 & 0xFFFF_FFFF_0000_0000) >> 32;
                        let lower_bits:u64 = value as u64 & 0xFFFF_FFFF;
                        mips_instrs.push(get_target_code("mips", "push", Some("long"), vec![
                            upper_bits.to_string(),
                            lower_bits.to_string()
                        ]));
                    },

                    Argument::Byte(value) => {
                        stack_types.push(Type::Byte);
                        mips_instrs.push(get_target_code("mips", "push", Some("byte"), vec![value.to_string()]));
                    },

                    Argument::Float(value) => {
                        stack_types.push(Type::Float);

                        let label = get_next_label();
                        text_section.push(format!("\t{}: .float {}", label, value));
                        mips_instrs.push(get_target_code("mips", "push", Some("float"), vec![label]));
                    },

                    Argument::Double(value) => {
                        stack_types.push(Type::Double);

                        let label = get_next_label();
                        text_section.push(format!("\t{}: .double {}", label, value));
                        mips_instrs.push(get_target_code("mips", "push", Some("double"), vec![label]));
                    },

                    Argument::Char(value) => {
                        stack_types.push(Type::Char);

                        let label = get_next_label();
                        text_section.push(format!("\t{}: .byte '{}'", label, value));
                        mips_instrs.push(get_target_code("mips", "push", Some("char"), vec![label]));
                    },

                    Argument::Boolean(value) => {
                        stack_types.push(Type::Boolean);
                        match value {
                            true => mips_instrs.push(get_target_code("mips", "push", Some("bool"), vec![String::from("1")])),
                            false => mips_instrs.push(get_target_code("mips", "push", Some("bool"), vec![String::from("0")])),
                        }
                    },

                    _ => todo!()
                }
            },

            IntermediateInstr::Store(var_type, id) => {
                match var_type {
                    Type::Integer => {
                        // if the key does not exist, add a new key to represent a new local variable
                        if !stack_id_offset_map.contains_key(&id) {
                            current_var_offset += 4;
                            stack_id_offset_map.insert(id, current_var_offset);
                        }

                        mips_instrs.push(get_target_code("mips", "store", Some("int"), vec![stack_id_offset_map.get(&id).unwrap().to_string()]));
                        stack_types.pop();
                    },

                    Type::Long => {
                        // if the key does not exist, add a new key to represent a new local variable
                        if !stack_id_offset_map.contains_key(&id) {
                            current_var_offset += 8;
                            stack_id_offset_map.insert(id, current_var_offset);
                        }

                        mips_instrs.push(get_target_code("mips", "store", Some("long"), vec![
                            stack_id_offset_map.get(&id).unwrap().to_string(),
                            (stack_id_offset_map.get(&id).unwrap() - 4).to_string()
                        ]));

                        stack_types.pop();
                    },

                    Type::Byte => {
                        // if the key does not exist, add a new key to represent a new local variable
                        if !stack_id_offset_map.contains_key(&id) {
                            current_var_offset += 4;
                            stack_id_offset_map.insert(id, current_var_offset);
                        }

                        mips_instrs.push(get_target_code("mips", "store", Some("byte"), vec![
                            stack_id_offset_map.get(&id).unwrap().to_string()
                        ]));

                        stack_types.pop();
                    },

                    Type::Float => {
                        // if the key does not exist, add a new key to represent a new local variable
                        if !stack_id_offset_map.contains_key(&id) {
                            current_var_offset += 4;
                            stack_id_offset_map.insert(id, current_var_offset);
                        }

                        mips_instrs.push(get_target_code("mips", "store", Some("float"), vec![
                            stack_id_offset_map.get(&id).unwrap().to_string()
                        ]));

                        stack_types.pop();
                    },

                    Type::Double => {
                        // if the key does not exist, add a new key to represent a new local variable
                        if !stack_id_offset_map.contains_key(&id) {
                            current_var_offset += 8;
                            stack_id_offset_map.insert(id, current_var_offset);
                        }

                        mips_instrs.push(get_target_code("mips", "store", Some("double"), vec![
                            stack_id_offset_map.get(&id).unwrap().to_string(),
                            (stack_id_offset_map.get(&id).unwrap() - 4).to_string()
                        ]));

                        stack_types.pop();
                    },

                    Type::Char => {
                        // if the key does not exist, add a new key to represent a new local variable
                        if !stack_id_offset_map.contains_key(&id) {
                            current_var_offset += 4;
                            stack_id_offset_map.insert(id, current_var_offset);
                        }

                        mips_instrs.push(get_target_code("mips", "store", Some("char"), vec![
                            stack_id_offset_map.get(&id).unwrap().to_string()
                        ]));

                        stack_types.pop();
                    },

                    Type::Boolean => {
                        // if the key does not exist, add a new key to represent a new local variable
                        if !stack_id_offset_map.contains_key(&id) {
                            current_var_offset += 4;
                            stack_id_offset_map.insert(id, current_var_offset);
                        }
    
                        mips_instrs.push(get_target_code("mips", "store", Some("bool"), vec![
                            stack_id_offset_map.get(&id).unwrap().to_string()
                        ]));
    
                        stack_types.pop();
                    },

                    Type::Void => panic!("Cannot store type Void"),
                    _ => todo!()
                }
            },

            IntermediateInstr::Load(var_type, id) => {
                match var_type {
                    Type::Integer => {
                        stack_types.push(Type::Integer);

                        let offset = stack_id_offset_map.get(&id).unwrap_or(&0);
                        mips_instrs.push(get_target_code("mips", "load", Some("int"), vec![offset.to_string()]));
                    },

                    Type::Long => {
                        stack_types.push(Type::Long);

                        let offset = stack_id_offset_map.get(&id).unwrap();
                        mips_instrs.push(get_target_code("mips", "load", Some("long"), vec![
                            offset.to_string(), (offset - 4).to_string()
                        ]));
                    },

                    Type::Byte => {
                        stack_types.push(Type::Byte);

                        let offset = stack_id_offset_map.get(&id).unwrap_or(&0);
                        mips_instrs.push(get_target_code("mips", "load", Some("byte"), vec![offset.to_string()]));
                    },

                    Type::Float => {
                        stack_types.push(Type::Float);

                        let offset = stack_id_offset_map.get(&id).unwrap_or(&0);
                        mips_instrs.push(get_target_code("mips", "load", Some("float"), vec![offset.to_string()]));
                    },

                    Type::Double => {
                        stack_types.push(Type::Double);

                        let offset = stack_id_offset_map.get(&id).unwrap_or(&0);
                        mips_instrs.push(get_target_code("mips", "load", Some("double"), vec![
                            offset.to_string(), (offset - 4).to_string()
                        ]));
                    },

                    Type::Char => {
                        stack_types.push(Type::Char);

                        let offset = stack_id_offset_map.get(&id).unwrap_or(&0);
                        mips_instrs.push(get_target_code("mips", "load", Some("char"), vec![offset.to_string()]));
                    },

                    Type::Boolean => {
                        stack_types.push(Type::Boolean);

                        let offset = stack_id_offset_map.get(&id).unwrap_or(&0);
                        mips_instrs.push(get_target_code("mips", "load", Some("bool"), vec![offset.to_string()]));
                    },

                    Type::Void => panic!("Cannot load type Void"),
                    _ => todo!()
                }
            },

            IntermediateInstr::Return(return_type) => {
                match return_type {
                    Type::Integer => mips_instrs.push(get_target_code("mips", "return", Some("int"), vec![])),
                    Type::Long => mips_instrs.push(get_target_code("mips", "return", Some("long"), vec![])),
                    Type::Byte => mips_instrs.push(get_target_code("mips", "return", Some("byte"), vec![])),
                    Type::Float => mips_instrs.push(get_target_code("mips", "return", Some("float"), vec![])),
                    Type::Double => mips_instrs.push(get_target_code("mips", "return", Some("double"), vec![])),
                    Type::Char => mips_instrs.push(get_target_code("mips", "return", Some("char"), vec![])),
                    Type::Boolean => mips_instrs.push(get_target_code("mips", "return", Some("bool"), vec![])),
                    Type::Void => panic!("Cannot return type Void"),
                    Type::String => todo!()
                }

                stack_types.pop();
            },

            IntermediateInstr::Add => {
                let op_type = stack_types.pop().unwrap();
                match op_type {
                    Type::Integer => mips_instrs.push(get_target_code("mips", "add", Some("int"), vec![])),
                    Type::Long => mips_instrs.push(get_target_code("mips", "add", Some("long"), vec![])),
                    Type::Byte => mips_instrs.push(get_target_code("mips", "add", Some("byte"), vec![])),
                    Type::Float => mips_instrs.push(get_target_code("mips", "add", Some("float"), vec![])),
                    Type::Double => mips_instrs.push(get_target_code("mips", "add", Some("double"), vec![])),
                    Type::Char | Type::Void => panic!("Cannot apply + operator to type {:?}", op_type),
                    _ => todo!()
                }
            },

            IntermediateInstr::Sub => {
                let op_type = stack_types.pop().unwrap();
                match op_type {
                    Type::Integer => mips_instrs.push(get_target_code("mips", "sub", Some("int"), vec![])),
                    Type::Long => mips_instrs.push(get_target_code("mips", "sub", Some("long"), vec![])),
                    Type::Byte => mips_instrs.push(get_target_code("mips", "sub", Some("byte"), vec![])),
                    Type::Float => mips_instrs.push(get_target_code("mips", "sub", Some("float"), vec![])),
                    Type::Double => mips_instrs.push(get_target_code("mips", "sub", Some("double"), vec![])),
                    Type::Char | Type::Void => panic!("Cannot apply - operator to type {:?}", op_type),
                    _ => todo!()
                }
            },
            
            IntermediateInstr::Mult => {
                let op_type = stack_types.pop().unwrap();
                match op_type {
                    Type::Integer => mips_instrs.push(get_target_code("mips", "mult", Some("int"), vec![])),
                    Type::Long => mips_instrs.push(get_target_code("mips", "mult", Some("long"), vec![])),
                    Type::Byte => mips_instrs.push(get_target_code("mips", "mult", Some("byte"), vec![])),
                    Type::Float => mips_instrs.push(get_target_code("mips", "mult", Some("float"), vec![])),
                    Type::Double => mips_instrs.push(get_target_code("mips", "mult", Some("double"), vec![])),
                    Type::Char | Type::Void => panic!("Cannot apply * operator to type {:?}", op_type),
                    _ => todo!()
                }
            },

            IntermediateInstr::Div => {
                let op_type = stack_types.pop().unwrap();
                match op_type {
                    Type::Integer => mips_instrs.push(get_target_code("mips", "div", Some("int"), vec![])),
                    Type::Long => mips_instrs.push(get_target_code("mips", "div", Some("long"), vec![])),
                    Type::Byte => mips_instrs.push(get_target_code("mips", "div", Some("byte"), vec![])),
                    Type::Float => mips_instrs.push(get_target_code("mips", "div", Some("float"), vec![])),
                    Type::Double => mips_instrs.push(get_target_code("mips", "div", Some("double"), vec![])),
                    Type::Char | Type::Void => panic!("Cannot apply / operator to type {:?}", op_type),
                    _ => todo!()
                }
            },

            IntermediateInstr::BitwiseAnd => {
                let op_type = stack_types.pop().unwrap();
                match op_type {
                    Type::Integer => mips_instrs.push(get_target_code("mips", "bitwise_and", Some("int"), vec![])),
                    Type::Long => mips_instrs.push(get_target_code("mips", "bitwise_and", Some("long"), vec![])),
                    Type::Byte => mips_instrs.push(get_target_code("mips", "bitwise_and", Some("byte"), vec![])),
                    Type::Float | Type::Double | Type::Char | Type::Void => panic!("Cannot apply & operator to type {:?}", op_type),
                    _ => todo!()
                }
            },

            IntermediateInstr::BitwiseOr => {
                let op_type = stack_types.pop().unwrap();
                match op_type {
                    Type::Integer => mips_instrs.push(get_target_code("mips", "bitwise_or", Some("int"), vec![])),
                    Type::Long => mips_instrs.push(get_target_code("mips", "bitwise_or", Some("long"), vec![])),
                    Type::Byte => mips_instrs.push(get_target_code("mips", "bitwise_or", Some("byte"), vec![])),
                    Type::Float | Type::Double | Type::Char | Type::Void => panic!("Cannot apply | operator to type {:?}", op_type),
                    _ => todo!()
                }
            },

            IntermediateInstr::BitwiseXor => {
                let op_type = stack_types.pop().unwrap();
                match op_type {
                    Type::Integer => mips_instrs.push(get_target_code("mips", "bitwise_xor", Some("int"), vec![])),
                    Type::Long => mips_instrs.push(get_target_code("mips", "bitwise_xor", Some("long"), vec![])),
                    Type::Byte => mips_instrs.push(get_target_code("mips", "bitwise_xor", Some("byte"), vec![])),
                    Type::Float | Type::Double | Type::Char | Type::Void => panic!("Cannot apply ^ operator to type {:?}", op_type),
                    _ => todo!()
                }
            },

            IntermediateInstr::NumNeg => {
                let op_type = stack_types.last().unwrap();
                match op_type {
                    Type::Integer => mips_instrs.push(get_target_code("mips", "numerical_neg", Some("int"), vec![])),
                    Type::Long => mips_instrs.push(get_target_code("mips", "numerical_neg", Some("long"), vec![])),
                    Type::Float => mips_instrs.push(get_target_code("mips", "numerical_neg", Some("float"), vec![])),
                    Type::Double => mips_instrs.push(get_target_code("mips", "numerical_neg", Some("double"), vec![])),
                    Type::Byte | Type::Char | Type::Void => panic!("Numerical negation cannot be applied to type {:?}", op_type),
                    _ => todo!()
                }
            },

            IntermediateInstr::Complement => {
                let op_type = stack_types.last().unwrap();
                match op_type {
                    Type::Integer => mips_instrs.push(get_target_code("mips", "complement", Some("int"), vec![])),
                    Type::Long => mips_instrs.push(get_target_code("mips", "complement", Some("long"), vec![])),
                    Type::Byte => mips_instrs.push(get_target_code("mips", "complement", Some("byte"), vec![])),
                    Type::Float | Type::Double | Type::Char | Type::Void => panic!("Cannot apply ~ operator to type {:?}", op_type),
                    _ => todo!()
                }
            },

            IntermediateInstr::LogicNeg => {
                let op_type = stack_types.last().unwrap();
                match op_type {
                    Type::Integer => mips_instrs.push(get_target_code("mips", "logical_neg", Some("int"), vec![])),
                    Type::Long => mips_instrs.push(get_target_code("mips", "logical_neg", Some("long"), vec![])),
                    Type::Byte => mips_instrs.push(get_target_code("mips", "logical_neg", Some("byte"), vec![])),
                    Type::Float => mips_instrs.push(get_target_code("mips", "logical_neg", Some("float"), vec![])),
                    Type::Double => mips_instrs.push(get_target_code("mips", "logical_neg", Some("double"), vec![])),
                    Type::Boolean => mips_instrs.push(get_target_code("mips", "logical_neg", Some("bool"), vec![])),
                    Type::Char | Type::Void => panic!("Logical negation cannot be applied to type {:?}", op_type),
                    _ => todo!()
                }
            },

            IntermediateInstr::LeftShiftLogical => {
                let op_type = stack_types.pop().unwrap();
                match op_type {
                    Type::Integer => mips_instrs.push(get_target_code("mips", "sll", Some("int"), vec![])),
                    Type::Long => mips_instrs.push(get_target_code("mips", "sll", Some("long"), vec![])),
                    Type::Byte => mips_instrs.push(get_target_code("mips", "sll", Some("byte"), vec![])),
                    Type::Float | Type::Double | Type::Char | Type::Void => panic!("Cannot apply >> operator to type {:?}", op_type),
                    _ => todo!()
                }
            },

            IntermediateInstr::RightShiftLogical => {
                let op_type = stack_types.pop().unwrap();
                match op_type {
                    Type::Integer => mips_instrs.push(get_target_code("mips", "srl", Some("int"), vec![])),
                    Type::Long => mips_instrs.push(get_target_code("mips", "srl", Some("long"), vec![])),
                    Type::Byte => mips_instrs.push(get_target_code("mips", "srl", Some("byte"), vec![])),
                    Type::Float | Type::Double | Type::Char | Type::Void => panic!("Cannot apply << operator to type {:?}", op_type),
                    _ => todo!()
                }
            },

            IntermediateInstr::RightShiftArithmetic => {
                let op_type = stack_types.pop().unwrap();
                match op_type {
                    Type::Integer => mips_instrs.push(get_target_code("mips", "sra", Some("int"), vec![])),
                    Type::Long => mips_instrs.push(get_target_code("mips", "sra", Some("long"), vec![])),
                    Type::Byte => mips_instrs.push(get_target_code("mips", "sra", Some("byte"), vec![])),
                    Type::Float | Type::Double | Type::Char | Type::Void => panic!("Cannot apply >>> operator to type {:?}", op_type),
                    _ => todo!()
                }
            },
         
            IntermediateInstr::Equal => {
                let op_type = stack_types.pop().unwrap();
                stack_types.pop();

                match op_type {
                    Type::Integer => mips_instrs.push(get_target_code("mips", "test_equal", Some("int"), vec![])),
                    Type::Long => mips_instrs.push(get_target_code("mips", "test_equal", Some("long"), vec![])),
                    Type::Byte => mips_instrs.push(get_target_code("mips", "test_equal", Some("byte"), vec![])),
                    Type::Float => mips_instrs.push(get_target_code("mips", "test_equal", Some("float"), vec![])),
                    Type::Double => mips_instrs.push(get_target_code("mips", "test_equal", Some("double"), vec![])),
                    Type::Char => mips_instrs.push(get_target_code("mips", "test_equal", Some("char"), vec![])),
                    Type::Void => panic!("Cannot apply == operator to type {:?}", op_type),
                    _ => todo!()
                }

                stack_types.push(Type::Byte);
            },

            IntermediateInstr::NotEqual => {
                let op_type = stack_types.pop().unwrap();
                stack_types.pop();

                match op_type {
                    Type::Integer => mips_instrs.push(get_target_code("mips", "test_unequal", Some("int"), vec![])),
                    Type::Long => mips_instrs.push(get_target_code("mips", "test_unequal", Some("long"), vec![])),
                    Type::Byte => mips_instrs.push(get_target_code("mips", "test_unequal", Some("byte"), vec![])),
                    Type::Float => mips_instrs.push(get_target_code("mips", "test_unequal", Some("float"), vec![])),
                    Type::Double => mips_instrs.push(get_target_code("mips", "test_unequal", Some("double"), vec![])),
                    Type::Char => mips_instrs.push(get_target_code("mips", "test_unequal", Some("char"), vec![])),
                    Type::Void => panic!("Cannot apply != operator to type {:?}", op_type),
                    _ => todo!()
                }

                stack_types.push(Type::Byte);
            },

            IntermediateInstr::GreaterThan => {
                let op_type = stack_types.pop().unwrap();
                stack_types.pop();

                match op_type {
                    Type::Integer => mips_instrs.push(get_target_code("mips", "test_greater_than", Some("int"), vec![])),
                    Type::Long => mips_instrs.push(get_target_code("mips", "test_greater_than", Some("long"), vec![])),
                    Type::Byte => mips_instrs.push(get_target_code("mips", "test_greater_than", Some("byte"), vec![])),
                    Type::Float => mips_instrs.push(get_target_code("mips", "test_greater_than", Some("float"), vec![])),
                    Type::Double => mips_instrs.push(get_target_code("mips", "test_greater_than", Some("double"), vec![])),
                    Type::Char | Type::Void => panic!("Cannot apply > operator to type {:?}", op_type),
                    _ => todo!()
                }

                stack_types.push(Type::Byte);
            },

            IntermediateInstr::GreaterEqual => {
                let op_type = stack_types.pop().unwrap();
                stack_types.pop();

                match op_type {
                    Type::Integer => mips_instrs.push(get_target_code("mips", "test_greater_equal", Some("int"), vec![])),
                    Type::Long => mips_instrs.push(get_target_code("mips", "test_greater_equal", Some("long"), vec![])),
                    Type::Byte => mips_instrs.push(get_target_code("mips", "test_greater_equal", Some("byte"), vec![])),
                    Type::Float => mips_instrs.push(get_target_code("mips", "test_greater_equal", Some("float"), vec![])),
                    Type::Double => mips_instrs.push(get_target_code("mips", "test_greater_equal", Some("double"), vec![])),
                    Type::Char | Type::Void => panic!("Cannot apply >= operator to type {:?}", op_type),
                    _ => todo!()
                }

                stack_types.push(Type::Byte);
            },

            IntermediateInstr::LessThan => {
                let op_type = stack_types.pop().unwrap();
                stack_types.pop();

                match op_type {
                    Type::Integer => mips_instrs.push(get_target_code("mips", "test_less_than", Some("int"), vec![])),
                    Type::Long => mips_instrs.push(get_target_code("mips", "test_less_than", Some("long"), vec![])),
                    Type::Byte => mips_instrs.push(get_target_code("mips", "test_less_than", Some("byte"), vec![])),
                    Type::Float => mips_instrs.push(get_target_code("mips", "test_less_than", Some("float"), vec![])),
                    Type::Double => mips_instrs.push(get_target_code("mips", "test_less_than", Some("double"), vec![])),
                    Type::Char | Type::Void => panic!("Cannot apply < operator to type {:?}", op_type),
                    _ => todo!()
                }

                stack_types.push(Type::Byte);
            },

            IntermediateInstr::LessEqual => {
                let op_type = stack_types.pop().unwrap();
                stack_types.pop();

                match op_type {
                    Type::Integer => mips_instrs.push(get_target_code("mips", "test_less_equal", Some("int"), vec![])),
                    Type::Long => mips_instrs.push(get_target_code("mips", "test_less_equal", Some("long"), vec![])),
                    Type::Byte => mips_instrs.push(get_target_code("mips", "test_less_equal", Some("byte"), vec![])),
                    Type::Float => mips_instrs.push(get_target_code("mips", "test_less_equal", Some("float"), vec![])),
                    Type::Double => mips_instrs.push(get_target_code("mips", "test_less_equal", Some("double"), vec![])),
                    Type::Char | Type::Void => panic!("Cannot apply <= operator to type {:?}", op_type),
                    _ => todo!()
                }

                stack_types.push(Type::Byte);
            },

            IntermediateInstr::LogicAnd => {
                stack_types.pop();
                mips_instrs.push(get_target_code("mips", "logical_and", None, vec![]));
            },

            IntermediateInstr::LogicOr => {
                stack_types.pop();
                mips_instrs.push(get_target_code("mips", "logical_or", None, vec![]));
            },

            IntermediateInstr::LogicXor => {
                stack_types.pop();
                mips_instrs.push(get_target_code("mips", "logical_xor", None, vec![]));
            },

            IntermediateInstr::JumpZero(label) => {
                let op_type = stack_types.pop().unwrap();
                match op_type {
                    Type::Integer => mips_instrs.push(get_target_code("mips", "jump_zero", Some("int"), vec![label])),
                    Type::Long => mips_instrs.push(get_target_code("mips", "jump_zero", Some("long"), vec![label])),
                    Type::Byte => mips_instrs.push(get_target_code("mips", "jump_zero", Some("byte"), vec![label])),
                    _ => todo!()
                }
            },

            IntermediateInstr::Call(func_name, return_type) => {
                let frame_size = get_frame_size(&func_name, symbol_table);
                mips_instrs.push(get_target_code("mips", "call", Some(&return_type.to_string()), vec![func_name.clone(), func_name, frame_size.to_string()]));
                if return_type != Type::Void {
                    stack_types.push(return_type);
                }
            },

            IntermediateInstr::LoadParam(param_type, offset) => {
                match param_type {
                    Type::Integer | Type::Byte | Type::Float | Type::Char => {
                        mips_instrs.push(get_target_code("mips", "load_param", 
                            Some(&param_type.to_string()), 
                            vec![((offset + 2) * 4).to_string()]
                        ));
                    },

                    Type::Long | Type::Double => {
                        mips_instrs.push(get_target_code("mips", "load_param", 
                            Some(&param_type.to_string()), 
                            vec![
                                ((offset + 2) * 4).to_string(),
                                ((offset + 3) * 4).to_string()
                            ]
                        ));
                    },

                    Type::Void => panic!("Cannot load parameter of type Void"),
                    _ => todo!()
                }
            },

            IntermediateInstr::Jump(label) => mips_instrs.push(get_target_code("mips", "jump", None, vec![label])),
            IntermediateInstr::Label(label) => mips_instrs.push(get_target_code("mips", "label", None, vec![label])),
            
            _ => {}
        }
    }

    mips_instrs.push("\nend:".to_owned());
    mips_instrs.push("\tli $v0, 10 # halt syscall".to_owned());
    mips_instrs.push("\tsyscall".to_owned());

    file.write(text_section.join("\n").as_bytes()).expect("Could not write target text section to file");
    file.write(mips_instrs.join("\n").as_bytes()).expect("Could not write target code to file");

    Ok(())
}
