
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
    
    // Values 
    IntLiteral(i32),
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


#[derive(Debug)]
pub enum Statement {
    Assign {
        varname: String,
        value: Expression
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


#[derive(Debug)]
pub struct Function {
    pub name: String,
    pub args: Vec<String>,
    pub body: Vec<Statement>,
}


#[derive(Debug)]
pub struct Program {
    pub functions: Vec<Function>,
    pub main_statements: Vec<Statement>
}


