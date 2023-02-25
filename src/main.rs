mod frontend;
mod backend;
mod errors;

extern crate pest;
#[macro_use]
extern crate pest_derive;

fn main() {
    let filename = "idk.iri";
    let ast = frontend::parser::parse(filename).unwrap();
    // println!("{:#?}\n\n\n", ast);
    let symbol_table = frontend::semantics::generate_symbol_table(ast.clone());
    println!("{:#?}", symbol_table);
    frontend::semantics::semantic_validation(ast.clone(), &symbol_table).unwrap();
    let instructions = frontend::intermediate_gen::generate_program_intermediate(ast);

    for instr in &instructions {
        println!("{}", instr);
    }

    backend::mips::generate_mips(instructions, symbol_table, "mips.asm").unwrap();
}
