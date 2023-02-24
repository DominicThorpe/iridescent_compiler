use std::fs::OpenOptions;
use std::io::prelude::*;
use std::error::Error;

use crate::frontend::intermediate_gen::{IntermediateInstr, Argument};
use crate::frontend::ast::Type;


fn generate_mips_preamble() -> Vec<String> {
    vec![
        "# initialise the stack memory",
        "li $v0, 9",
        "li $a0, 256 # size of the stack buffer",
        "syscall",
        "add $s6, $zero, $a0 # current position (top) in the stack",
        "add $s7, $zero, $a0 # start address of the stack\n",

        "j main # start program execution\n\n"
    ].into_iter().map(|s| s.to_owned()).collect()
}


pub fn generate_mips(intermediate_code:Vec<IntermediateInstr>, filename:&str) -> Result<(), Box<dyn Error>> {
    let mut file = OpenOptions::new().write(true).truncate(true).create(true).open(filename)?;
    let mut mips_instrs:Vec<String> = vec![];

    mips_instrs.append(&mut generate_mips_preamble());

    for instr in intermediate_code {
        match instr {
            IntermediateInstr::FuncStart(name) => {
                mips_instrs.push(format!("{}: ", name));
            },

            IntermediateInstr::Push(_, var) => {
                match var {
                    Argument::Integer(value) => {
                        mips_instrs.push(format!("\tli $t0, {:?}", value));
                    },

                    _ => todo!()
                }
            },

            IntermediateInstr::Return(return_type) => {
                match return_type {
                    Type::Integer => {
                        mips_instrs.push("\tadd $a0, $zero, $t0".to_owned());
                    },

                    _ => todo!()
                }
            }

            _ => {}
        }
    }

    mips_instrs.push("\tli $v0, 10".to_owned());
    mips_instrs.push("\tsyscall".to_owned());

    file.write(mips_instrs.join("\n").as_bytes()).expect("Could not write target code to file");

    Ok(())
}
