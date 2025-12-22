
#[derive(PartialEq,Eq, Debug, Hash, Clone)]
pub enum Type {
    Integer,
    Bool,
}

#[derive(Debug, Clone)]
pub struct Variable {
    pub name: String,
    pub typ: Type,
    // TODO: mutable, etc
}

#[derive(Debug, Clone)]
pub enum BinaryOperator {
    Add, 
    Sub, 
    Mul, 
    Equals,
    Less,       // NOTE: represents left < right 
    Modulo

    // TODO
        //Greater, 
        //Div (later, when floats ig?),
        //NotEqual
}

// TODO: eventually also UnaryOperation (eg. negation)
