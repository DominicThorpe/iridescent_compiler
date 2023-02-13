use std::fs::OpenOptions;
use std::io::prelude::*;
use std::error::Error;
use pest::Parser;


#[derive(Parser)]
#[grammar = "grammar.pest"]
pub struct IridescentParser;


/**
 * Represents all the currently implemented primitive datatypes.
 */
#[derive(PartialEq, Eq, Debug, Clone)]
pub enum Type {
    Void,
    Integer
}


/**
 * Represents a literal of any primitive datatype.
 */
#[derive(PartialEq, Eq, Debug, Clone)]
pub enum Literal {
    Integer(i64),
}


/**
 * Represents unary and binary operators.
 */
#[derive(PartialEq, Eq, Debug, Clone)]
pub enum Operator {
    NumericNegation,
    LogicalNegation,
    Complement
}


/**
 * Represents a node in the AST, including information about the node such as:
 *  - identifier
 *  - literal value
 *  - datatype
 */
#[derive(PartialEq, Eq, Debug, Clone)]
pub enum ASTNode {
    Function {
        return_type: Type,
        identifier: String,
        statements: Vec<ASTNode>
    },

    ReturnStatement {
        expression: Box<ASTNode>
    },

    Expression {
        lhs: Box<ASTNode>,
        operator: Option<Operator>
    },

    Term {
        child: Box<ASTNode>
    },
    
    Value {
        literal_type: Type,
        value: Literal
    }
}


/**
 * Takes a string representing a primitive type and returns `Type` struct object representing it.
 * 
 * ### Examples
 * `assert_eq!("int", Type::Integer)`
 * 
 * `assert_eq!("void", Type::Void)`
 */
fn get_type_from_string(type_str:&str) -> Type {
    match type_str {
        "void" => Type::Void,
        "int" => Type::Integer,
        _ => panic!("Unknown type {}", type_str)
    } 
}


/**
 * Takes a string representing an operator and returns an `Operator` struct object 
 * representing it.
 * 
 * ### Examples
 * `assert_eq!("!", Type::LogicalNegation)`
 */
fn get_operator_from_str(operator_str:&str) -> Operator {
    match operator_str {
        "!" => Operator::LogicalNegation,
        "-" => Operator::NumericNegation,
        "~" => Operator::Complement,
        _ => panic!("Unknown operator {}", operator_str)
    }
}


/**
 * Takes a string representing a path to a file and returns the contents of the file as a `String`. Will
 * return an error if the file cannot be opened or read.
 * 
 * ### Examples
 * `let contents:String = get_file_contents("hello_world.iri")`
 */
fn get_file_contents(filename:&str) -> Result<String, Box<dyn Error>> {
    let mut file = OpenOptions::new().read(true).open(filename)?;
    let mut contents = String::new();
    file.read_to_string(&mut contents)?;

    Ok(contents)
}


/**
 * Takes a string representing a number in decimal, binary (prefix "0b"), or hexadecimal (prefix "0x") and
 * returns the corresponding number.
 * 
 * ### Examples
 * `assert_eq!(get_int_from_str_literal("0xFA"), 250);`
 * 
 * `assert_eq!(get_int_from_str_literal("0b1101"), 13);`
 * 
 * `assert_eq!(get_int_from_str_literal("20"), 20);`
 */
fn get_int_from_str_literal(literal:&str) -> i64 {
    if literal.starts_with("0b") {
        return i64::from_str_radix(literal, 2).unwrap();
    } else if literal.starts_with("0x") {
        return i64::from_str_radix(literal, 16).unwrap();
    } else {
        return literal.parse().unwrap();
    }
}


/**
 * Takes a `Pair` representing a value and returns it as a subtree of the AST, including children nodes.
 */
fn build_ast_from_value(pair: pest::iterators::Pair<Rule>) -> ASTNode {
    let mut parent = pair.clone().into_inner();
    let value = parent.next().unwrap();
    match value.as_rule() {
        Rule::int_literal => ASTNode::Value{
            literal_type: Type::Integer, 
            value: Literal::Integer(get_int_from_str_literal(value.as_str()))
        },

        _ => panic!("Could not parse value {:?}", pair.as_str())
    }
}


