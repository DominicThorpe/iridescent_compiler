use crate::parser::{Type, ASTNode, Mutability};
use crate::errors::*;

use std::error::Error;


/**
 * Represents the symbol table which is used to track variables and functions during semantic analysis
 * and code generation.
 */
#[derive(Clone, Debug)]
pub struct SymbolTable {
    rows: Vec<SymbolTableRow>
}

impl SymbolTable {
    /**
     * Adds a row to the symbol table. Will panic if a duplicate identifier in an overlapping scope 
     * is found.
     */
    fn add(&mut self, new_row:SymbolTableRow) {
        let new_identifier = new_row.get_identifier();
        for row in &self.rows {
            if row.get_identifier() == new_row.get_identifier()  && 
                    row.get_parent_identifier() == new_row.get_parent_identifier() {
                panic!("Duplicate identifier {} detected", new_identifier);
            }
        }

        self.rows.push(new_row);
    }


    /**
     * Finds the highest numbered scope ID in the whole symbol table and returns that ID + 1 as the next
     * ID to be assigned.
     */
    fn get_next_scope_id(&self) -> usize {
        let mut max_id:usize = 1;
        for row in &self.rows {
            if row.get_scope_id() >= max_id {
                max_id = row.get_scope_id() + 1;
            }
        }

        max_id
    }


    /**
     * Takes an identifier and an array of the scopes containing the symbol starting broad and moving down, and returns 
     * the scope of the symbol if the identifier is in scope, and an Error if not.
     */
    fn get_identifier_in_scope(&self, identifier:&str, scope_history:&Vec<usize>) -> Result<usize, Box<dyn Error>> {
        for row in &self.rows {
            if row.get_identifier() == identifier && scope_history.contains(&row.get_parent_scope_id()) {
                return Ok(row.get_scope_id());
            }
        }

        Err(Box::new(SymbolNotFoundError(identifier.to_owned())))
    }


    /**
     * Takes an identifier and an array of the scopes as in get_identifier_in_scope(), and returns the type or return type 
     * of the symbol if the identifier is in scope, and an Error if not.
     */
    fn get_identifier_type_in_scope(&self, identifier:&str, scope_history:&Vec<usize>) -> Result<Type, Box<dyn Error>> {
        for row in &self.rows {
            if row.get_identifier() == identifier && scope_history.contains(&row.get_parent_scope_id()) {
                return Ok(row.get_scope_type());
            }
        }

        Err(Box::new(SymbolNotFoundError(identifier.to_owned())))
    }


    /**
     * Takes an identifier and an array of the scopes as in get_identifier_in_scope(), and returns the mutability of the 
     * symbol if the identifier is in scope, and an Error if not.
     */
    fn get_mutability_in_scope(&self, identifier:&str, scope_history:&Vec<usize>) -> Result<Mutability, Box<dyn Error>> {
        for row in &self.rows {
            if row.get_identifier() == identifier && scope_history.contains(&row.get_parent_scope_id()) {
                return Ok(row.get_mutability());
            }
        }

        Err(Box::new(SymbolNotFoundError(identifier.to_owned())))
    }


    /**
     * Takes an identifier of a function and returns a vector of the types of the parameters of that function. Returns
     * an error if the identifier was not found or was a variable.
     */
    fn get_function_param_types(&self, identifier:&String) -> Result<Vec<Type>, Box<dyn Error>> {
        for row in &self.rows {
            if &row.get_identifier() == identifier {
                match row {
                    SymbolTableRow::Variable {..} => {
                        return Err(Box::new(IncorrectDatatype))
                    },

                    SymbolTableRow::ScopeBlock {..} => {
                        return Err(Box::new(IncorrectDatatype))
                    },
                    
                    SymbolTableRow::Function {parameters, ..} => {
                        return Ok(parameters.clone())
                    }
                }
            }
        }

        Err(Box::new(SymbolNotFoundError(identifier.to_owned())))
    }
}


/**
 * Represents a single entry in the symbol table represented by `SymbolTable`. It contains information about
 * the datatypes of variables, identifiers, scopse, and more.
 */
