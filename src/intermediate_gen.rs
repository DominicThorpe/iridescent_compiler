use crate::parser::*;

use std::fmt;
use std::collections::HashMap;
use std::sync::atomic::{AtomicUsize, Ordering};


/**
 * Represents possible arguments to intermediate code instrs
 */
#[derive(Debug)]
pub enum Argument {
    Integer(i16)
}


/**
 * Used to represent the instruction set of the intermediate code language
 */
#[derive(Debug)]
pub enum IntermediateInstr {
    Add,
    Sub,
    Div,
    Mult,
    BitwiseAnd,
    BitwiseOr,
    BitwiseXor,
    Complement,
    LogicNeg,
    LeftShiftLogical,
    LeftShiftArithmetic,
    RightShiftLogical,
    NumNeg,
    Call(String),
    Push(Type, Argument),
    Load(Type, usize),
    Store(Type, usize),
    Return(Type),
    FuncStart(String),
    FuncEnd(String),
}

impl fmt::Display for IntermediateInstr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            IntermediateInstr::FuncStart(_) => write!(f, "\n\n{:?}", self),
            _ => write!(f, "{:?}", self)
        }
    }
}


/**
 * Used to map identifiers to address, type pairs
 */
#[derive(Debug)]
pub struct AddrTypePair {
    address: usize,
    var_type: Type
}


/**
 * Takes an operator and returns the corresponding intermediate stack instr
 */
fn gen_operator_code(operator:&Operator) -> IntermediateInstr {
    match operator {
        Operator::Addition => IntermediateInstr::Add,
        Operator::Subtraction => IntermediateInstr::Sub,
        Operator::Multiplication => IntermediateInstr::Mult,
        Operator::Division => IntermediateInstr::Div,
        Operator::Complement => IntermediateInstr::Complement,
        Operator::NegateNumerical => IntermediateInstr::NumNeg,
        Operator::NegateLogical => IntermediateInstr::LogicNeg,
        Operator::And => IntermediateInstr::BitwiseAnd,
        Operator::Or => IntermediateInstr::BitwiseOr,
        Operator::XOr => IntermediateInstr::BitwiseXor,
        Operator::LeftShiftLogical => IntermediateInstr::LeftShiftLogical,
        Operator::LeftShiftArithmetic => IntermediateInstr::LeftShiftArithmetic,
        Operator::RightShiftLogical => IntermediateInstr::RightShiftLogical
    }
}


/** 
 * Takes the identifier of a function and a variable and returns a string in the format "__function_variable__"
 */
fn get_var_repr(func_id:&str, id:&str) -> String {
    format!("{}_{}", func_id, id)
}


/**
 * Takes an AST node and returns the intermediate code for it, then calls itself recursively to generate the
 * code of the sub nodes. Adding instructions to instructions vec is done through passing a mutable reference,
 * which is modified.
 * 
 * Requires the memory map, which maps identifiers to their scope and type, and a primitive type only when
 * handling a function to ensure the correct return type instr.
 */
fn gen_intermediate_code(root:&ASTNode, instructions:&mut Vec<IntermediateInstr>, memory_map:&mut HashMap<String, AddrTypePair>, primitive_type:Option<Type>, func_name:&str) {
    static NEXT_ADDRESS:AtomicUsize = AtomicUsize::new(0);
    match root {
        ASTNode::Function {identifier, statements, return_type, parameters} => {
            instructions.push(IntermediateInstr::FuncStart(identifier.to_owned()));

            for param in parameters {
                gen_intermediate_code(param, instructions, memory_map, None, identifier)
            }

            for stmt in statements {
                gen_intermediate_code(stmt, instructions, memory_map, Some(return_type.clone()), identifier);
            }

            instructions.push(IntermediateInstr::FuncEnd(identifier.to_owned()));
        },

        ASTNode::ReturnStatement {expression} => {
            gen_intermediate_code(expression, instructions, memory_map, None, func_name);
            instructions.push(IntermediateInstr::Return(primitive_type.unwrap()))
        },

        ASTNode::VarDeclStatement {identifier, value, var_type, ..} => {
            match &**value {
                ASTNode::Expression {..} => {
                    gen_intermediate_code(value, instructions, memory_map, None, func_name);

                    let address = NEXT_ADDRESS.fetch_add(1, Ordering::Relaxed);
                    memory_map.insert(get_var_repr(func_name, identifier), AddrTypePair {address: address, var_type: var_type.clone()});
                    instructions.push(IntermediateInstr::Store(var_type.clone(), address));
                },
                _ => {}
            }
        },

        ASTNode::VarAssignStatement {identifier, value} => {
            match &**value {
                ASTNode::Expression {..} => {
                    gen_intermediate_code(value, instructions, memory_map, None, func_name);

                    let metadata = memory_map.get(&get_var_repr(func_name, identifier)).unwrap();
                    instructions.push(IntermediateInstr::Store(metadata.var_type.clone(), metadata.address));
                },
                _ => {}
            }
        },

        ASTNode::Expression {rhs, lhs, operator} => {
            gen_intermediate_code(&*lhs, instructions, memory_map, None, func_name);

            match rhs {
                Some(rhs) => gen_intermediate_code(rhs, instructions, memory_map, None, func_name),
                None => {}
            }

            match operator {
                Some(op) => instructions.push(gen_operator_code(op)),
                None => {}
            }
        },

        ASTNode::Term {child} => gen_intermediate_code(child, instructions, memory_map, None, func_name),

        ASTNode::Value {literal_type, value} => {
            let val = match *value {
                Literal::Integer(int) => int
            };
            instructions.push(IntermediateInstr::Push(literal_type.clone(), Argument::Integer(val)));
        },

        ASTNode::Identifier(identifier) => {
            let metadata = memory_map.get(&get_var_repr(func_name, identifier)).unwrap();
            instructions.push(IntermediateInstr::Load(metadata.var_type.clone(), metadata.address));
        },

        ASTNode::Parameter {param_type, identifier} => {
            let address = NEXT_ADDRESS.fetch_add(1, Ordering::Relaxed);
            memory_map.insert(get_var_repr(func_name, identifier), AddrTypePair {address: address, var_type: param_type.clone()});
        },

        ASTNode::FunctionCall {identifier, arguments} => {
            for arg in arguments {
                gen_intermediate_code(arg, instructions, memory_map, None, func_name);
            }

            instructions.push(IntermediateInstr::Call(identifier.to_string()));
        }
    }
}


/**
 * Takes the root node vector of the program's AST and returns a vector representing the intermediate code of
 * the program.
 */
pub fn generate_program_intermediate(ast:Vec<ASTNode>) -> Vec<IntermediateInstr> {
    let mut instructions = vec![];
    let mut memory_map:HashMap<String, AddrTypePair> = HashMap::new();
    for top_level in ast {
        gen_intermediate_code(&top_level, &mut instructions, &mut memory_map, None, "global");
    }

    instructions
}
