use crate::parser::{Type, ASTNode, Mutability};


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
    pub fn add(&mut self, new_row:SymbolTableRow) {
        let new_identifier = new_row.get_identifier();
        for row in &self.rows {
            if row.get_identifier() == new_row.get_identifier()  && 
                    row.get_function_identifier() == new_row.get_function_identifier() {
                panic!("Duplicate identifier {} detected", new_identifier);
            }
        }

        self.rows.push(new_row);
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
        function: Box<SymbolTableRow>
    },

    Function {
        identifier: String,
        return_type: Type
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
     * Gets the identifier of the symbol table row if this row is a function, or the identifier of the
     * parent function if the row is a variable.
     */
    fn get_function_identifier(&self) -> String {
        match self {
            SymbolTableRow::Function {identifier, ..} => identifier.to_string(),
            SymbolTableRow::Variable {function, ..} => function.get_identifier().to_string()
        }
    } 
}


/**
 * Takes an `ASTNode` struct and either generates a row for the symbol table, which is passed by
 * reference, or calls itself recursively on each of that row's children to generate additional 
 * rows for them. 
 */
fn generate_sub_symbol_table(subtree:ASTNode, table:&mut SymbolTable, function:Option<SymbolTableRow>) {
    match subtree.clone() {
        ASTNode::Function {return_type, identifier, statements} => {
            let function_row = SymbolTableRow::Function {
                identifier: identifier,
                return_type: return_type
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
                    scope: 0,
                    function: Box::new(function.expect(&format!("Statement {:?} does not have a function.", subtree)))
                }
            )
        },

        _ => {}
    };
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
