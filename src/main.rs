mod frontend;
mod backend;
mod errors;

extern crate pest;
#[macro_use]
extern crate pest_derive;
use std::env;

fn main() {
    let cmd_args:Vec<String> = env::args().collect();

    let filename = &cmd_args[1];
    let output_name = format!("{}.asm", &cmd_args[2]);
    if !filename.ends_with(".iri") {
        panic!("Input filename must have the .iri file extension");
    }

    println!("Compiling {} into {}", filename, &cmd_args[2]);
    let ast = frontend::parser::parse(filename).unwrap();
    // println!("{:#?}\n\n\n", ast);
    let symbol_table = frontend::semantics::generate_symbol_table(ast.clone());
    println!("{:#?}", symbol_table);
    frontend::semantics::semantic_validation(ast.clone(), &symbol_table).unwrap();
    let instructions = frontend::intermediate_gen::generate_program_intermediate(ast, &symbol_table);

    for instr in &instructions {
        println!("{}", instr);
    }

    match &*cmd_args[3] {
        "-mips" => backend::mips::generate_mips(instructions, &output_name, &symbol_table).unwrap(),
        "-ird" => panic!("Iridium architecture compilation is not yet supported"),
        "-x64" => panic!("The x86-64 architecture compilation is not yet supported"),
        option => panic!("{} is not a valid target code flag", option)
    }
}
