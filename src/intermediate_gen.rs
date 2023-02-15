use crate::parser::*;
use crate::semantics::SymbolTable;

use std::collections::HashMap;
use std::sync::atomic::{AtomicUsize, Ordering};


#[allow(dead_code)]
#[derive(Debug)]
pub enum Argument {
    Integer(i16)
}


#[allow(dead_code)]
#[derive(Debug)]
pub enum IntermediateInstr {
    Add,
    Sub,
    Div,
    Mult,
    Complement,
    LogicNeg,
    NumNeg,
    Push(Type, Argument),
    Load(Type, usize),
    Store(Type, usize),
    Return(Type),
    PushImm(Argument),
    FuncStart(String),
    FuncEnd(String),
}


pub struct AddrTypePair {
    address: usize,
    var_type: Type
}


fn gen_operator_code(operator:&Operator) -> IntermediateInstr {
    match operator {
        Operator::Addition => IntermediateInstr::Add,
        Operator::Subtraction => IntermediateInstr::Sub,
        Operator::Multiplication => IntermediateInstr::Mult,
        Operator::Division => IntermediateInstr::Div,
        Operator::Complement => IntermediateInstr::Complement,
        Operator::NegateNumerical => IntermediateInstr::NumNeg,
        Operator::NegateLogical => IntermediateInstr::LogicNeg,
    }
}


fn gen_intermediate_code(root:&ASTNode, instructions:&mut Vec<IntermediateInstr>, memory_map:&mut HashMap<String, AddrTypePair>, symbol_table:&SymbolTable, primitive_type:Option<Type>) {
    static NEXT_ADDRESS:AtomicUsize = AtomicUsize::new(0);
    match root {
        ASTNode::Function {identifier, statements, return_type} => {
            instructions.push(IntermediateInstr::FuncStart(identifier.to_owned()));

            for stmt in statements {
                gen_intermediate_code(stmt, instructions, memory_map, symbol_table, Some(return_type.clone()));
            }

            instructions.push(IntermediateInstr::FuncEnd(identifier.to_owned()));
        },

        ASTNode::ReturnStatement {expression} => {
            gen_intermediate_code(expression, instructions, memory_map, symbol_table, None);
            instructions.push(IntermediateInstr::Return(primitive_type.unwrap()))
        },

        ASTNode::VarDeclStatement {identifier, value, var_type, ..} => {
            match &**value {
                ASTNode::Expression {..} => {
                    gen_intermediate_code(value, instructions, memory_map, symbol_table, None);

                    let address = NEXT_ADDRESS.fetch_add(1, Ordering::Relaxed);
                    memory_map.insert(identifier.to_owned(), AddrTypePair {address: address, var_type: var_type.clone()});
                    instructions.push(IntermediateInstr::Store(var_type.clone(), address));
                },
                _ => {}
            }
        },

        ASTNode::VarAssignStatement {identifier, value} => {
            match &**value {
                ASTNode::Expression {..} => {
                    gen_intermediate_code(value, instructions, memory_map, symbol_table, None);

                    let metadata = memory_map.get(identifier).unwrap();
                    instructions.push(IntermediateInstr::Store(metadata.var_type.clone(), metadata.address));
                },
                _ => {}
            }
        },

        ASTNode::Expression {rhs, lhs, operator} => {
            gen_intermediate_code(&*lhs, instructions, memory_map, symbol_table, None);

            match rhs {
                Some(rhs) => gen_intermediate_code(rhs, instructions, memory_map, symbol_table, None),
                None => {}
            }

            match operator {
                Some(op) => instructions.push(gen_operator_code(op)),
                None => {}
            }
        },

        ASTNode::Term {child} => gen_intermediate_code(child, instructions, memory_map, symbol_table, None),

        ASTNode::Value {literal_type, value} => {
            let val = match *value {
                Literal::Integer(int) => int
            };
            instructions.push(IntermediateInstr::Push(literal_type.clone(), Argument::Integer(val)));
        },

        ASTNode::Identifier(identifier) => {
            let metadata = memory_map.get(identifier).unwrap();
            instructions.push(IntermediateInstr::Load(metadata.var_type.clone(), metadata.address));
        }
    }
}


pub fn generate_program_intermediate(ast:Vec<ASTNode>, symbol_table:&SymbolTable) -> Vec<IntermediateInstr> {
    let mut instructions = vec![];
    let mut memory_map:HashMap<String, AddrTypePair> = HashMap::new();
    for top_level in ast {
        gen_intermediate_code(&top_level, &mut instructions, &mut memory_map, &symbol_table, None);
    }

    instructions
}
