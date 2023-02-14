mod parser;
mod semantics;
mod errors;

extern crate pest;
#[macro_use]
extern crate pest_derive;


fn main() {
    let filename = "idk.iri";
    let ast = parser::parse(filename).unwrap();
    let symbol_table = semantics::semantic_analysis(ast);
    println!("{:#?}", symbol_table);
}
