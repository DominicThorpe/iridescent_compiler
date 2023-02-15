mod parser;
mod semantics;
mod errors;
mod intermediate_gen;

extern crate pest;
#[macro_use]
extern crate pest_derive;

fn main() {
    let filename = "idk.iri";
    let ast = parser::parse(filename).unwrap();
    // println!("{:#?}", ast);
    let symbol_table = semantics::generate_symbol_table(ast.clone());
    // println!("{:#?}", symbol_table);
    semantics::semantic_validation(ast.clone(), &symbol_table).unwrap();
    let instructions = intermediate_gen::generate_program_intermediate(ast);

    for instr in instructions {
        println!("{:?}", instr);
    }
}