/**
 * Takes a `Pair` representing a term and returns it as a subtree of the AST, including children nodes.
 */
fn build_ast_from_term(pair: pest::iterators::Pair<Rule>) -> ASTNode {
    let mut parent = pair.clone().into_inner();
    let val_or_expr = parent.next().unwrap();
    let child = match val_or_expr.as_rule() {
        Rule::value => build_ast_from_value(val_or_expr),
        Rule::expression => build_ast_from_expression(val_or_expr),
        _ => panic!("Could not parse term {:?}", pair.as_str())
    };

    ASTNode::Term {
        child: Box::new(child)
    }
}


/**
 * Takes a `Pair` representing an expression and returns it as a subtree of the AST, including 
 * children nodes.
 */
fn build_ast_from_expression(pair: pest::iterators::Pair<Rule>) -> ASTNode {
    let mut parent = pair.clone().into_inner();
    let value_or_expr = parent.next().unwrap();
    let term = match value_or_expr.as_rule() {
        Rule::term => build_ast_from_term(value_or_expr),
        Rule::value => {
            ASTNode::Term {
                child: Box::new(build_ast_from_value(value_or_expr))
            }
        },
        _ => panic!("Could not parse expression {:?}", pair.as_str())
    };
    
    let operator = match parent.next() {
        Some(token) => Some(get_operator_from_str(token.as_str())),
        None => None
    };

    ASTNode::Expression {
        lhs: Box::new(term),
        operator: operator
    }
}


/**
 * Takes a `Pair` representing a return statement and returns it as a subtree of the AST, including 
 * children nodes.
 */
fn build_ast_from_return_stmt(pair: pest::iterators::Pair<Rule>) -> ASTNode {
    let mut parent = pair.clone().into_inner().next().unwrap().into_inner();
    let expression = build_ast_from_expression(parent.next().unwrap());

    ASTNode::ReturnStatement {
        expression: Box::new(expression)
    }
}


/**
 * Takes a `Pair` representing a statement and dispatches it to the relevant AST builder function.
 */
fn build_ast_from_statement(pair: pest::iterators::Pair<Rule>) -> ASTNode {
    let mut parent = pair.clone().into_inner();
    let token = parent.next().unwrap();
    match token.as_rule() {
        Rule::return_stmt => build_ast_from_return_stmt(pair),
        _ => panic!("Could not parse statement {:?}", pair.as_str())
    }
}


/**
 * Takes a `Pair` representing a function and returns it as a subtree of the AST, including children nodes.
 */
fn build_ast_from_function(pair: pest::iterators::Pair<Rule>) -> ASTNode {
    let mut parent = pair.into_inner();
    let return_type = get_type_from_string(parent.next().unwrap().as_str());
    let identifier = parent.next().unwrap().as_str().to_owned();
    let mut statements = vec![];

    while let Some(statement) = parent.next() {
        statements.push(build_ast_from_statement(statement));
    }

    ASTNode::Function {
        return_type: return_type,
        identifier: identifier,
        statements: statements
    }
}


/**
 * Takes a filename and returns a vector of `ASTNode` structs which represent the AST subtrees of the
 * top-level nodes in the Iridescent AST, such as function declarations, struct definitions, and 
 * include statements.
 */
pub fn parse(filename:&str) -> Result<Vec<ASTNode>, Box::<dyn Error>> {
    let program_text = get_file_contents(filename)?;
    let mut ast = vec![];

    // get the pairs and skip the program node
    let pairs = IridescentParser::parse(Rule::program, program_text.as_str())?
                                        .next().unwrap().into_inner();
    for pair in pairs {
        match pair.as_rule() {
            Rule::function_decl => {
                ast.push(build_ast_from_function(pair));
            },

            _ => {}
        }
    }

    Ok(ast)
}
