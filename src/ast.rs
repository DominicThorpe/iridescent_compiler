/**
 * Represents all the currently implemented primitive datatypes.
 */
#[derive(PartialEq, Eq, Debug, Clone)]
pub enum Type {
    Void,
    Byte,
    Integer,
    Long,
    Char,
    Boolean
}


/**
 * Represents a literal of any primitive datatype.
 */
#[derive(PartialEq, Eq, Debug, Clone)]
pub enum Literal {
    Byte(u8),
    Integer(i16),
    Long(i32),
    Char(char),
    Boolean(bool)
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
        statements: Vec<ASTNode>,
        scope: usize
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

    TernaryExpression {
        condition: Box<ASTNode>,
        if_true: Box<ASTNode>,
        if_false: Box<ASTNode>
    },

    IfElifElseStatement {
        statements: Vec<ASTNode>
    },

    IfStatement {
        condition: Box<ASTNode>,
        statements: Vec<ASTNode>,
        scope: usize
    },

    ElseStatement {
        statements: Vec<ASTNode>,
        scope: usize
    },

    TypeCast {
        from: Box<ASTNode>,
        into: Type
    },

    IndefLoop {
        statements: Vec<ASTNode>,
        scope: usize
    },

    WhileLoop {
        condition: Box<ASTNode>,
        statements: Vec<ASTNode>,
        scope: usize
    },

    ForLoop {
        control_type: Type,
        control_identifier: String,
        control_initial: Box<ASTNode>,
        limit: Box<ASTNode>,
        step: Box<ASTNode>,
        statements: Vec<ASTNode>,
        scope: usize
    },

    Identifier(String),
    Break,
    Continue
}


/**
 * Takes a string representing a primitive type and returns `Type` struct object representing it.
 * 
 * ### Examples
 * `assert_eq!("int", Type::Integer)`
 * 
 * `assert_eq!("void", Type::Void)`
 */
pub fn get_type_from_string(type_str:&str) -> Type {
    match type_str {
        "void" => Type::Void,
        "byte" => Type::Byte,
        "int" => Type::Integer,
        "bool" => Type::Boolean,
        "long" => Type::Long,
        "char" => Type::Char,
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
pub fn get_boolean_operator_from_str(operator_str:&str) -> BooleanOperator {
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
pub fn get_boolean_connector_from_str(connector_str:&str) -> BooleanConnector {
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
pub fn get_unary_operator_from_str(operator_str:&str) -> Operator {
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
pub fn get_binary_operator_from_str(operator_str:&str) -> Operator {
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
pub fn get_mutability_from_str(mutability_str:&str) -> Mutability {
    match mutability_str {
        "mut" => Mutability::Mutable,
        "const" => Mutability::Constant,
        _ => panic!("Unknown mutability modifier {}", mutability_str)
    }
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
pub fn get_int_from_str_literal(literal:&str) -> i64 {
    let mut literal = literal;
    if literal.ends_with("l") | literal.ends_with("b") {
        literal = &literal[0..literal.len() - 1];
    };

    if literal.starts_with("0b") {
        return i64::from_str_radix(&literal[2..], 2).unwrap();
    } else if literal.starts_with("0x") {
        return i64::from_str_radix(&literal[2..], 16).unwrap();
    } else {
        return literal.parse().unwrap();
    }
}


/**
 * Takes a string of either "true" or "false" and returns the corresponding boolean value.
 * 
 * ### Examples
 * `assert_eq!(get_bool_from_str_literal("true"), true);`
 * 
 * `assert_eq!(get_bool_from_str_literal("false"), false);`
 */
pub fn get_bool_from_str_literal(literal:&str) -> bool {
    match literal {
        "true" => true,
        "false" => false,
        _ => panic!("Invalid boolean literal {}", literal)
    }
}
