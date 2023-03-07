use std::fs::OpenOptions;
use std::io::prelude::*;
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
                    Type::Integer => {
                        frame_size += 4;
                    },
                    _ => todo!()
                }
            },

            _ => {}
        }
    }

    frame_size
}


/**
 * Generates the final MIPS assembly code that can then be compiled to native binary using a separate tool.
 */
pub fn generate_mips(intermediate_code:Vec<IntermediateInstr>, symbol_table:SymbolTable, filename:&str) -> Result<(), Box<dyn Error>> {
    let mut file = OpenOptions::new().write(true).truncate(true).create(true).open(filename)?;
    let mut mips_instrs:Vec<String> = vec![];
    let mut stack_id_offset_map: HashMap<usize, usize> = HashMap::new();
    let mut current_offset:usize = 0;

    let mut curr_register = "$t2";

    mips_instrs.push("j main # start program execution\n\n".to_owned());

    for instr in intermediate_code {
        match instr {
            IntermediateInstr::FuncStart(name) => {
                let frame_size = get_frame_size(&name, &symbol_table);

                // add label for the function and size required for the stack local variables to the frame pointer
                mips_instrs.push(format!("{}: ", name));
                mips_instrs.push(format!("\taddiu $sp, $sp, -{}", frame_size));
                mips_instrs.push(format!("\tsw $fp, 0($sp)"));
                mips_instrs.push("\tmove $fp, $sp\n".to_string());
            },

            // Push an integer to the stack, use registers $t0 and $t2 to allow for future implementation of long datatype
            IntermediateInstr::Push(_, var) => {
                match var {
                    Argument::Integer(value) => {
                        if curr_register == "$t0" {
                            curr_register = "$t2";
                        } else {
                            curr_register = "$t0";
                        }
                        mips_instrs.push(format!("\tli {}, {:?}", curr_register, value));
                    },

                    _ => todo!()
                }
            },

            IntermediateInstr::Store(var_type, id) => {
                match var_type {
                    Type::Integer => {
                        // Should not be able to have duplicate keys
                        // might move this outside of the match statement?
                        if stack_id_offset_map.contains_key(&id) {
                            panic!("Stack offset map already has key {}", id);
                        }

                        current_offset += 4;
                        stack_id_offset_map.insert(id, current_offset);

                        mips_instrs.push(format!("\tsw {}, -{}($sp)", curr_register, current_offset));
                    },

                    _ => todo!("Only int is currently supported for store instructions!")
                }
            },

            IntermediateInstr::Load(var_type, id) => {
                if curr_register == "$t0" {
                    curr_register = "$t2";
                } else {
                    curr_register = "$t0";
                }

                match var_type {
                    Type::Integer => {
                        let offset = stack_id_offset_map.get(&id).unwrap();
                        mips_instrs.push(format!("\tlw {}, -{}($sp)", curr_register, offset));
                    },

                    _ => todo!("Only int is currently supported for store instructions!")
                }
            },

            IntermediateInstr::Return(return_type) => {
                match return_type {
                    Type::Integer => {
                        mips_instrs.push("\tmove $a0, $t0".to_owned());
                    },
    
                    _ => todo!()
                }
            },

            IntermediateInstr::NumNeg => mips_instrs.push(format!("\tsubu {}, $zero, {}", curr_register, curr_register)),
            IntermediateInstr::Complement => mips_instrs.push(format!("\tnot {}, {}", curr_register, curr_register)),
            IntermediateInstr::LogicNeg => mips_instrs.push(format!("\tslt {}, $zero, {}", curr_register, curr_register)),
            
            _ => {}
        }
    }

    mips_instrs.push("\n\tli $v0, 10 # halt syscall".to_owned());
    mips_instrs.push("\tsyscall".to_owned());

    file.write(mips_instrs.join("\n").as_bytes()).expect("Could not write target code to file");

    Ok(())
}
