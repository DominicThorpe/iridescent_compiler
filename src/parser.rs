use std::fs::OpenOptions;
use std::io::prelude::*;
use std::error::Error;
use pest::Parser;

use crate::ast::*;


#[derive(Parser)]
#[grammar = "grammar.pest"]
pub struct IridescentParser;


/**
 * Represents a symbol in the parser AST
 */
#[derive(Debug, Clone)]
struct Symbol {
    scope: usize,
}


/**
 * Used to assign preliminary scope to blocks so that the semantic analysis phase can more easily check if
 * data is in scope or not.
 */
#[derive(Debug)]
struct SymbolTable {
    entries: Vec<Symbol>
}

impl SymbolTable {
    /**
     * Gets the next scope ID, calculated as highest id currently in the table + 1
     */
    fn get_next_scope_id(&self) -> usize {
        let mut next:usize = 1;
        for symbol in &self.entries {
            if symbol.scope >= next {
                next = symbol.scope + 1;
            }
        }

        next
    }

    /**
     * Adds a row to the symbol table with the given ID
     */
    fn add(&mut self) -> usize {
        let scope_id = self.get_next_scope_id();
        self.entries.push(Symbol {
            scope: scope_id,
        });

        scope_id
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
 * Takes a `Pair` representing a value and returns it as a subtree of the AST, including children nodes.
 */
fn build_ast_from_value(pair: pest::iterators::Pair<Rule>) -> ASTNode {
    let mut parent = pair.clone().into_inner();
    let value = parent.next().unwrap();
    match value.as_rule() {
        Rule::int_literal => ASTNode::Value {
            literal_type: Type::Integer, 
            value: Literal::Integer(i16::try_from(get_int_from_str_literal(value.as_str())).ok().expect("Could not convert int literal to i16"))
        },

        Rule::bool_literal => ASTNode::Value {
            literal_type: Type::Boolean,
            value: Literal::Boolean(get_bool_from_str_literal(value.as_str()))
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


/**
 * Takes a `Pair` representing a function call and returns it as a subtree of the AST including chld nodes.
 */
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


/**
 * Takes a `Pair` representing a boolean term and returns a subtree of the AST including
 * children nodes.
 */
fn build_ast_from_boolean_term(pair: pest::iterators::Pair<Rule>) -> ASTNode {
    let mut parent = pair.into_inner();
    let token = parent.next().unwrap();

    let lhs = match token.as_rule() {
        Rule::term => build_ast_from_term(token),
        Rule::boolean_term => build_ast_from_boolean_term(token),
        unknown => panic!("Invalid token for boolean term: {:?}", unknown)
    };

    let mut operator:Option<BooleanOperator> = None;
    let mut rhs:Option<Box<ASTNode>> = None;
    match parent.peek() {
        Some(_) => {
            let token = parent.next().unwrap();
            match token.as_rule() {
                Rule::boolean_unary_operator => {
                    operator = Some(get_boolean_operator_from_str(token.as_str()));
                },
                Rule::term => {
                    rhs = Some(Box::new(build_ast_from_term(token)))
                },
                Rule::boolean_term => {
                    rhs = Some(Box::new(build_ast_from_boolean_term(token)))
                },
                unknown => panic!("Invalid token for boolean term: {:?}", unknown)
            }

            match parent.next() {
                Some(op) => {
                    match op.as_rule() {
                        Rule::boolean_binary_operator => {
                            operator = Some(get_boolean_operator_from_str(op.as_str()));
                        }
                        unknown => panic!("{:?} is not a valid binary boolean operator token", unknown)
                    }
                }

                None => {}
            }
        },

        None => {}
    };

    ASTNode::BooleanTerm {
        lhs: Box::new(lhs),
        rhs: rhs,
        operator: operator
    }
}


/**
 * Takes a `Pair` representing a boolean expression and returns a subtree of the AST including
 * children nodes.
 */
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
fn build_ast_from_if_stmt(pair: pest::iterators::Pair<Rule>, symbol_table: &mut SymbolTable) -> ASTNode {
    let mut parent = pair.into_inner();
    let boolean_expr = build_ast_from_boolean_expression(parent.next().unwrap());

    let mut statements = vec![];
    while let Some(statement) = parent.next() {
        statements.push(build_ast_from_statement(statement, symbol_table));
    }

    let scope = symbol_table.add();
    ASTNode::IfStatement {
        condition: Box::new(boolean_expr),
        statements: statements,
        scope: scope
    }
}


/**
 * Takes a `Pair` representing an else statement and returns it as a subtree of the AST, including 
 * children nodes.
 */
fn build_ast_from_else_stmt(pair: pest::iterators::Pair<Rule>, symbol_table: &mut SymbolTable) -> ASTNode {
    let mut parent = pair.into_inner();
    let mut statements = vec![];
    while let Some(statement) = parent.next() {
        statements.push(build_ast_from_statement(statement, symbol_table));
    }

    let scope = symbol_table.add();
    ASTNode::ElseStatement {
        statements: statements,
        scope: scope
    }
}


/**
 * Takes a `Pair` representing an if-else-if-else statement and returns it as a subtree of the AST, 
 * including children nodes.
 */
fn build_ast_from_if_structure(pair: pest::iterators::Pair<Rule>, symbol_table: &mut SymbolTable) -> ASTNode {
    let mut parent = pair.clone().into_inner();
    let mut statements = vec![];
    while let Some(token) = parent.next() {
        statements.push(match token.as_rule() {
            Rule::if_stmt => build_ast_from_if_stmt(token, symbol_table),
            Rule::elif_stmt => todo!(),
            Rule::else_stmt => build_ast_from_else_stmt(token, symbol_table),
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
fn build_ast_from_statement(pair: pest::iterators::Pair<Rule>, symbol_table: &mut SymbolTable) -> ASTNode {
    let mut parent = pair.clone().into_inner();
    let token = parent.next().unwrap();
    match token.as_rule() {
        Rule::return_stmt => build_ast_from_return_stmt(pair),
        Rule::var_decl => build_ast_from_var_decl_stmt(pair),
        Rule::var_assign => build_ast_from_var_assign_stmt(pair),
        Rule::if_structure => build_ast_from_if_structure(pair.into_inner().next().unwrap(), symbol_table),
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
fn build_ast_from_function(pair: pest::iterators::Pair<Rule>, symbol_table:&mut SymbolTable) -> ASTNode {
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

    let scope = symbol_table.add();
    while let Some(statement) = parent.next() {
        statements.push(build_ast_from_statement(statement, symbol_table));
    }

    ASTNode::Function {
        return_type: return_type,
        identifier: identifier,
        parameters: parameters,
        statements: statements,
        scope: scope
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
    let mut symbol_table = SymbolTable {entries: vec![]};
    for pair in pairs {
        match pair.as_rule() {
            Rule::function_decl => {
                ast.push(build_ast_from_function(pair, &mut symbol_table));
            },

            _ => {}
        }
    }

    Ok(ast)
}
