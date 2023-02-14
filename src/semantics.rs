use crate::parser::{Type, ASTNode, Mutability};
use crate::errors::SymbolNotFoundError;

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


    fn check_identifier_in_scope(&self, identifier:String, scope_history:Vec<usize>) -> Result<(), Box<dyn Error>> {
        for row in &self.rows {
            if row.get_identifier() == identifier && scope_history.contains(&row.get_scope_id()) {
                return Ok(());
            }
        }

        Err(Box::new(SymbolNotFoundError(identifier)))
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
        scope: usize,
        parent: Box<SymbolTableRow>
    },

    Function {
        identifier: String,
        return_type: Type,
        scope: usize,
        child_scopes: Vec<usize>
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
            SymbolTableRow::Variable {scope, ..} => *scope
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
 * 
 * Will panic if it finds an invalid reference to an identifier.
 */
fn semantic_analysis_subtree(subtree:ASTNode, table:&mut SymbolTable, parent:Option<SymbolTableRow>, mut scope_history:Vec<usize>) {
    match subtree.clone() {
        ASTNode::Function {return_type, identifier, statements} => {
            let scope_id = table.get_next_scope_id();
            scope_history.push(scope_id);
            let function_row = SymbolTableRow::Function {
                identifier: identifier,
                return_type: return_type,
                scope: scope_id,
                child_scopes: vec![]
            };
            table.add(function_row.clone());


            for statement in statements {
                semantic_analysis_subtree(statement, table, Some(function_row.clone()), scope_history.clone());
            }
        },

        ASTNode::VarDeclStatement {var_type, mutability, identifier, ..} => {
            table.add(
                SymbolTableRow::Variable {
                    identifier: identifier,
                    primitive_type: var_type,
                    mutability: mutability,
                    scope: parent.clone().unwrap().get_scope_id(),
                    parent: Box::new(parent.expect(&format!("Statement {:?} does not have a parent.", subtree)))
                }
            )
        },

        ASTNode::VarAssignStatement {identifier, ..} => {
            table.check_identifier_in_scope(identifier, scope_history).unwrap();
        },

        ASTNode::Identifier(_) => {}

        _ => {}
    };
}


/**
 * Called to generate an entire symbol table for all functions and variables in a program. Takes the root
 * `Vec<ASTNode>` of the program.
 */
pub fn semantic_analysis(root:Vec<ASTNode>) -> SymbolTable {
    let mut table = SymbolTable { rows: vec![] };
    for node in root {
        semantic_analysis_subtree(node, &mut table, None, vec![0]);
    }

    table
}
