mod parser;
mod semantics;

extern crate pest;
#[macro_use]
extern crate pest_derive;


fn main() {
    let filename = "idk.iri";
    let ast = parser::parse(filename).unwrap();
    let symbol_table = semantics::generate_symbol_table(ast);

    println!("{:#?}", symbol_table);
}
