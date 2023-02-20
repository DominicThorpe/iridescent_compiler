use crate::ast::*;

use std::fmt;
use std::collections::HashMap;
use std::sync::atomic::{AtomicUsize, Ordering};


/**
 * Represents possible arguments to intermediate code instrs
 */
#[derive(Debug)]
pub enum Argument {
    Integer(i16),
    Boolean(bool)
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
    LogicAnd,
    LogicOr,
    LogicXor,
    LeftShiftLogical,
    LeftShiftArithmetic,
    RightShiftLogical,
    NumNeg,
    GreaterThan,
    LessThan,
    GreaterEqual,
    LessEqual,
    Equal,
    NotEqual,
    Jump(String),
    JumpZero(String),
    Call(String),
    Push(Type, Argument),
    Load(Type, usize),
    Store(Type, usize),
    Return(Type),
    FuncStart(String),
    FuncEnd(String),
    Label(String)
}

impl fmt::Display for IntermediateInstr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            IntermediateInstr::FuncStart(_) => write!(f, "\n\n{:?}", self),
            IntermediateInstr::FuncEnd(_) => write!(f, "{:?}", self),
            IntermediateInstr::Label(label) => write!(f, "\n{}:", label),
            _ => write!(f, "    {:?}", self)
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
 * Takes a boolean operator and returns the corresponding intermediate stack instr
 */
fn gen_boolean_operator_code(operator:&BooleanOperator) -> IntermediateInstr {
    match operator {
        BooleanOperator::Equal => IntermediateInstr::Equal,
        BooleanOperator::NotEqual => IntermediateInstr::NotEqual,
        BooleanOperator::Greater => IntermediateInstr::GreaterThan,
        BooleanOperator::GreaterOrEqual => IntermediateInstr::GreaterEqual,
        BooleanOperator::Less => IntermediateInstr::LessThan,
        BooleanOperator::LessOrEqual => IntermediateInstr::LessEqual,
        BooleanOperator::Invert => IntermediateInstr::LogicNeg
    }
}


fn gen_boolean_connector_code(connector:&BooleanConnector) -> IntermediateInstr {
    match connector {
        BooleanConnector::And => IntermediateInstr::LogicAnd,
        BooleanConnector::Or => IntermediateInstr::LogicOr,
        BooleanConnector::XOr => IntermediateInstr::LogicXor
    }
}


/** 
 * Takes the identifier of a function and a variable and returns a string in the format "__function_variable__"
 */
fn get_var_repr(func_id:&str, id:&str) -> String {
    format!("{}_{}", func_id, id)
}


fn get_next_label() -> String {
    static NEXT_LABEL:AtomicUsize = AtomicUsize::new(1);
    let next_label = NEXT_LABEL.fetch_add(1, Ordering::Relaxed);
    format!("_{:x}", next_label)
}


/**
 * Takes an AST node and returns the intermediate code for it, then calls itself recursively to generate the
 * code of the sub nodes. Adding instructions to instructions vec is done through passing a mutable reference,
 * which is modified.
 * 
 * Requires the memory map, which maps identifiers to their scope and type, and a primitive type only when
 * handling a function to ensure the correct return type instr.
 */
