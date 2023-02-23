use crate::ast::*;

use std::fmt;
use std::collections::HashMap;
use std::sync::atomic::{AtomicUsize, Ordering};


/**
 * Represents possible arguments to intermediate code instrs
 */
#[derive(Debug)]
pub enum Argument {
    Byte(u8),
    Integer(i16),
    Long(i32),
    Boolean(bool),
    Char(char)
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
    JumpNotZero(String),
    Call(String),
    Push(Type, Argument),
    Load(Type, usize),
    Store(Type, usize),
    Return(Type),
    FuncStart(String),
    FuncEnd(String),
    Label(String),
    Cast(Type, Type)
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
 * Contains the current context of the labels in the intermediate code, such as:
 *  - ieie_return_label: label for the end of the current if, else if, else block,
 *  - loop_break_label: label for the end of the current loop block,
 *  - loop_continue_label: label for the start of the current loop block
 */
#[derive(Clone)]
struct LabelContext {
    ieie_return_label:Option<String>,
    loop_break_label:Option<String>,
    loop_continue_label:Option<String>
}

impl LabelContext {
    fn new() -> LabelContext {
        LabelContext {
            ieie_return_label: None,
            loop_break_label: None,
            loop_continue_label: None
        }
    }

    fn update_ieie(&mut self, label:String) {
        self.ieie_return_label = Some(label)
    }

    fn update_break(&mut self, label:String) {
        self.loop_break_label = Some(label)
    }