#[derive(Clone, Debug)]
pub enum SymbolTableRow {
    Variable {
        identifier: String,
        primitive_type: Type,
        mutability: Mutability,
        parent_scope: usize,
        parent: Box<SymbolTableRow>
    },

    Function {
        identifier: String,
        return_type: Type,
        parameters: Vec<Type>,
        scope: usize,
        parent_scope: usize
    },

    ScopeBlock {
        identifier: String,
        scope: usize,
        parent_scope: usize,
        parent: Box<SymbolTableRow>
    }
}

impl SymbolTableRow {
    /**
     * Returns the identifier of the symbol table row
     */
    fn get_identifier(&self) -> String {
        match self {
            SymbolTableRow::Function {identifier, ..} => identifier.to_string(),
            SymbolTableRow::Variable {identifier, ..} => identifier.to_string(),
            SymbolTableRow::ScopeBlock {identifier, ..} => identifier.to_string(),
        }
    }


    /**
     * Returns the ID of the scope of the symbol
     */
    fn get_scope_id(&self) -> usize {
        match self {
            SymbolTableRow::Function {scope, ..} => *scope,
            SymbolTableRow::Variable {parent_scope, ..} => *parent_scope,
            SymbolTableRow::ScopeBlock {scope, ..} => *scope
        }
    }


    /**
     * Returns the type most appropriate to the entry in question: variable type or return type
     */
    fn get_scope_type(&self) -> Type {
        match self {
            SymbolTableRow::Function {return_type, ..} => return_type.clone(),
            SymbolTableRow::Variable {primitive_type, ..} => primitive_type.clone(),
            SymbolTableRow::ScopeBlock {..} => panic!("Cannot get type of scope block"),
        }
    }


    /**
     * Returns the mutability of the symbol, which is always `Constant` for a function
     */
    fn get_mutability(&self) -> Mutability {
        match self {
            SymbolTableRow::Function {..} => Mutability::Constant,
            SymbolTableRow::Variable {mutability, ..} => mutability.clone(),
            SymbolTableRow::ScopeBlock {..} => panic!("Cannot get mutability of scope block"),
        }
    }


    /**
     * Returns the ID of the scope of the symbol's parent
     */
    fn get_parent_scope_id(&self) -> usize {
        match self {
            SymbolTableRow::Function {parent_scope, ..} => *parent_scope,
            SymbolTableRow::Variable {parent_scope, ..} => *parent_scope,
            SymbolTableRow::ScopeBlock {parent_scope, ..} => *parent_scope
        }
    }


    /**
     * Gets the identifier of the symbol table row if this row is a function, or the identifier of the
     * parent function if the row is a variable.
     */
    fn get_parent_identifier(&self) -> String {
        match self {
            SymbolTableRow::Function {..} => "global".to_string(),
            SymbolTableRow::Variable {parent, ..} => parent.get_identifier().to_string(),
            SymbolTableRow::ScopeBlock {parent, ..} => parent.get_identifier().to_string(),
        }
    }
}


/**
 * Takes an `ASTNode` struct and either generates a row for the symbol table, which is passed by
 * reference, or calls itself recursively on each of that row's children to generate additional 
 * rows for them.
 */
