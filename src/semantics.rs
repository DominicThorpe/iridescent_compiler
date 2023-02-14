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
        scope: usize,
        parent_scope: usize
    }
}

impl SymbolTableRow {
    /**
     * Returns the identifier of the symbol table row
     */
    fn get_identifier(&self) -> String {
        match self {
            SymbolTableRow::Function {identifier, ..} => identifier.to_string(),
            SymbolTableRow::Variable {identifier, ..} => identifier.to_string()
        }
    }


    /**
     * Returns the ID of the scope of the symbol
     */
    fn get_scope_id(&self) -> usize {
        match self {
            SymbolTableRow::Function {scope, ..} => *scope,
            SymbolTableRow::Variable {parent_scope, ..} => *parent_scope
        }
    }


    /**
     * Returns the ID of the scope of the symbol's parent
     */
    fn get_parent_scope_id(&self) -> usize {
        match self {
            SymbolTableRow::Function {parent_scope, ..} => *parent_scope,
            SymbolTableRow::Variable {parent_scope, ..} => *parent_scope
        }
    }


    /**
     * Gets the identifier of the symbol table row if this row is a function, or the identifier of the
     * parent function if the row is a variable.
     */
    fn get_parent_identifier(&self) -> String {
        match self {
            SymbolTableRow::Function {identifier, ..} => identifier.to_string(),
            SymbolTableRow::Variable {parent, ..} => parent.get_identifier().to_string()
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
        ASTNode::Function {return_type, identifier, statements} => {
            let scope_id = table.get_next_scope_id();
            let function_row = SymbolTableRow::Function {
                identifier: identifier,
                return_type: return_type,
                parent_scope: 0,
                scope: scope_id
            };
            table.add(function_row.clone());


            for statement in statements {
                generate_sub_symbol_table(statement, table, Some(function_row.clone()));
            }
        },

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

        _ => {}
    };
}


fn validate_term_of_type(node:&ASTNode, required_type:&Type) -> Result<(), Box<dyn Error>> {
    match node {
        ASTNode::Term { child } => {
            match &**child {
                ASTNode::Expression {..} => {
                    match validate_statement_of_type( &*child, &required_type ) {
                        Ok(_) => {},
                        Err(_) => {
                            return Err(Box::new(IncorrectDatatype)); 
                        }
                    }
                },

                ASTNode::Value {literal_type, ..} => {
                    if literal_type == required_type {
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
 * semantically valid (i.e. everything is of the same datatype). Will return an Error if this
 * is not true.
 */
fn validate_statement_of_type(node:&ASTNode, required_type:&Type) -> Result<(), Box<dyn Error>> {
    match &node {
        ASTNode::Expression {lhs, rhs, ..} => {
            validate_term_of_type(lhs, required_type)?;
            match &rhs {
                None => {},
                Some(term) => {
                    validate_term_of_type(term, required_type)?;
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
fn semantic_validation_subtree(node:ASTNode, symbol_table:&SymbolTable, scope_history:&mut Vec<usize>) -> Result<(), Box<dyn Error>> {
    match node {
        ASTNode::Function {identifier, statements, ..} => {
            for statement in statements {
                let mut scope_history = scope_history.clone();
                scope_history.push(symbol_table.get_identifier_in_scope(&identifier, &scope_history)?);
                semantic_validation_subtree(statement, &symbol_table, &mut scope_history)?;
            }
        },

        ASTNode::Expression {..} => {
            validate_statement_of_type(&node, &Type::Integer)?;
        },
        
        ASTNode::VarAssignStatement {identifier, ..} => {
            symbol_table.get_identifier_in_scope(&identifier, &scope_history)?;
        },

        ASTNode::Identifier(_) => {},

        _ => {}
    }

    Ok(())
}


/**
 * Takes the root node of the AST and runs semantic analysis, checking for:
 *   - undeclared/out of scope variables
 * 
 * TODO:
 *   - no/incorrect return statements
 *   - reassignment to immutable variable
 *   - operations on non-matching datatypes
 *   - invalid-sized literal assignments
 */
pub fn semantic_validation(root:Vec<ASTNode>, symbol_table:&SymbolTable) -> Result<(), Box<dyn Error>> {
    for node in root {
        semantic_validation_subtree(node, symbol_table, &mut vec![0])?;
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