    fn update_continue(&mut self, label:String) {
        self.loop_continue_label = Some(label)
    }
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
 * Takes the identifier of a function and a variable and returns a string in the format.
 */
fn get_var_repr(func_id:&str, id:&str) -> String {
    format!("{}_{}", func_id, id)
}


/**
 * Derives the next label from a static variable. Label is an underscore '_' followed by a hex representation
 * of the number of the label. 
 * 
 * For example, we start at "_1", then "_2", and the 32nd is "_20".
 */
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
            primitive_type:Option<Type>, func_name:&str, label_context:&mut LabelContext) {
    static NEXT_ADDRESS:AtomicUsize = AtomicUsize::new(0);
    match root {
        ASTNode::Function {identifier, statements, return_type, parameters, ..} => {
            instructions.push(IntermediateInstr::FuncStart(identifier.to_owned()));

            for param in parameters {
                gen_intermediate_code(param, instructions, memory_map, None, identifier, label_context)
            }

            for stmt in statements {
                gen_intermediate_code(stmt, instructions, memory_map, Some(return_type.clone()), identifier, label_context);
            }

            instructions.push(IntermediateInstr::FuncEnd(identifier.to_owned()));
        },

        ASTNode::ReturnStatement {expression} => {
            gen_intermediate_code(expression, instructions, memory_map, None, func_name, label_context);
            instructions.push(IntermediateInstr::Return(primitive_type.unwrap()))
        },

        ASTNode::VarDeclStatement {identifier, value, var_type, ..} => {
            match &**value {
                ASTNode::Expression {..} | ASTNode::TernaryExpression {..} => gen_intermediate_code(value, instructions, memory_map, None, func_name, label_context),
                _ => panic!("Cannot generate intermdeiate code in variable assignment for {:?}", value)
            }

            let address = NEXT_ADDRESS.fetch_add(1, Ordering::Relaxed);
            memory_map.insert(get_var_repr(func_name, identifier), AddrTypePair {address: address, var_type: var_type.clone()});
            instructions.push(IntermediateInstr::Store(var_type.clone(), address));
        },

        ASTNode::VarAssignStatement {identifier, value} => {
            match &**value {
                ASTNode::Expression {..} => {
                    gen_intermediate_code(value, instructions, memory_map, None, func_name, label_context);

                    let metadata = memory_map.get(&get_var_repr(func_name, identifier)).unwrap();
                    instructions.push(IntermediateInstr::Store(metadata.var_type.clone(), metadata.address));
                },
                _ => {}
            }
        },

        ASTNode::Expression {rhs, lhs, operator} => {
            gen_intermediate_code(&*lhs, instructions, memory_map, None, func_name, label_context);

            match rhs {
                Some(rhs) => gen_intermediate_code(rhs, instructions, memory_map, None, func_name, label_context),
                None => {}
            }

            match operator {
                Some(op) => instructions.push(gen_operator_code(op)),
                None => {}
            }
        },

        ASTNode::Term {child} => gen_intermediate_code(child, instructions, memory_map, None, func_name, label_context),

        ASTNode::Value {literal_type, value} => {
            let argument = match *value {
                Literal::Byte(byte) => Argument::Byte(byte),
                Literal::Integer(int) => Argument::Integer(int),
                Literal::Long(long) => Argument::Long(long),
                Literal::Boolean(boolean) => Argument::Boolean(boolean),
                Literal::Char(character) => Argument::Char(character)
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
                gen_intermediate_code(arg, instructions, memory_map, None, func_name, label_context);
            }

            instructions.push(IntermediateInstr::Call(identifier.to_string()));
        },

        ASTNode::IfElifElseStatement {statements} => {
            let return_label = get_next_label();
            label_context.update_ieie(return_label.clone());

            for statement in statements {
                gen_intermediate_code(statement, instructions, memory_map, None, func_name, label_context);
            }

            instructions.push(IntermediateInstr::Label(return_label));
        },

        ASTNode::IfStatement {condition, statements, ..} => {
            let label = get_next_label();
            gen_intermediate_code(condition, instructions, memory_map, None, func_name, label_context);
            instructions.push(IntermediateInstr::JumpZero(label.clone()));
            for statement in statements {
                gen_intermediate_code(statement, instructions, memory_map, None, func_name, label_context);
            }

            let return_label = label_context.ieie_return_label.as_ref().unwrap();
            instructions.push(IntermediateInstr::Jump(return_label.to_string()));
            instructions.push(IntermediateInstr::Label(label));
        },

        ASTNode::ElseStatement {statements, ..} => {
            for statement in statements {
                gen_intermediate_code(statement, instructions, memory_map, None, func_name, label_context);
            }
        },

        ASTNode::BooleanExpression {lhs, rhs, operator, connector} => {
            gen_intermediate_code(lhs, instructions, memory_map, None, func_name, label_context);
            match rhs {
                Some(rhs) => {
                    gen_intermediate_code(rhs, instructions, memory_map, None, func_name, label_context);
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
            gen_intermediate_code(lhs, instructions, memory_map, None, func_name, label_context);
            match rhs {
                Some(rhs) => {
                    gen_intermediate_code(rhs, instructions, memory_map, None, func_name, label_context);
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

        ASTNode::TypeCast {from, into} => {
            gen_intermediate_code(from, instructions, memory_map, None, func_name, label_context);
            let from_type = match &**from {
                ASTNode::Identifier(identifier) => &memory_map.get(&get_var_repr(func_name, &identifier)).unwrap().var_type,
                ASTNode::Value {literal_type, ..} => literal_type,
                other => panic!("{:?} is not a valid target for a type cast expression", other)
            };

            instructions.push(IntermediateInstr::Cast(from_type.clone(), into.clone()));
        },

        ASTNode::IndefLoop {statements, ..} => {
            let continue_label = get_next_label();
            let return_label = get_next_label();
            label_context.update_continue(continue_label.clone());
            label_context.update_break(return_label.clone());

            instructions.push(IntermediateInstr::Label(continue_label.clone()));
            for statement in statements {
                gen_intermediate_code(statement, instructions, memory_map, None, func_name, label_context);
            }

            instructions.push(IntermediateInstr::Jump(continue_label));
            instructions.push(IntermediateInstr::Label(return_label.clone()));
        },

        ASTNode::WhileLoop {statements, condition, ..} => {
            let start_label = get_next_label();
            let return_label = get_next_label();
            label_context.update_continue(start_label.clone());
            label_context.update_break(return_label.clone());
            instructions.push(IntermediateInstr::Label(start_label.clone()));

            gen_intermediate_code(condition, instructions, memory_map, None, func_name, label_context);
            instructions.push(IntermediateInstr::JumpZero(return_label.clone()));

            for statement in statements {
                gen_intermediate_code(statement, instructions, memory_map, None, func_name, label_context);
            }

            instructions.push(IntermediateInstr::Jump(start_label.to_string()));
            instructions.push(IntermediateInstr::Label(return_label));
        },

        ASTNode::ForLoop {control_type, control_identifier, control_initial, limit, step, statements, ..} => {
            // get initial control value
            gen_intermediate_code(control_initial, instructions, memory_map, None, func_name, label_context);

            // add control variable to memory map and memory
            let address = NEXT_ADDRESS.fetch_add(1, Ordering::Relaxed);
            memory_map.insert(get_var_repr(func_name, control_identifier), AddrTypePair {address: address, var_type: control_type.clone()});
            instructions.push(IntermediateInstr::Store(control_type.clone(), address));

            // add start label
            let start_label = get_next_label();
            let return_label = get_next_label();
            label_context.update_continue(start_label.clone());
            label_context.update_break(return_label.clone());
            instructions.push(IntermediateInstr::Label(start_label.clone()));

            // generate condition code
            gen_intermediate_code(limit, instructions, memory_map, None, func_name, label_context);
            instructions.push(IntermediateInstr::GreaterThan);
            instructions.push(IntermediateInstr::JumpNotZero(return_label.clone()));

            // generate statement block code
            for statement in statements {
                gen_intermediate_code(statement, instructions, memory_map, None, func_name, label_context);
            }

            // generate step code
            gen_intermediate_code(step, instructions, memory_map, None, func_name, label_context);

            // add step to control variable value
            let metadata = memory_map.get(&get_var_repr(func_name, control_identifier)).unwrap();
            instructions.push(IntermediateInstr::Load(metadata.var_type.clone(), metadata.address));
            instructions.push(IntermediateInstr::Add);

            // store result of control variable
            let metadata = memory_map.get(&get_var_repr(func_name, control_identifier)).unwrap();
            instructions.push(IntermediateInstr::Store(metadata.var_type.clone(), metadata.address));

            // go back to start of loop
            instructions.push(IntermediateInstr::Jump(start_label.to_string()));

            // add return label
            instructions.push(IntermediateInstr::Label(return_label.clone()));
        },

        ASTNode::Break => {
            instructions.push(IntermediateInstr::Jump(label_context.clone().loop_break_label.unwrap().to_string())); 
        },

        ASTNode::Continue => {
            instructions.push(IntermediateInstr::Jump(label_context.clone().loop_continue_label.unwrap().to_string()));
        },

        ASTNode::TernaryExpression {condition, if_true, if_false} => {
            let return_label = get_next_label();
            let false_label = get_next_label();
            gen_intermediate_code(condition, instructions, memory_map, None, func_name, label_context);

            instructions.push(IntermediateInstr::JumpZero(false_label.clone()));
            gen_intermediate_code(if_true, instructions, memory_map, None, func_name, label_context);
            instructions.push(IntermediateInstr::Jump(return_label.to_string()));

            instructions.push(IntermediateInstr::Label(false_label));
            gen_intermediate_code(if_false, instructions, memory_map, None, func_name, label_context);

            instructions.push(IntermediateInstr::Label(return_label));
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
    let context = LabelContext::new();
    for top_level in ast {
        gen_intermediate_code(&top_level, &mut instructions, &mut memory_map, None, "global", &mut context.clone());
    }

    instructions
}