fn generate_sub_symbol_table(subtree:ASTNode, table:&mut SymbolTable, parent:Option<SymbolTableRow>) {
    match subtree.clone() {
        ASTNode::Function {return_type, identifier, statements, parameters, scope} => {
            let param_types = parameters.clone().into_iter().map(|param| {
                match param {
                    ASTNode::Parameter {param_type, ..} => param_type,
                    unknown => panic!("{:?} is not a valid parameter in function call {}", unknown, identifier) 
                }
            }).collect();

            let function_row = SymbolTableRow::Function {
                identifier: identifier,
                return_type: return_type,
                parameters: param_types,
                parent_scope: 0,
                scope: scope
            };
            table.add(function_row.clone());

            for param in parameters {
                generate_sub_symbol_table(param, table, Some(function_row.clone()));
            }

            for statement in statements {
                generate_sub_symbol_table(statement, table, Some(function_row.clone()));
            }
        },

        ASTNode::Parameter {param_type, identifier} => {
            table.add(
                SymbolTableRow::Variable {
                    identifier: identifier,
                    primitive_type: param_type,
                    mutability: Mutability::Constant,
                    parent_scope: parent.clone().unwrap().get_scope_id(),
                    parent: Box::new(parent.expect(&format!("Statement {:?} does not have a parent.", subtree)))
                }
            )
        }

        ASTNode::VarDeclStatement {var_type, mutability, identifier, ..} => {
            table.add(
                SymbolTableRow::Variable {
                    identifier: identifier,
                    primitive_type: var_type,
                    mutability: mutability,
                    parent_scope: parent.clone().unwrap().get_scope_id(),
                    parent: Box::new(parent.expect(&format!("Statement {:?} does not have a parent.", subtree)))
                }
            )
        },

        ASTNode::IfElifElseStatement {statements} => {
            for statement in statements {
                generate_sub_symbol_table(statement, table, parent.clone());
            }
        },

        ASTNode::IfStatement {statements, scope, ..} => {
            let scope_id = table.get_next_scope_id();
            let parent_struct = parent.clone().unwrap();
            let new_row = SymbolTableRow::ScopeBlock {
                identifier: format!("{}_{}", parent_struct.get_identifier(), scope_id),
                parent_scope: parent_struct.get_scope_id(),
                scope: scope,
                parent: Box::new(parent_struct)
            };

            table.add(new_row.clone());

            for statement in statements {
                generate_sub_symbol_table(statement, table, Some(new_row.clone()));
            }
        },

        _ => {}
    };
}


/**
 * Verifies that the given expression node has a child of the correct type
 */
fn validate_term_of_type(node:&ASTNode, required_type:&Type, symbol_table:&SymbolTable, scope_history:&Vec<usize>) -> Result<(), Box<dyn Error>> {
    match node {
        ASTNode::Term { child } => {
            match &**child {
                ASTNode::Expression {..} => {
                    match validate_expression_of_type(&*child, &required_type, symbol_table, scope_history) {
                        Ok(_) => {},
                        Err(_) => {
                            return Err(Box::new(IncorrectDatatype)); 
                        }
                    }
                },

                ASTNode::Value {literal_type, ..} => {
                    if literal_type != required_type {
                        return Err(Box::new(IncorrectDatatype));
                    }
                },

                ASTNode::Identifier(identifier) => {
                    if &symbol_table.get_identifier_type_in_scope(identifier, scope_history)? != required_type {
                        return Err(Box::new(IncorrectDatatype));
                    }
                },

                ASTNode::FunctionCall {identifier, ..} => {
                    if &symbol_table.get_identifier_type_in_scope(identifier, &vec![0]).unwrap() != required_type {
                        return Err(Box::new(IncorrectDatatype));
                    }
                },

                _ => panic!("{:?} is not a valid token for semantic analysis of terms.", node)
            }
        },

        _ => panic!("{:?} is not a valid token for semantic analysis of terms.", node)
    };

    Ok(())
}


/**
 * Takes an expression node and uses recursion to verify that the result of the expression is
 * semantically valid (i.e. everything is of the same datatype and datatype is valid for the 
 * operation) - otherwise will return an Error.
 */
fn validate_expression_of_type(node:&ASTNode, required_type:&Type, symbol_table:&SymbolTable, scope_history:&Vec<usize>) -> Result<(), Box<dyn Error>> {
    match &node {
        ASTNode::Expression {lhs, rhs, operator} => {
            validate_term_of_type(lhs, required_type, symbol_table, &scope_history)?;
            match &rhs {
                None => {},
                Some(term) => {
                    validate_term_of_type(term, required_type, symbol_table, &scope_history)?;
                }
            }

            // check that operator arg types are valid for operator (e.g. cannot do true - false or "hello" / "world")
            // we already have validated that the args are the "required_type"
            match operator {
                None => {},
                Some(op) => {
                    match required_type {
                        Type::Boolean => panic!("{:?} is not a valid operator for arguments of type {:?}", op, required_type),
                        _ => {}
                    }
                } 
            }
        },

        _ => panic!("{:?} is not an expression", node)
    };

    Ok(())
}


