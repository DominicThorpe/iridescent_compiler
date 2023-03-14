use std::fs::OpenOptions;
use std::io::prelude::*;
use std::io::{self, BufRead};
use std::error::Error;
use std::collections::HashMap;

use crate::frontend::intermediate_gen::{IntermediateInstr, Argument};
use crate::frontend::semantics::{SymbolTable, SymbolTableRow};
use crate::frontend::ast::Type;


#[allow(dead_code)]
#[derive(Debug)]
struct VariableTableRow {
    identifier: usize,
    offset:usize
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
        panic!("Instruction {} takes {} arguments, but only {} were provided", instr, required_arg_count, arguments.len());
    }

    for arg in arguments {
        target_code = target_code.replacen("{}", &arg, 1);
    }

    target_code += "\n";
    target_code
}


/**
 * Calculates the size required for the function frame. Used when invoking a function.
 */
fn get_frame_size(function_id:&str, symbol_table:&SymbolTable) -> u64 {
    let mut frame_size = 8; // make space for the return address
    for symbol in &symbol_table.rows {
        match symbol {
            SymbolTableRow::Variable {primitive_type, function_id: fid, ..} => {
                if fid != function_id {
                    continue;
                }

                // add the size in bytes of the datatype to the frame size
                match primitive_type {
                    Type::Integer => frame_size += 4,
                    Type::Long => frame_size += 8,
                    _ => todo!()
                }
            },

            _ => {}
        }
    }

    frame_size
}


fn add_library(library_name:&str) -> Vec<String> {
    let file = OpenOptions::new().read(true).open(format!("src/backend/{}.asm", library_name)).unwrap();
    let lines:Vec<String> = io::BufReader::new(file).lines().map(|l| l.unwrap()).collect();
    lines
}


/**
 * Generates the final MIPS assembly code that can then be compiled to native binary using a separate tool.
 */
