mod parser;

extern crate pest;
#[macro_use]
extern crate pest_derive;


fn main() {
    let filename = "test.iri";
    let ast = parser::parse(filename).unwrap();

    println!("{:#?}", ast);
}
