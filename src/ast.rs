
#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    // Operators
    Assign,
    Plus,
    Minus,
    Multiply,
    Equals,
    Less,
    Modulo,

    // Delimiters
    Semicolon,
    LeftParen,
    RightParen,
    LeftBrace,
    RightBrace,
    Comma,
    Colon,
    
    // Values 
    IntLiteral(i32),
    True, 
    False, 
    Identifier(String),
    
    // Keywords
    Print,
    If,
    Else,
    While,
    Break,
    Continue,
    Function,
    Return,
    
    // Type shit --all tbd
    IntType, 
    BoolType, 
    RightArrow, // TBD

    // Special
    EOF,
}



#[derive(Debug, Clone)]
pub enum BinaryOperator {
    Add, 
    Sub, 
    Mul, 
    Equals,
    Less,       // left < right 
    Modulo
    //Greater, 
    //Div (later, when floats ig?),
    //NotEqual
}


#[derive(Debug, Clone)]
pub enum Expression {
    IntLiteral(i32),

    Variable(String),

    BinOp {
       op: BinaryOperator,
       left: Box<Expression>,
       right: Box<Expression>,
    },

    FuncCall {
        funcname: String,
        args: Vec<Box<Expression>>,
    }
    
    // UnaryOp (eg. negation)
}


#[derive(Debug, Clone)]
pub enum Statement {
    Assign {
        varname: String,
        value: Expression,      // TODO: rename
    },
    If {
        condition: Expression,
        if_body: Vec<Statement>,
        else_body: Option<Vec<Statement>>,
    },
    While {
        condition: Expression,
        body: Vec<Statement>,
    },
    Break,
    Continue,
    Return(Expression),
    Print(Expression),
}


#[derive(Debug, Clone)]
pub struct Function {
    pub name: String,
    pub args: Vec<String>,
    pub body: Vec<Statement>,
}


#[derive(Debug, Clone)]
pub struct Program {
    pub functions: Vec<Function>,
    pub main_statements: Vec<Statement>
}