pub fn generate_mips(intermediate_code:Vec<IntermediateInstr>, symbol_table:SymbolTable, filename:&str) -> Result<(), Box<dyn Error>> {
    let mut file = OpenOptions::new().write(true).truncate(true).create(true).open(filename)?;
    let mut mips_instrs:Vec<String> = vec![];
    let mut stack_id_offset_map: HashMap<usize, usize> = HashMap::new();
    let mut current_var_offset:usize = 0;
    let mut stack_types:Vec<Type> = vec![];

    mips_instrs.push("j main # start program execution\n\n".to_owned());
    mips_instrs.append(&mut add_library("math64_mips"));

    for instr in intermediate_code {
        match instr {
            IntermediateInstr::FuncStart(name) => {
                let frame_size = get_frame_size(&name, &symbol_table);
                mips_instrs.push(get_target_code("mips", "start_func", None, vec![name, frame_size.to_string()]));
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
                        let upper_bits:u64 = value as u64 & 0xFFFF_FFFF_0000_0000 >> 32;
                        let lower_bits:u64 = value as u64 & 0xFFFF_FFFF;
                        mips_instrs.push(get_target_code("mips", "push", Some("long"), vec![
                            upper_bits.to_string(),
                            lower_bits.to_string()
                        ]));
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
                    }

                    _ => todo!("Only int is currently supported for store instructions!")
                }
            },

            IntermediateInstr::Load(var_type, id) => {
                match var_type {
                    Type::Integer => {
                        stack_types.push(Type::Integer);

                        let offset = stack_id_offset_map.get(&id).unwrap();
                        mips_instrs.push(get_target_code("mips", "load", Some("int"), vec![offset.to_string()]));
                    },

                    Type::Long => {
                        stack_types.push(Type::Long);

                        let offset = stack_id_offset_map.get(&id).unwrap();
                        mips_instrs.push(get_target_code("mips", "load", Some("long"), vec![
                            offset.to_string(), (offset - 4).to_string()
                        ]));
                    },

                    _ => todo!("Only int is currently supported for store instructions!")
                }
            },

            IntermediateInstr::Return(return_type) => {
                match return_type {
                    Type::Integer => mips_instrs.push(get_target_code("mips", "return", Some("int"), vec![])),
                    Type::Long => mips_instrs.push(get_target_code("mips", "return", Some("long"), vec![])),
                    _ => todo!()
                }

                stack_types.pop();
            },

            IntermediateInstr::Add => {
                let add_type = stack_types.pop().unwrap();
                match add_type {
                    Type::Integer => mips_instrs.push(get_target_code("mips", "add", Some("int"), vec![])),
                    Type::Long => mips_instrs.push(get_target_code("mips", "add", Some("long"), vec![])),
                    _ => todo!()
                }
            },

            IntermediateInstr::Sub => {
                let sub_type = stack_types.pop().unwrap();
                match sub_type {
                    Type::Integer => mips_instrs.push(get_target_code("mips", "sub", Some("int"), vec![])),
                    Type::Long => mips_instrs.push(get_target_code("mips", "sub", Some("long"), vec![])),
                    _ => todo!()
                }
            },
            
            IntermediateInstr::Mult => {
                let mult_type = stack_types.pop().unwrap();
                match mult_type {
                    Type::Integer => mips_instrs.push(get_target_code("mips", "mult", Some("int"), vec![])),
                    Type::Long => mips_instrs.push(get_target_code("mips", "mult", Some("long"), vec![])),
                    _ => todo!()
                }
            },

            IntermediateInstr::Div => {
                let op_type = stack_types.pop().unwrap();
                match op_type {
                    Type::Integer => mips_instrs.push(get_target_code("mips", "div", Some("int"), vec![])),
                    Type::Long => mips_instrs.push(get_target_code("mips", "div", Some("long"), vec![])),
                    _ => todo!()
                }
            },

            IntermediateInstr::BitwiseAnd => {
                let op_type = stack_types.pop().unwrap();
                match op_type {
                    Type::Integer => mips_instrs.push(get_target_code("mips", "bitwise_and", Some("int"), vec![])),
                    Type::Long => mips_instrs.push(get_target_code("mips", "bitwise_and", Some("long"), vec![])),
                    _ => todo!()
                }
            },

            IntermediateInstr::BitwiseOr => {
                let op_type = stack_types.pop().unwrap();
                match op_type {
                    Type::Integer => mips_instrs.push(get_target_code("mips", "bitwise_or", Some("int"), vec![])),
                    Type::Long => mips_instrs.push(get_target_code("mips", "bitwise_or", Some("long"), vec![])),
                    _ => todo!()
                }
            },

            IntermediateInstr::BitwiseXor => {
                let op_type = stack_types.pop().unwrap();
                match op_type {
                    Type::Integer => mips_instrs.push(get_target_code("mips", "bitwise_xor", Some("int"), vec![])),
                    Type::Long => mips_instrs.push(get_target_code("mips", "bitwise_xor", Some("long"), vec![])),
                    _ => todo!()
                }
            },

            IntermediateInstr::NumNeg => {
                let op_type = stack_types.last().unwrap();
                match op_type {
                    Type::Integer => mips_instrs.push(get_target_code("mips", "numerical_neg", Some("int"), vec![])),
                    Type::Long => mips_instrs.push(get_target_code("mips", "numerical_neg", Some("long"), vec![])),
                    _ => todo!()
                }
            },

            IntermediateInstr::Complement => {
                let op_type = stack_types.last().unwrap();
                match op_type {
                    Type::Integer => mips_instrs.push(get_target_code("mips", "complement", Some("int"), vec![])),
                    Type::Long => mips_instrs.push(get_target_code("mips", "complement", Some("long"), vec![])),
                    _ => todo!()
                }
            },

            IntermediateInstr::LogicNeg => {
                let op_type = stack_types.last().unwrap();
                match op_type {
                    Type::Integer => mips_instrs.push(get_target_code("mips", "logical_neg", Some("int"), vec![])),
                    Type::Long => mips_instrs.push(get_target_code("mips", "logical_neg", Some("long"), vec![])),
                    _ => todo!()
                }
            },

            IntermediateInstr::LeftShiftLogical => {
                let op_type = stack_types.pop().unwrap();
                match op_type {
                    Type::Integer => mips_instrs.push(get_target_code("mips", "sll", Some("int"), vec![])),
                    Type::Long => mips_instrs.push(get_target_code("mips", "sll", Some("long"), vec![])),
                    _ => todo!()
                }
            },

            IntermediateInstr::RightShiftLogical => {
                let op_type = stack_types.pop().unwrap();
                match op_type {
                    Type::Integer => mips_instrs.push(get_target_code("mips", "srl", Some("int"), vec![])),
                    Type::Long => mips_instrs.push(get_target_code("mips", "srl", Some("long"), vec![])),
                    _ => todo!()
                }
            },

            IntermediateInstr::RightShiftArithmetic => {
                let op_type = stack_types.pop().unwrap();
                match op_type {
                    Type::Integer => mips_instrs.push(get_target_code("mips", "sra", Some("int"), vec![])),
                    Type::Long => mips_instrs.push(get_target_code("mips", "sra", Some("long"), vec![])),
                    _ => todo!()
                }
            },
         
            IntermediateInstr::Equal => {
                let op_type = stack_types.pop().unwrap();
                match op_type {
                    Type::Integer => mips_instrs.push(get_target_code("mips", "test_equal", Some("int"), vec![])),
                    Type::Long => mips_instrs.push(get_target_code("mips", "test_equal", Some("long"), vec![])),
                    _ => todo!()
                }
            },

            IntermediateInstr::NotEqual => {
                let op_type = stack_types.pop().unwrap();
                match op_type {
                    Type::Integer => mips_instrs.push(get_target_code("mips", "test_unequal", Some("int"), vec![])),
                    Type::Long => mips_instrs.push(get_target_code("mips", "test_unequal", Some("long"), vec![])),
                    _ => todo!()
                }
            },

            IntermediateInstr::GreaterThan => {
                let op_type = stack_types.pop().unwrap();
                match op_type {
                    Type::Integer => mips_instrs.push(get_target_code("mips", "test_greater_than", Some("int"), vec![])),
                    Type::Long => mips_instrs.push(get_target_code("mips", "test_greater_than", Some("long"), vec![])),
                    _ => todo!()
                }
            },

            IntermediateInstr::GreaterEqual => {
                let op_type = stack_types.pop().unwrap();
                match op_type {
                    Type::Integer => mips_instrs.push(get_target_code("mips", "test_greater_equal", Some("int"), vec![])),
                    Type::Long => mips_instrs.push(get_target_code("mips", "test_greater_equal", Some("long"), vec![])),
                    _ => todo!()
                }
            },

            IntermediateInstr::LessThan => {
                let op_type = stack_types.pop().unwrap();
                match op_type {
                    Type::Integer => mips_instrs.push(get_target_code("mips", "test_less_than", Some("int"), vec![])),
                    Type::Long => mips_instrs.push(get_target_code("mips", "test_less_than", Some("long"), vec![])),
                    _ => todo!()
                }
            },

            IntermediateInstr::LessEqual => {
                let op_type = stack_types.pop().unwrap();
                match op_type {
                    Type::Integer => mips_instrs.push(get_target_code("mips", "test_less_equal", Some("int"), vec![])),
                    Type::Long => mips_instrs.push(get_target_code("mips", "test_less_equal", Some("long"), vec![])),
                    _ => todo!()
                }
            },

            IntermediateInstr::LogicAnd => {
                let op_type = stack_types.pop().unwrap();
                match op_type {
                    Type::Integer => mips_instrs.push(get_target_code("mips", "logical_and", Some("int"), vec![])),
                    Type::Long => mips_instrs.push(get_target_code("mips", "logical_and", Some("long"), vec![])),
                    _ => todo!()
                }
            },

            IntermediateInstr::LogicOr => {
                let op_type = stack_types.pop().unwrap();
                match op_type {
                    Type::Integer => mips_instrs.push(get_target_code("mips", "logical_or", Some("int"), vec![])),
                    Type::Long => mips_instrs.push(get_target_code("mips", "logical_or", Some("long"), vec![])),
                    _ => todo!()
                }
            },

            IntermediateInstr::LogicXor => {
                let op_type = stack_types.pop().unwrap();
                match op_type {
                    Type::Integer => mips_instrs.push(get_target_code("mips", "logical_xor", Some("int"), vec![])),
                    Type::Long => mips_instrs.push(get_target_code("mips", "logical_xor", Some("long"), vec![])),
                    _ => todo!()
                }
            },

            IntermediateInstr::JumpZero(label) => {
                let op_type = stack_types.pop().unwrap();
                match op_type {
                    Type::Integer => mips_instrs.push(get_target_code("mips", "jump_zero", Some("int"), vec![label])),
                    Type::Long => mips_instrs.push(get_target_code("mips", "jump_zero", Some("long"), vec![label])),
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

    file.write(mips_instrs.join("\n").as_bytes()).expect("Could not write target code to file");

    Ok(())
}