fn gen_intermediate_code(root:&ASTNode, instructions:&mut Vec<IntermediateInstr>, memory_map:&mut HashMap<String, AddrTypePair>, 
            primitive_type:Option<Type>, func_name:&str, return_label:Option<&str>) {
    static NEXT_ADDRESS:AtomicUsize = AtomicUsize::new(0);
    match root {
        ASTNode::Function {identifier, statements, return_type, parameters, ..} => {
            instructions.push(IntermediateInstr::FuncStart(identifier.to_owned()));

            for param in parameters {
                gen_intermediate_code(param, instructions, memory_map, None, identifier, None)
            }

            for stmt in statements {
                gen_intermediate_code(stmt, instructions, memory_map, Some(return_type.clone()), identifier, None);
            }

            instructions.push(IntermediateInstr::FuncEnd(identifier.to_owned()));
        },

        ASTNode::ReturnStatement {expression} => {
            gen_intermediate_code(expression, instructions, memory_map, None, func_name, None);
            instructions.push(IntermediateInstr::Return(primitive_type.unwrap()))
        },

        ASTNode::VarDeclStatement {identifier, value, var_type, ..} => {
            match &**value {
                ASTNode::Expression {..} => {
                    gen_intermediate_code(value, instructions, memory_map, None, func_name, None);

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
                    gen_intermediate_code(value, instructions, memory_map, None, func_name, None);

                    let metadata = memory_map.get(&get_var_repr(func_name, identifier)).unwrap();
                    instructions.push(IntermediateInstr::Store(metadata.var_type.clone(), metadata.address));
                },
                _ => {}
            }
        },

        ASTNode::Expression {rhs, lhs, operator} => {
            gen_intermediate_code(&*lhs, instructions, memory_map, None, func_name, None);

            match rhs {
                Some(rhs) => gen_intermediate_code(rhs, instructions, memory_map, None, func_name, None),
                None => {}
            }

            match operator {
                Some(op) => instructions.push(gen_operator_code(op)),
                None => {}
            }
        },

        ASTNode::Term {child} => gen_intermediate_code(child, instructions, memory_map, None, func_name, None),

        ASTNode::Value {literal_type, value} => {
            let argument = match *value {
                Literal::Integer(int) => Argument::Integer(int),
                Literal::Boolean(boolean) => Argument::Boolean(boolean)
            };
            instructions.push(IntermediateInstr::Push(literal_type.clone(), argument));
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
                gen_intermediate_code(arg, instructions, memory_map, None, func_name, None);
            }

            instructions.push(IntermediateInstr::Call(identifier.to_string()));
        },

        ASTNode::IfElifElseStatement {statements} => {
            let return_label = get_next_label();
            for statement in statements {
                gen_intermediate_code(statement, instructions, memory_map, None, func_name, Some(&return_label));
            }

            instructions.push(IntermediateInstr::Label(return_label));
        },

        ASTNode::IfStatement {condition, statements, ..} => {
            let label = get_next_label();
            gen_intermediate_code(condition, instructions, memory_map, None, func_name, None);
            instructions.push(IntermediateInstr::JumpZero(label.clone()));
            for statement in statements {
                gen_intermediate_code(statement, instructions, memory_map, None, func_name, None);
            }

            instructions.push(IntermediateInstr::Jump(return_label.unwrap().to_string()));
            instructions.push(IntermediateInstr::Label(label));
        },

        ASTNode::ElseStatement {statements, ..} => {
            for statement in statements {
                gen_intermediate_code(statement, instructions, memory_map, None, func_name, None);
            }
        },

        ASTNode::BooleanExpression {lhs, rhs, operator, connector} => {
            gen_intermediate_code(lhs, instructions, memory_map, None, func_name, None);
            match rhs {
                Some(rhs) => {
                    gen_intermediate_code(rhs, instructions, memory_map, None, func_name, None);
                },
                None => {}
            }

            match operator {
                Some(operator) => {
                    instructions.push(gen_boolean_operator_code(operator));
                },

                None => {}
            }

            match connector {
                Some(connector) => {
                    instructions.push(gen_boolean_connector_code(connector));
                },

                None => {}
            }
        },

        ASTNode::BooleanTerm {lhs, operator, rhs} => {
            gen_intermediate_code(lhs, instructions, memory_map, None, func_name, None);
            match rhs {
                Some(rhs) => {
                    gen_intermediate_code(rhs, instructions, memory_map, None, func_name, None);
                },
                None => {}
            }

            match operator {
                Some(operator) => {
                    instructions.push(gen_boolean_operator_code(operator));
                },

                None => {}
            }
        },

        _ => {}
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
        gen_intermediate_code(&top_level, &mut instructions, &mut memory_map, None, "global", None);
    }

    instructions
}
