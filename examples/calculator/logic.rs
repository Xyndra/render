pub enum CalculationPart {
    Number(f64),
    Operator(Operator),
}
pub enum Operator {
    Add,
    Subtract,
    Multiply,
    Divide,
}

pub enum CalculationResult {
    Number(f64),
    Error(ErrorType),
}

pub enum ErrorType {
    DivisionByZero,
}
