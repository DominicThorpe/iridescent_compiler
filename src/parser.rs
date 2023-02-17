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
    Integer(i16),
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
    Division,
    And,
    Or,
    XOr,
    LeftShiftLogical,
    LeftShiftArithmetic,
    RightShiftLogical
}


/**
 * Represents unary and binary boolean operators for use in boolean expressions and terms
 */
#[derive(PartialEq, Eq, Debug, Clone)]
pub enum BooleanOperator {
    Equal,
    NotEqual,
    Greater,
    GreaterOrEqual,
    Less,
    LessOrEqual,
    Invert
}


/**
 * Used to logically connect boolean terms and expressions
 */
#[derive(PartialEq, Eq, Debug, Clone)]
pub enum BooleanConnector {
    And,
    Or,
    XOr
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
        parameters: Vec<ASTNode>,
        statements: Vec<ASTNode>
    },

    Parameter {
        param_type: Type,
        identifier: String
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

    FunctionCall {
        identifier: String,
        arguments: Vec<ASTNode>
    },

    BooleanTerm {
        lhs: Box<ASTNode>,
        operator: Option<BooleanOperator>,
        rhs: Option<Box<ASTNode>>
    },

    BooleanExpression {
        lhs: Box<ASTNode>,
        operator: Option<BooleanOperator>,
        connector: Option<BooleanConnector>,
        rhs: Option<Box<ASTNode>>
    },

    IfElifElseStatement {
        statements: Vec<ASTNode>
    },

    IfStatement {
        condition: Box<ASTNode>,
        statements: Vec<ASTNode>
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
 * Takes a string representing a boolean operator and returns a `BooleanOperator` struct object
 * representing it.
 * 
 * ### Examples
 * `assert_eq!(">=", BooleanOperator::GreaterOrEqual)`
 */
fn get_boolean_operator_from_str(operator_str:&str) -> BooleanOperator {
    match operator_str {
        "==" => BooleanOperator::Equal,
        "!=" => BooleanOperator::NotEqual,
        ">" => BooleanOperator::Greater,
        ">=" => BooleanOperator::GreaterOrEqual,
        "<" => BooleanOperator::Less,
        "<=" => BooleanOperator::LessOrEqual,
        "!" => BooleanOperator::Invert,
        _ => panic!("Unknown boolean operator {}", operator_str)
    }
}


/**
 * Takes a string representing a boolean connector and returns a `BooleanConnector` struct object
 * representing it.
 * 
 * ### Examples
 * `assert_eq!("&&", BooleanConnector::And)`
 */
fn get_boolean_connector_from_str(connector_str:&str) -> BooleanConnector {
    match connector_str {
        "&&" => BooleanConnector::And,
        "||" => BooleanConnector::Or,
        "^^" => BooleanConnector::XOr,
        _ => panic!("Unknown boolean connector {}", connector_str)
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
        "&" => Operator::And,
        "|" => Operator::Or,
        "^" => Operator::XOr,
        ">>" => Operator::LeftShiftLogical,
        ">>>" => Operator::LeftShiftArithmetic,
        "<<" => Operator::RightShiftLogical,
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
            value: Literal::Integer(i16::try_from(get_int_from_str_literal(value.as_str())).ok().expect("Could not convert int literal to i16"))
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
        Rule::function_call => build_ast_from_function_call(child_token),
        Rule::expression => build_ast_from_expression(child_token),
        _ => panic!("Could not parse term {:?}", pair.as_str())
    };

    ASTNode::Term {
        child: Box::new(child)
    }
}


fn build_ast_from_function_call(pair: pest::iterators::Pair<Rule>) -> ASTNode {
    let mut parent = pair.clone().into_inner();
    let identifier = parent.next().unwrap().as_str().to_string();
    let arguments = match parent.next() {
        Some(args_list) => {
            let mut parent = args_list.into_inner();
            let mut args = vec![];
            while let Some(arg) = parent.next() {
                args.push(match arg.as_rule() {
                    Rule::identifier => build_ast_from_identifier(arg),
                    Rule::value => build_ast_from_value(arg),
                    _ => panic!("Could not parse argument {:?}", pair.as_str())
                });
            }

            args
        },
        None => vec![]
    };

    ASTNode::FunctionCall {
        identifier: identifier,
        arguments: arguments
    }
}


/**
 * Takes a `Pair` representing an expression and returns it as a subtree of the AST, including 
 * children nodes.
 */
fn build_ast_from_expression(pair: pest::iterators::Pair<Rule>) -> ASTNode {
    // get the left hand side of the expression from the first token
    let mut parent = pair.clone().into_inner();
    let child = parent.next().unwrap();
    let term = match child.as_rule() {
        Rule::term => build_ast_from_term(child),
        Rule::value => {
            ASTNode::Term {
                child: Box::new(build_ast_from_value(child))
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
    let mut parent = pair.clone().into_inner();
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


fn build_ast_from_boolean_term(pair: pest::iterators::Pair<Rule>) -> ASTNode {
    let mut parent = pair.into_inner();
    let token = parent.next().unwrap();

    let lhs = match token.as_rule() {
        Rule::term => build_ast_from_term(token),
        unknown => panic!("Invalid token for boolean term: {:?}", unknown)
    };

    ASTNode::BooleanTerm {
        lhs: Box::new(lhs),
        rhs: None,
        operator: None
    }
}


fn build_ast_from_boolean_expression(pair: pest::iterators::Pair<Rule>) -> ASTNode {
    let mut parent = pair.into_inner();
    let token = parent.next().unwrap();

    let lhs = Box::new(match token.as_rule() {
        Rule::boolean_expr => build_ast_from_boolean_expression(token),
        Rule::boolean_term => build_ast_from_boolean_term(token),
        Rule::term => build_ast_from_term(token),
        unknown => panic!("Invalid token for boolean expression: {:?}", unknown)
    });
    
    let mut connector:Option<BooleanConnector> = None;
    let mut operator:Option<BooleanOperator> = None;
    let rhs  = match parent.peek() {
        Some(_) => {
            let token = parent.next().unwrap();
            println!("B: {:?}", token);
            match token.as_rule() {
                Rule::boolean_expr => {
                    let operator_or_connector = parent.next().unwrap();
                    match operator_or_connector.as_rule() {
                        Rule::boolean_connector => {
                            connector = Some(get_boolean_connector_from_str(operator_or_connector.as_str()));
                        },

                        Rule::boolean_binary_operator => {
                            operator = Some(get_boolean_operator_from_str(operator_or_connector.as_str()))
                        },

                        unknown => panic!("Invalid token for boolean expression: {:?}", unknown)
                    }
                    Some(Box::new(build_ast_from_boolean_expression(token)))
                },
                
                Rule::boolean_term => {
                    let operator_or_connector = parent.next().unwrap();
                    match operator_or_connector.as_rule() {
                        Rule::boolean_connector => {
                            connector = Some(get_boolean_connector_from_str(operator_or_connector.as_str()));
                        },

                        Rule::boolean_binary_operator => {
                            operator = Some(get_boolean_operator_from_str(operator_or_connector.as_str()))
                        },

                        unknown => panic!("Invalid token for boolean expression: {:?}", unknown)
                    }
                    Some(Box::new(build_ast_from_boolean_term(token)))
                },

                Rule::boolean_unary_operator => {
                    operator = Some(get_boolean_operator_from_str(token.as_str()));                    
                    None
                },
                unknown => panic!("Invalid token for boolean expression: {:?}", unknown)
            }
        },

        None => None
    };

    ASTNode::BooleanExpression {
        lhs: lhs,
        rhs: rhs,
        connector: connector,
        operator: operator
    }
}


/**
 * Takes a `Pair` representing an if statement and returns it as a subtree of the AST, including 
 * children nodes.
 */
fn build_ast_from_if_stmt(pair: pest::iterators::Pair<Rule>) -> ASTNode {
    let mut parent = pair.into_inner();
    let boolean_expr = build_ast_from_boolean_expression(parent.next().unwrap());

    let mut statements = vec![];
    while let Some(statement) = parent.next() {
        statements.push(build_ast_from_statement(statement));
    }

    ASTNode::IfStatement {
        condition: Box::new(boolean_expr),
        statements: statements
    }
}


/**
 * Takes a `Pair` representing an if-else-if-else statement and returns it as a subtree of the AST, 
 * including children nodes.
 */
fn build_ast_from_if_structure(pair: pest::iterators::Pair<Rule>) -> ASTNode {
    let mut parent = pair.clone().into_inner();
    let mut statements = vec![];
    while let Some(token) = parent.next() {
        statements.push(match token.as_rule() {
            Rule::if_stmt => build_ast_from_if_stmt(token),
            Rule::elif_stmt => todo!(),
            Rule::else_stmt => todo!(),
            unknown => panic!("Invalid token for if statement: {:?}", unknown)
        });
    }

    ASTNode::IfElifElseStatement {
        statements: statements
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
        Rule::if_structure => build_ast_from_if_structure(pair.into_inner().next().unwrap()),
        Rule::function_call => build_ast_from_function_call(pair.into_inner().next().unwrap()),
        _ => panic!("Could not parse statement {:?}", pair.as_str())
    }
}


/**
 * Takes a `Pair` representing a parameter and returns it as a subtree of the AST, including children nodes.
 */
fn build_ast_from_param(pair: pest::iterators::Pair<Rule>) -> ASTNode {
    let mut param = pair.into_inner();
    let param_type = get_type_from_string(param.next().unwrap().as_str());
    let param_identifier = param.next().unwrap().as_str().to_owned();
    ASTNode::Parameter {
        param_type: param_type,
        identifier: param_identifier
    }
}


/**
 * Takes a `Pair` representing a function and returns it as a subtree of the AST, including children nodes.
 */
fn build_ast_from_function(pair: pest::iterators::Pair<Rule>) -> ASTNode {
    let mut parent = pair.into_inner();
    let return_type = get_type_from_string(parent.next().unwrap().as_str());
    let identifier = parent.next().unwrap().as_str().to_owned();
    let mut parameters = vec![];
    let mut statements = vec![];

    match parent.peek().unwrap().as_rule() {
        Rule::param_list => {
            let mut param_list_parent = parent.next().unwrap().into_inner();
            while let Some(param) = param_list_parent.next() {
                parameters.push(build_ast_from_param(param));
            }
        },

        _ => {}
    }

    while let Some(statement) = parent.next() {
        statements.push(build_ast_from_statement(statement));
    }

    ASTNode::Function {
        return_type: return_type,
        identifier: identifier,
        parameters: parameters,
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