/**
 * Takes an AST node and runs semantic analysis on it to ensure it is valid when the context of the whole program
 * is taken into consideration.
 */
fn semantic_validation_subtree(node:&ASTNode, symbol_table:&SymbolTable, scope_history:&Vec<usize>) -> Result<(), Box<dyn Error>> {
    let mut scope_history = scope_history.clone();
    match node {
        ASTNode::Function {identifier, statements, return_type, ..} => {
            let mut has_return = false;
            for statement in statements {
                scope_history.push(symbol_table.get_identifier_in_scope(&identifier, &scope_history)?);
                semantic_validation_subtree(statement, &symbol_table, &scope_history)?;

                match statement.clone() {
                    ASTNode::ReturnStatement { expression } => {
                        validate_expression_of_type(&expression, &return_type, symbol_table, &scope_history)?;
                        has_return = true;
                    },

                    ASTNode::FunctionCall {identifier, arguments} => {
                        let param_types = symbol_table.get_function_param_types(&identifier)?;
                        let arg_types:Vec<Type> = arguments.into_iter().map(|param|
                            match param {
                                ASTNode::Value {literal_type, ..} => literal_type, 
                                ASTNode::Identifier(identifier) => symbol_table.get_identifier_type_in_scope(&identifier, &scope_history).unwrap(),
                                unknown => panic!("{:?} is not a valid parameter in function call {}", unknown, identifier) 
                            }
                        ).collect();

                        if arg_types.len() != param_types.len() {
                            return Err(Box::new(IncorrectNumArguments(identifier)));
                        }

                        for i in 0..arg_types.len() {
                            if param_types[i] != arg_types[i] {
                                return Err(Box::new(IncorrectDatatype));
                            }
                        }
                    }

                    _ => {}
                }
            }

            if return_type != &Type::Void && !has_return {
                return Err(Box::new(BadFunctionReturn(identifier.to_string())));
            }
        },

        ASTNode::VarDeclStatement {var_type, value, ..} => {
            validate_expression_of_type(&value, &var_type, symbol_table, &scope_history)?;
        }
        
        ASTNode::VarAssignStatement {identifier, value} => {
            if symbol_table.get_mutability_in_scope(&identifier, &scope_history)? != Mutability::Mutable {
                return Err(Box::new(ImmutableReassignmentError(identifier.to_string())));
            }

            symbol_table.get_identifier_in_scope(&identifier, &scope_history)?;
            let var_type = symbol_table.get_identifier_type_in_scope(&identifier, &scope_history)?;
            validate_expression_of_type(&value, &var_type, symbol_table, &scope_history)?;
        },

        ASTNode::IfElifElseStatement {statements} => {
            for statement in statements {
                match statement {
                    ASTNode::IfStatement {statements, scope, ..}=> {
                        for sub_stmt in statements {
                            scope_history.push( *scope );
                            semantic_validation_subtree(sub_stmt, symbol_table, &scope_history).unwrap();
                        }
                    },

                    _ => panic!("Invalid block if if, else if, else structure {:?}", statement)
                }
            }
        }

        _ => {}
    }

    Ok(())
}


/**
 * Takes the root node of the AST and runs semantic analysis, checking for:
 *   - undeclared/out of scope variables
 *   - no/incorrect return statements
 *   - reassignment to immutable variable
 *   - operations on non-matching datatypes
 *   - functions with incorrect return types
 *   - incorrect arguments to function calls
 * 
 * TODO:
 *   - check validity of boolean statements
 */
pub fn semantic_validation(root:Vec<ASTNode>, symbol_table:&SymbolTable) -> Result<(), Box<dyn Error>> {
    for node in root {
        semantic_validation_subtree(&node, symbol_table, &vec![0])?;
    }

    Ok(())
}


/**
 * Called to generate an entire symbol table for all functions and variables in a program. Takes the root
 * `Vec<ASTNode>` of the program.
 */
pub fn generate_symbol_table(root:Vec<ASTNode>) -> SymbolTable {
    let mut table = SymbolTable { rows: vec![] };
    for node in root {
        generate_sub_symbol_table(node, &mut table, None);
    }

    table
}
