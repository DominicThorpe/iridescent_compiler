use std::{error::Error, fmt};


#[derive(Debug)]
pub struct SymbolNotFoundError(pub String);
impl Error for SymbolNotFoundError {}

impl fmt::Display for SymbolNotFoundError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Could not find symbol {} in this scope.", self.0)
    }
}


#[derive(Debug)]
pub struct IncorrectDatatype;
impl Error for IncorrectDatatype {}

impl fmt::Display for IncorrectDatatype {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Incorrect datatype detected.")
    }
}


#[derive(Debug)]
pub struct IncorrectNumArguments(pub String);
impl Error for IncorrectNumArguments {}

impl fmt::Display for IncorrectNumArguments {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Incorrect number of arguments detected for function {}.", self.0)
    }
}


#[derive(Debug)]
pub struct BadFunctionReturn(pub String);
impl Error for BadFunctionReturn {}

impl fmt::Display for BadFunctionReturn {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Function {} does not return a value but does not have the void return type.", self.0)
    }
}


#[derive(Debug)]
pub struct ImmutableReassignmentError(pub String);
impl Error for ImmutableReassignmentError {}

impl fmt::Display for ImmutableReassignmentError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Cannot reassign constant variable {}", self.0)
    }
}
