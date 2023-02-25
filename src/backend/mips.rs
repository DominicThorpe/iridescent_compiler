use std::fs::OpenOptions;
use std::io::prelude::*;
use std::error::Error;

use crate::frontend::intermediate_gen::{IntermediateInstr, Argument};
use crate::frontend::semantics::{SymbolTable, SymbolTableRow};
use crate::frontend::ast::Type;


fn get_frame_size(function_id:&str, symbol_table:&SymbolTable) -> i32 {
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
                    _ => todo!()
                }
            },

            _ => {}
        }
    }

    frame_size
}


pub fn generate_mips(intermediate_code:Vec<IntermediateInstr>, symbol_table:SymbolTable, filename:&str) -> Result<(), Box<dyn Error>> {
    let mut file = OpenOptions::new().write(true).truncate(true).create(true).open(filename)?;
    let mut mips_instrs:Vec<String> = vec![];
    let mut curr_register = "$t0";

    mips_instrs.push("j main # start program execution\n\n".to_owned());

    for instr in intermediate_code {
        match instr {
            IntermediateInstr::FuncStart(name) => {
                let frame_size = get_frame_size(&name, &symbol_table);

                // add label for the function and size required for the stack local variables to the frame pointer
                mips_instrs.push(format!("{}: ", name));
                mips_instrs.push(format!("\taddiu $sp, $sp, -{}", frame_size));
                mips_instrs.push(format!("\tsw $fp, {}($sp)", frame_size - 4));
                mips_instrs.push("\tmove $fp, $sp\n".to_string());
            },

            IntermediateInstr::Push(_, var) => {
                match var {
                    Argument::Integer(value) => {
                        mips_instrs.push(format!("\tli {}, {:?}", curr_register, value));
                        if curr_register == "$t0" {
                            curr_register = "$t2";
                        } else {
                            curr_register = "$t0";
                        }
                    },

                    _ => todo!()
                }
            },

            IntermediateInstr::Return(return_type) => {
                match return_type {
                    Type::Integer => {
                        mips_instrs.push("\tmove $a0, $t0".to_owned());
                    },

                    _ => todo!()
                }
            }

            _ => {}
        }
    }

    mips_instrs.push("\n\tli $v0, 10 # halt syscall".to_owned());
    mips_instrs.push("\tsyscall".to_owned());

    file.write(mips_instrs.join("\n").as_bytes()).expect("Could not write target code to file");

    Ok(())
}
