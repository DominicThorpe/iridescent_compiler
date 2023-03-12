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
    let mut current_stack_offset:i64 = 0;
    let mut stack_types:Vec<Type> = vec![];

    mips_instrs.push("j main # start program execution\n\n".to_owned());
    mips_instrs.append(&mut add_library("math64_mips"));

    for instr in intermediate_code {
        match instr {
            IntermediateInstr::FuncStart(name) => {
                let frame_size = get_frame_size(&name, &symbol_table);

                // add label for the function and size required for the stack local variables to the frame pointer
                mips_instrs.push(format!("{}: # start func", name));
                mips_instrs.push(format!("\taddiu $sp, $sp, -{}", frame_size));
                mips_instrs.push(format!("\tsw $fp, 0($sp)"));
                mips_instrs.push("\tmove $fp, $sp\n".to_string());
            },

            // Push an integer to the stack, use registers $t0 and $t2 to allow for future implementation of long datatype
            IntermediateInstr::Push(_, var) => {
                match var {
                    Argument::Integer(value) => {
                        current_stack_offset += 4;
                        stack_types.push(Type::Integer);
                        mips_instrs.push(format!("\tli $t4, {:?} # push int", value));
                        mips_instrs.push(format!("\tsw $t4, {}($sp)\n", current_stack_offset));
                    },

                    Argument::Long(value) => {
                        current_stack_offset += 8;
                        stack_types.push(Type::Long);
                        mips_instrs.push(format!("\tli $t4, {:?} # push long", value & 0xFFFF_FFFF));
                        mips_instrs.push(format!("\tli $t5, {:?}", (value as u64 & 0xFFFF_FFFF_0000_0000) >> 32));
                        mips_instrs.push(format!("\tsw $t4, {}($sp)", current_stack_offset));
                        mips_instrs.push(format!("\tsw $t5, {}($sp)\n", current_stack_offset - 4));
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

                        mips_instrs.push(format!("\tlw $t0, {}($sp) # store int", current_stack_offset));
                        mips_instrs.push(format!("\tsw $t0, -{}($sp)\n", stack_id_offset_map.get(&id).unwrap()));

                        stack_types.pop();
                        current_stack_offset -= 4;
                    },

                    Type::Long => {
                        // if the key does not exist, add a new key to represent a new local variable
                        if !stack_id_offset_map.contains_key(&id) {
                            current_var_offset += 8;
                            stack_id_offset_map.insert(id, current_var_offset);
                        }

                        mips_instrs.push(format!("\tlw $t0, {}($sp) # store long", current_stack_offset));
                        mips_instrs.push(format!("\tlw $t1, {}($sp)", current_stack_offset - 4));
                        mips_instrs.push(format!("\tsw $t0, -{}($sp)", stack_id_offset_map.get(&id).unwrap()));
                        mips_instrs.push(format!("\tsw $t1, -{}($sp)\n", stack_id_offset_map.get(&id).unwrap() - 4));

                        stack_types.pop();
                        current_stack_offset -= 8;
                    }

                    _ => todo!("Only int is currently supported for store instructions!")
                }
            },

            IntermediateInstr::Load(var_type, id) => {
                match var_type {
                    Type::Integer => {
                        current_stack_offset += 4;
                        stack_types.push(Type::Integer);

                        let offset = stack_id_offset_map.get(&id).unwrap();
                        mips_instrs.push(format!("\tlw $t0, -{}($sp) # load int", offset));
                        mips_instrs.push(format!("\tsw $t0, {}($sp)\n", current_stack_offset));
                    },

                    Type::Long => {
                        current_stack_offset += 8;
                        stack_types.push(Type::Long);

                        let offset = stack_id_offset_map.get(&id).unwrap();
                        mips_instrs.push(format!("\tlw $t0, -{}($sp) # load long", offset));
                        mips_instrs.push(format!("\tlw $t1, -{}($sp)", offset - 4));
                        mips_instrs.push(format!("\tsw $t0, {}($sp)", current_stack_offset));
                        mips_instrs.push(format!("\tsw $t1, {}($sp)\n", current_stack_offset - 4));
                    },

                    _ => todo!("Only int is currently supported for store instructions!")
                }
            },

            IntermediateInstr::Return(return_type) => {
                match return_type {
                    Type::Integer => {
                        mips_instrs.push(format!("\tlw $a0, {}($sp) # return int\n", current_stack_offset));
                    },

                    Type::Long => {
                        mips_instrs.push(format!("\tlw $a0, {}($sp) # return long", current_stack_offset));
                        mips_instrs.push(format!("\tlw $a1, {}($sp)\n", current_stack_offset - 4));
                    }
    
                    _ => todo!()
                }

                stack_types.pop();
            },

            IntermediateInstr::Add => {
                let add_type = stack_types.pop().unwrap();
                match add_type {
                    Type::Integer => {
                        mips_instrs.push(format!("\tlw $t0, {}($sp) # add int", current_stack_offset));
                        mips_instrs.push(format!("\tlw $t2, {}($sp)", current_stack_offset - 4));
                        mips_instrs.push(format!("\tadd $t0, $t2, $t0"));
                        mips_instrs.push(format!("\tsw $t0, {}($sp)\n", current_stack_offset - 4));
                        current_stack_offset -= 4;

                        stack_types.pop();
                        stack_types.push(Type::Integer);
                    },

                    Type::Long => {
                        mips_instrs.push(format!("\tlw $t0, {}($sp) # add long", current_stack_offset));
                        mips_instrs.push(format!("\tlw $t1, {}($sp)", current_stack_offset - 4));
                        mips_instrs.push(format!("\tlw $t2, {}($sp)", current_stack_offset - 8));
                        mips_instrs.push(format!("\tlw $t3, {}($sp)", current_stack_offset - 12));

                        mips_instrs.push(format!("\tadd $t0, $t0, $t2"));
                        mips_instrs.push(format!("\tadd $t1, $t1, $t3"));

                        mips_instrs.push(format!("\tsw $t0, {}($sp)", current_stack_offset - 8));
                        mips_instrs.push(format!("\tsw $t1, {}($sp)\n", current_stack_offset - 12));

                        current_stack_offset -= 8;

                        stack_types.pop();
                        stack_types.pop();
                        stack_types.push(Type::Long);
                    },

                    _ => todo!()
                }
            },

            IntermediateInstr::Sub => {
                let sub_type = stack_types.pop().unwrap();
                match sub_type {
                    Type::Integer => {
                        mips_instrs.push(format!("\tlw $t0, {}($sp) # sub integer", current_stack_offset));
                        mips_instrs.push(format!("\tlw $t2, {}($sp)", current_stack_offset - 4));
                        mips_instrs.push(format!("\tsub $t0, $t2, $t0"));
                        mips_instrs.push(format!("\tsw $t0, {}($sp)\n", current_stack_offset - 4));

                        current_stack_offset -= 4;
                        stack_types.pop();
                        stack_types.push(Type::Integer);
                    },

                    Type::Long => {
                        mips_instrs.push(format!("\tlw $t0, {}($sp) # sub long", current_stack_offset));
                        mips_instrs.push(format!("\tlw $t1, {}($sp)", current_stack_offset - 4));
                        mips_instrs.push(format!("\tlw $t2, {}($sp)", current_stack_offset - 8));
                        mips_instrs.push(format!("\tlw $t3, {}($sp)", current_stack_offset - 12));

                        mips_instrs.push(format!("\tsubu $t0, $t2, $t0"));
                        mips_instrs.push(format!("\tsubu $t1, $t3, $t1"));

                        mips_instrs.push(format!("\tsw $t0, {}($sp)", current_stack_offset - 8));
                        mips_instrs.push(format!("\tsw $t1, {}($sp)\n", current_stack_offset - 12));

                        current_stack_offset -= 8;

                        stack_types.pop();
                        stack_types.pop();
                        stack_types.push(Type::Long);
                    },

                    _ => todo!()
                }
            },

            IntermediateInstr::Mult => {
                let mult_type = stack_types.pop().unwrap();
                match mult_type {
                    Type::Integer => {
                        mips_instrs.push(format!("\tlw $t0, {}($sp) # multiply int", current_stack_offset));
                        mips_instrs.push(format!("\tlw $t2, {}($sp)", current_stack_offset - 4));
                        mips_instrs.push(format!("\tmul $t0, $t2, $t0"));
                        mips_instrs.push(format!("\tsw $t0, {}($sp)\n", current_stack_offset - 4));

                        current_stack_offset -= 4;
                        stack_types.pop();
                        stack_types.push(Type::Integer);
                    },

                    Type::Long => {
                        mips_instrs.push(format!("\tlw $t0, {}($sp) # multiply long", current_stack_offset));
                        mips_instrs.push(format!("\tlw $t1, {}($sp)", current_stack_offset - 4));
                        mips_instrs.push(format!("\tlw $t2, {}($sp)", current_stack_offset - 8));
                        mips_instrs.push(format!("\tlw $t3, {}($sp)", current_stack_offset - 12));

                        mips_instrs.push(format!("\tmult $t0, $t2"));
                        mips_instrs.push(format!("\tmflo $t4"));
                        mips_instrs.push(format!("\tmfhi $s0"));
                        mips_instrs.push(format!("\tmult $t0, $t3"));
                        mips_instrs.push(format!("\tmflo $t7"));
                        mips_instrs.push(format!("\tadd $s1, $s0, $t7"));
                        mips_instrs.push(format!("\tmult $t1, $t2"));
                        mips_instrs.push(format!("\tmfhi $t7"));
                        mips_instrs.push(format!("\tadd $t5, $t7, $s1"));

                        mips_instrs.push(format!("\tsw $t4, {}($sp)", current_stack_offset - 8));
                        mips_instrs.push(format!("\tsw $t5, {}($sp)\n", current_stack_offset - 12));

                        current_stack_offset -= 8;

                        stack_types.pop();
                        stack_types.pop();
                        stack_types.push(Type::Long);
                    },

                    _ => todo!()
                }
            },

            IntermediateInstr::Div => {
                let mult_type = stack_types.pop().unwrap();
                match mult_type {
                    Type::Integer => {
                        mips_instrs.push(format!("\tlw $t0, {}($sp) # divide int", current_stack_offset));
                        mips_instrs.push(format!("\tlw $t2, {}($sp)", current_stack_offset - 4));
                        mips_instrs.push(format!("\tdiv $t0, $t2, $t0"));
                        mips_instrs.push(format!("\tsw $t0, {}($sp)\n", current_stack_offset - 4));

                        current_stack_offset -= 4;
                        stack_types.pop();
                        stack_types.push(Type::Integer);
                    },

                    Type::Long => {
                        mips_instrs.push(format!("\tlw $a0, {}($sp) # divide long", current_stack_offset - 8));
                        mips_instrs.push(format!("\tlw $a1, {}($sp)", current_stack_offset - 12));
                        mips_instrs.push(format!("\tlw $a2, {}($sp)", current_stack_offset));
                        mips_instrs.push(format!("\tlw $a3, {}($sp)", current_stack_offset - 4));
                        
                        mips_instrs.push(format!("\tjal __divint64"));

                        mips_instrs.push(format!("\tmove $t0, $a0"));
                        mips_instrs.push(format!("\tmove $t1, $a1"));

                        mips_instrs.push(format!("\tsw $t0, {}($sp)", current_stack_offset - 8));
                        mips_instrs.push(format!("\tsw $t1, {}($sp)\n", current_stack_offset - 12));

                        current_stack_offset -= 8;

                        stack_types.pop();
                        stack_types.pop();
                        stack_types.push(Type::Long);
                    },

                    _ => todo!()
                }
            },

            IntermediateInstr::BitwiseAnd => {
                let operand_type = stack_types.pop().unwrap();
                match operand_type {
                    Type::Integer => {
                        mips_instrs.push(format!("\tlw $t0, {}($sp) # bitwise and int", current_stack_offset));
                        mips_instrs.push(format!("\tlw $t2, {}($sp)", current_stack_offset - 4));
                        mips_instrs.push(format!("\tand $t0, $t2, $t0"));
                        mips_instrs.push(format!("\tsw $t0, {}($sp)\n", current_stack_offset - 4));

                        current_stack_offset -= 4;
                        stack_types.pop();
                        stack_types.push(Type::Integer);
                    },

                    Type::Long => {
                        mips_instrs.push(format!("\tlw $t0, {}($sp) # bitwise and long", current_stack_offset - 8));
                        mips_instrs.push(format!("\tlw $t1, {}($sp)", current_stack_offset - 12));
                        mips_instrs.push(format!("\tlw $t2, {}($sp)", current_stack_offset));
                        mips_instrs.push(format!("\tlw $t3, {}($sp)", current_stack_offset - 4));

                        mips_instrs.push(format!("\tand $t0, $t2, $t0"));
                        mips_instrs.push(format!("\tand $t1, $t3, $t1"));

                        mips_instrs.push(format!("\tsw $t0, {}($sp)", current_stack_offset - 12));
                        mips_instrs.push(format!("\tsw $t1, {}($sp)\n", current_stack_offset - 8));

                        current_stack_offset -= 8;

                        stack_types.pop();
                        stack_types.pop();
                        stack_types.push(Type::Long);
                    },

                    _ => todo!()
                }
            },

            IntermediateInstr::BitwiseOr => {
                let operand_type = stack_types.pop().unwrap();
                match operand_type {
                    Type::Integer => {
                        mips_instrs.push(format!("\tlw $t0, {}($sp) # bitwise or int", current_stack_offset));
                        mips_instrs.push(format!("\tlw $t2, {}($sp)", current_stack_offset - 4));
                        mips_instrs.push(format!("\tor $t0, $t2, $t0"));
                        mips_instrs.push(format!("\tsw $t0, {}($sp)\n", current_stack_offset - 4));

                        current_stack_offset -= 4;
                        stack_types.pop();
                        stack_types.push(Type::Integer);
                    },

                    Type::Long => {
                        mips_instrs.push(format!("\tlw $t0, {}($sp) # bitwise or long", current_stack_offset - 8));
                        mips_instrs.push(format!("\tlw $t1, {}($sp)", current_stack_offset - 12));
                        mips_instrs.push(format!("\tlw $t2, {}($sp)", current_stack_offset));
                        mips_instrs.push(format!("\tlw $t3, {}($sp)", current_stack_offset - 4));

                        mips_instrs.push(format!("\tor $t0, $t2, $t0"));
                        mips_instrs.push(format!("\tor $t1, $t3, $t1"));

                        mips_instrs.push(format!("\tsw $t0, {}($sp)", current_stack_offset - 12));
                        mips_instrs.push(format!("\tsw $t1, {}($sp)\n", current_stack_offset - 8));

                        current_stack_offset -= 8;

                        stack_types.pop();
                        stack_types.pop();
                        stack_types.push(Type::Long);
                    },

                    _ => todo!()
                }
            },

            IntermediateInstr::BitwiseXor => {
                let operand_type = stack_types.pop().unwrap();
                match operand_type {
                    Type::Integer => {
                        mips_instrs.push(format!("\tlw $t0, {}($sp) # bitwise or int", current_stack_offset));
                        mips_instrs.push(format!("\tlw $t2, {}($sp)", current_stack_offset - 4));
                        mips_instrs.push(format!("\txor $t0, $t2, $t0"));
                        mips_instrs.push(format!("\tsw $t0, {}($sp)\n", current_stack_offset - 4));

                        current_stack_offset -= 4;
                        stack_types.pop();
                        stack_types.push(Type::Integer);
                    },

                    Type::Long => {
                        mips_instrs.push(format!("\tlw $t0, {}($sp) # bitwise or long", current_stack_offset - 8));
                        mips_instrs.push(format!("\tlw $t1, {}($sp)", current_stack_offset - 12));
                        mips_instrs.push(format!("\tlw $t2, {}($sp)", current_stack_offset));
                        mips_instrs.push(format!("\tlw $t3, {}($sp)", current_stack_offset - 4));

                        mips_instrs.push(format!("\txor $t0, $t2, $t0"));
                        mips_instrs.push(format!("\txor $t1, $t3, $t1"));

                        mips_instrs.push(format!("\tsw $t0, {}($sp)", current_stack_offset - 12));
                        mips_instrs.push(format!("\tsw $t1, {}($sp)\n", current_stack_offset - 8));

                        current_stack_offset -= 8;

                        stack_types.pop();
                        stack_types.pop();
                        stack_types.push(Type::Long);
                    },

                    _ => todo!()
                }
            },

            IntermediateInstr::NumNeg => {
                let operand_type = stack_types.pop().unwrap();
                match operand_type {
                    Type::Integer => {
                        mips_instrs.push(format!("\tlw $t0, {}($sp) # numerical negation int", current_stack_offset));
                        mips_instrs.push(format!("\tsubu $t0, $zero, $t0"));
                        mips_instrs.push(format!("\tsw $t0, {}($sp)\n", current_stack_offset));
                    },

                    Type::Long => {
                        mips_instrs.push(format!("\tlw $t0, {}($sp) # numerical negation long", current_stack_offset));
                        mips_instrs.push(format!("\tlw $t1, {}($sp)", current_stack_offset - 4));

                        mips_instrs.push(format!("\tnor $t0, $t0, $zero"));
                        mips_instrs.push(format!("\tnor $t1, $t1, $zero"));
                        mips_instrs.push(format!("\taddiu $t0, $t0, 1"));
                        mips_instrs.push(format!("\tsltiu $t2, $t0, 1"));
                        mips_instrs.push(format!("\taddu $t1, $t1, $t2"));

                        mips_instrs.push(format!("\tsw $t0, {}($sp)\n", current_stack_offset - 4));
                        mips_instrs.push(format!("\tsw $t1, {}($sp)\n", current_stack_offset));
                    },

                    _ => todo!()
                }

            },

            IntermediateInstr::Complement => {
                let operand_type = stack_types.pop().unwrap();
                match operand_type {
                    Type::Integer => {
                        mips_instrs.push(format!("\tlw $t0, {}($sp)", current_stack_offset));
                        mips_instrs.push(format!("\tnot $t0, $t0"));
                        mips_instrs.push(format!("\tsw $t0, {}($sp)\n", current_stack_offset));
                    },

                    Type::Long => {
                        mips_instrs.push(format!("\tlw $t0, {}($sp) # numerical negation long", current_stack_offset));
                        mips_instrs.push(format!("\tlw $t1, {}($sp)", current_stack_offset - 4));

                        mips_instrs.push(format!("\tnot $t0, $t0"));
                        mips_instrs.push(format!("\tnot $t1, $t1"));

                        mips_instrs.push(format!("\tsw $t0, {}($sp)\n", current_stack_offset - 4));
                        mips_instrs.push(format!("\tsw $t1, {}($sp)\n", current_stack_offset));
                    },

                    _ => todo!()
                }
            },

            IntermediateInstr::LogicNeg => {
                let operand_type = stack_types.pop().unwrap();
                match operand_type {
                    Type::Integer => {
                        mips_instrs.push(format!("\tlw $t0, {}($sp) # logical negation int", current_stack_offset));
                        mips_instrs.push(format!("\tslt $t0, $zero, $t0"));
                        mips_instrs.push(format!("\tsw $t0, {}($sp)\n", current_stack_offset));
                    },

                    Type::Long => {
                        mips_instrs.push(format!("\tlw $t0, {}($sp) # logical negation long", current_stack_offset));
                        mips_instrs.push(format!("\tlw $t1, {}($sp)", current_stack_offset - 4));

                        mips_instrs.push(format!("\tslt $t0, $zero, $t0"));
                        mips_instrs.push(format!("\tslt $t1, $zero, $t0"));
                        mips_instrs.push(format!("\tand $t0, $t0, $t1"));
                        mips_instrs.push(format!("\tmove $t0, $zero"));

                        mips_instrs.push(format!("\tsw $t0, {}($sp)\n", current_stack_offset - 4));
                        mips_instrs.push(format!("\tsw $t1, {}($sp)\n", current_stack_offset));
                    },

                    _ => todo!()
                }
            },

            IntermediateInstr::LeftShiftLogical => {
                let operand_type = stack_types.pop().unwrap();
                match operand_type {
                    Type::Integer => {
                        mips_instrs.push(format!("\tlw $t0, {}($sp) # shift left int", current_stack_offset));
                        mips_instrs.push(format!("\tlw $t2, {}($sp)", current_stack_offset - 4));
                        mips_instrs.push(format!("\tsllv $t0, $t2, $t0"));
                        mips_instrs.push(format!("\tsw $t0, {}($sp)\n", current_stack_offset - 4));

                        current_stack_offset -= 4;
                        stack_types.pop();
                    },

                    Type::Long => {
                        mips_instrs.push(format!("\tlw $a2, {}($sp) # shift left long", current_stack_offset));
                        mips_instrs.push(format!("\tlw $a0, {}($sp)", current_stack_offset - 12));
                        mips_instrs.push(format!("\tlw $a1, {}($sp)", current_stack_offset - 8));

                        mips_instrs.push(format!("\tjal __sllint64"));
                        mips_instrs.push(format!("\tmove $t0, $a0"));
                        mips_instrs.push(format!("\tmove $t1, $a1"));

                        mips_instrs.push(format!("\tsw $t0, {}($sp)\n", current_stack_offset - 8));
                        mips_instrs.push(format!("\tsw $t1, {}($sp)\n", current_stack_offset - 12));

                        current_stack_offset -= 8;
                        stack_types.pop();
                    },

                    _ => todo!()
                }
            },

            IntermediateInstr::RightShiftLogical => {
                mips_instrs.push(format!("\tlw $t0, {}($sp)", current_stack_offset));
                mips_instrs.push(format!("\tlw $t2, {}($sp)", current_stack_offset - 4));
                mips_instrs.push(format!("\tsrlv $t0, $t2, $t0"));
                mips_instrs.push(format!("\tsw $t0, {}($sp)\n", current_stack_offset - 4));
                current_stack_offset -= 4;
            },

            IntermediateInstr::RightShiftArithmetic => {
                mips_instrs.push(format!("\tlw $t0, {}($sp)", current_stack_offset));
                mips_instrs.push(format!("\tlw $t2, {}($sp)", current_stack_offset - 4));
                mips_instrs.push(format!("\tsrav $t0, $t2, $t0"));
                mips_instrs.push(format!("\tsw $t0, {}($sp)\n", current_stack_offset - 4));
                current_stack_offset -= 4;
            },

            IntermediateInstr::Equal => {
                mips_instrs.push(format!("\tlw $t0, {}($sp)", current_stack_offset));
                mips_instrs.push(format!("\tlw $t2, {}($sp)", current_stack_offset - 4));
                mips_instrs.push(format!("\tseq $t0, $t0, $t2"));
                mips_instrs.push(format!("\tsw $t0, {}($sp)\n", current_stack_offset - 4));
                current_stack_offset -= 4;
            },

            IntermediateInstr::NotEqual => {
                mips_instrs.push(format!("\tlw $t0, {}($sp)", current_stack_offset));
                mips_instrs.push(format!("\tlw $t2, {}($sp)", current_stack_offset - 4));
                mips_instrs.push(format!("\tsne $t0, $t0, $t2"));
                mips_instrs.push(format!("\tsw $t0, {}($sp)\n", current_stack_offset - 4));
                current_stack_offset -= 4;
            },

            IntermediateInstr::GreaterThan => {
                mips_instrs.push(format!("\tlw $t0, {}($sp)", current_stack_offset));
                mips_instrs.push(format!("\tlw $t2, {}($sp)", current_stack_offset - 4));
                mips_instrs.push(format!("\tsgt $t0, $t2, $t0"));
                mips_instrs.push(format!("\tsw $t0, {}($sp)\n", current_stack_offset - 4));
                current_stack_offset -= 4;
            },

            IntermediateInstr::GreaterEqual => {
                mips_instrs.push(format!("\tlw $t0, {}($sp)", current_stack_offset));
                mips_instrs.push(format!("\tlw $t2, {}($sp)", current_stack_offset - 4));
                mips_instrs.push(format!("\tsge $t0, $t2, $t0"));
                mips_instrs.push(format!("\tsw $t0, {}($sp)\n", current_stack_offset - 4));
                current_stack_offset -= 4;
            },

            IntermediateInstr::LessThan => {
                mips_instrs.push(format!("\tlw $t0, {}($sp)", current_stack_offset));
                mips_instrs.push(format!("\tlw $t2, {}($sp)", current_stack_offset - 4));
                mips_instrs.push(format!("\tslt $t0, $t2, $t0"));
                mips_instrs.push(format!("\tsw $t0, {}($sp)\n", current_stack_offset - 4));
                current_stack_offset -= 4;
            },

            IntermediateInstr::LessEqual => {
                mips_instrs.push(format!("\tlw $t0, {}($sp)", current_stack_offset));
                mips_instrs.push(format!("\tlw $t2, {}($sp)", current_stack_offset - 4));
                mips_instrs.push(format!("\tsle $t0, $t2, $t0"));
                mips_instrs.push(format!("\tsw $t0, {}($sp)\n", current_stack_offset - 4));
                current_stack_offset -= 4;
            },

            IntermediateInstr::LogicAnd => {
                mips_instrs.push(format!("\tlw $t0, {}($sp)", current_stack_offset));
                mips_instrs.push(format!("\tlw $t2, {}($sp)", current_stack_offset - 4));
                mips_instrs.push(format!("\tand $t0, $t2, $t0"));
                mips_instrs.push(format!("\tsw $t0, {}($sp)\n", current_stack_offset - 4));
                current_stack_offset -= 4;
            },

            IntermediateInstr::LogicOr => {
                mips_instrs.push(format!("\tlw $t0, {}($sp)", current_stack_offset));
                mips_instrs.push(format!("\tlw $t2, {}($sp)", current_stack_offset - 4));
                mips_instrs.push(format!("\tor $t0, $t2, $t0"));
                mips_instrs.push(format!("\tsw $t0, {}($sp)\n", current_stack_offset - 4));
                current_stack_offset -= 4;
            },

            IntermediateInstr::LogicXor => {
                mips_instrs.push(format!("\tlw $t0, {}($sp)", current_stack_offset));
                mips_instrs.push(format!("\tlw $t2, {}($sp)", current_stack_offset - 4));
                mips_instrs.push(format!("\txor $t0, $t2, $t0"));
                mips_instrs.push(format!("\tsw $t0, {}($sp)\n", current_stack_offset - 4));
                current_stack_offset -= 4;
            },

            IntermediateInstr::JumpZero(label) => {
                mips_instrs.push(format!("\tlw $t0, {}($sp)", current_stack_offset));
                mips_instrs.push(format!("\tbeqz $t0, {}\n", label));
                current_stack_offset -= 4;
            },

            IntermediateInstr::Jump(label) => mips_instrs.push(format!("\tj {}\n", label)),
            IntermediateInstr::Label(label) => mips_instrs.push(format!("\n\n{}:", label)),
            
            _ => {}
        }
    }

    mips_instrs.push("\nend:".to_owned());
    mips_instrs.push("\tli $v0, 10 # halt syscall".to_owned());
    mips_instrs.push("\tsyscall".to_owned());

    file.write(mips_instrs.join("\n").as_bytes()).expect("Could not write target code to file");

    Ok(())
}
