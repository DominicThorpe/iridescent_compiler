mod parser;

extern crate pest;
#[macro_use]
extern crate pest_derive;


fn main() {
    let filename = "idk.iri";
    let ast = parser::parse(filename).unwrap();

    println!("{:#?}", ast);
}
