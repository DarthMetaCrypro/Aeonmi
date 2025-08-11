use std::fmt;

#[derive(Debug)]
pub enum CoreError {
    InterpretationError(String),
    IoError(String),
    InvalidOperation(String),
    GeneralError(String),
}

impl fmt::Display for CoreError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CoreError::InterpretationError(msg) => write!(f, "Interpretation Error: {}", msg),
            CoreError::IoError(msg) => write!(f, "IO Error: {}", msg),
            CoreError::InvalidOperation(msg) => write!(f, "Invalid Operation: {}", msg),
            CoreError::GeneralError(msg) => write!(f, "Error: {}", msg),
        }
    }
}

impl std::error::Error for CoreError {}

impl CoreError {
    pub fn interpretation(message: &str) -> Self { CoreError::InterpretationError(message.to_string()) }
    pub fn io_error(message: &str) -> Self { CoreError::IoError(message.to_string()) }
    pub fn invalid_operation(message: &str) -> Self { CoreError::InvalidOperation(message.to_string()) }
    pub fn general_error(message: &str) -> Self { CoreError::GeneralError(message.to_string()) }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test] fn test_interpretation_error() {
        let err = CoreError::interpretation("Invalid syntax");
        assert_eq!(format!("{}", err), "Interpretation Error: Invalid syntax");
    }
    #[test] fn test_io_error() {
        let err = CoreError::io_error("File not found");
        assert_eq!(format!("{}", err), "IO Error: File not found");
    }
    #[test] fn test_invalid_operation_error() {
        let err = CoreError::invalid_operation("Cannot divide by zero");
        assert_eq!(format!("{}", err), "Invalid Operation: Cannot divide by zero");
    }
    #[test] fn test_general_error() {
        let err = CoreError::general_error("Something went wrong");
        assert_eq!(format!("{}", err), "Error: Something went wrong");
    }
}
