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
    NegateNumerical,
    NegateLogical,
    Complement,
    Addition,
    Subtraction,
    Multiplication,
    Division
}


/**
 * Represents the mutability of a variable.
 */
#[derive(PartialEq, Eq, Debug, Clone)]
pub enum Mutability {
    Mutable,
    Constant
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

    VarDeclStatement {
        var_type: Type,
        mutability: Mutability,
        identifier: String,
        value: Box<ASTNode>
    },

    VarAssignStatement {
        identifier: String,
        value: Box<ASTNode>
    },

    Expression {
        lhs: Box<ASTNode>,
        operator: Option<Operator>,
        rhs: Option<Box<ASTNode>>
    },

    Term {
        child: Box<ASTNode>
    },
    
    Value {
        literal_type: Type,
        value: Literal
    },

    Identifier(String)
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
 * Takes a string representing a unary operator and returns an `Operator` struct object 
 * representing it.
 * 
 * ### Examples
 * `assert_eq!("!", Type::LogicalNegation)`
 */
fn get_unary_operator_from_str(operator_str:&str) -> Operator {
    match operator_str {
        "!" => Operator::NegateLogical,
        "-" => Operator::NegateNumerical,
        "~" => Operator::Complement,
        _ => panic!("Unknown operator {}", operator_str)
    }
}


/**
 * Takes a string representing a unary operator and returns an `Operator` struct object 
 * representing it.
 * 
 * ### Examples
 * `assert_eq!("+", Type::Addition)`
 * 
 * `assert_eq!("-", Type::Subtraction)`
 */
fn get_binary_operator_from_str(operator_str:&str) -> Operator {
    match operator_str {
        "+" => Operator::Addition,
        "-" => Operator::Subtraction,
        "*" => Operator::Multiplication,
        "/" => Operator::Division,
        _ => panic!("Unknown operator {}", operator_str)
    }
}


/**
 * Takes a string representing a mutability modifier of mutable or constant and returns the corresponding
 * representation from the `Mutability` enum.
 * 
 * ### Examples
 * `assert_eq!("mut", Mutability::Mutabile)`
 * 
 * `assert_eq!("const", Mutability::Constant)`
 */
fn get_mutability_from_str(mutability_str:&str) -> Mutability {
    match mutability_str {
        "mut" => Mutability::Mutable,
        "const" => Mutability::Constant,
        _ => panic!("Unknown mutability modifier {}", mutability_str)
    }
}


/**
 * Takes a `Pair` representing an expression or a term and returns an `Expression` struct representing
 * that pair and its children. If the pair is a term, then it will be made the single child of a new
 * `Expression` node.
 */
fn get_expr_from_expr_or_term(pair: pest::iterators::Pair<Rule>) -> ASTNode {
    match pair.as_rule() {
        Rule::expression => build_ast_from_expression(pair),
        Rule::term => {
            ASTNode::Expression {
                lhs: Box::new(build_ast_from_term(pair)),
                operator: None,
                rhs: None
            }
        },
        _ => panic!("Could not parse expression {:?}", pair.as_str())
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
        return i64::from_str_radix(&literal[2..], 2).unwrap();
    } else if literal.starts_with("0x") {
        return i64::from_str_radix(&literal[2..], 16).unwrap();
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


fn build_ast_from_identifier(pair: pest::iterators::Pair<Rule>) -> ASTNode {
    ASTNode::Identifier(pair.as_str().to_string())
}


/**
 * Takes a `Pair` representing a term and returns it as a subtree of the AST, including children nodes.
 */
fn build_ast_from_term(pair: pest::iterators::Pair<Rule>) -> ASTNode {
    let mut parent = pair.clone().into_inner();
    let child_token = parent.next().unwrap();
    let child = match child_token.as_rule() {
        Rule::value => build_ast_from_value(child_token),
        Rule::identifier => build_ast_from_identifier(child_token),
        Rule::expression => build_ast_from_expression(child_token),
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
    // get the left hand side of the expression from the first token
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
    
    // get the operator and right hand side of the expression if they exist
    let lhs:Box<ASTNode> = Box::new(term);
    let operator:Option<Operator>;
    let mut rhs:Option<Box<ASTNode>> = None;

    // get the operator if there is one from the 2nd child token of the expression if the operator is unary, or 
    // the 3rd if it is a binary expression
    operator = match parent.next() { 
        Some(token) => {
            match token.as_rule() {
                Rule::unary_operator => Some(get_unary_operator_from_str(token.as_str())),
                Rule::term => { // get the right hand side if there is one from the 2nd child of the expression
                    rhs = Some(Box::new(build_ast_from_term(token)));
                    Some(get_binary_operator_from_str(parent.next().unwrap().as_str()))
                }

                _ => panic!("Could not parse expression {:?}", pair.as_str())
            }
        },

        None => None
    };

    // build and return the expression node
    ASTNode::Expression {
        lhs: lhs,
        operator: operator,
        rhs: rhs
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
 * Takes a `Pair` representing a variable declaration statement and returns it as a subtree of the AST, 
 * including children nodes.
 */
fn build_ast_from_var_decl_stmt(pair: pest::iterators::Pair<Rule>) -> ASTNode {
    let mut parent = pair.clone().into_inner().next().unwrap().into_inner();
    let mutability = match parent.peek().unwrap().as_rule() {
        Rule::mutability_mod => get_mutability_from_str(parent.next().unwrap().as_str()),
        Rule::primitive_type => Mutability::Constant,
        _ => panic!("Could not parse variable declaration {:?}", pair.as_str())
    };

    let var_type = get_type_from_string(parent.next().unwrap().as_str());
    let identifier = parent.next().unwrap().as_str().to_string();

    let value_token = parent.next().unwrap();
    let value = get_expr_from_expr_or_term(value_token);

    ASTNode::VarDeclStatement {
        var_type: var_type,
        mutability: mutability,
        identifier: identifier,
        value: Box::new(value)
    }
}


/**
 * Takes a `Pair` representing a variable assignment statement and returns it as a subtree of the AST, 
 * including children nodes.
 */
fn build_ast_from_var_assign_stmt(pair: pest::iterators::Pair<Rule>) -> ASTNode {
    let mut parent = pair.clone().into_inner().next().unwrap().into_inner();
    let identifier = parent.next().unwrap().as_str().to_string();

    let value_token = parent.next().unwrap();
    let value = get_expr_from_expr_or_term(value_token);
    
    ASTNode::VarAssignStatement {
        identifier: identifier,
        value: Box::new(value)
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
        Rule::var_decl => build_ast_from_var_decl_stmt(pair),
        Rule::var_assign => build_ast_from_var_assign_stmt(pair),
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
