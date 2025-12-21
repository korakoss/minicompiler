use crate::ast::*;
use crate::common::*;

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
    Let,
    
    // Type stuff
    Int, 
    Bool,
    RightArrow,

    // Special
    EOF,
}


pub fn lex(program: &str) -> Vec<Token> {
    let mut tokens: Vec<Token> = Vec::new();
    let mut chars = program.chars().peekable();
    
    while let Some(&c) = chars.peek() {

        if c.is_whitespace() {
            chars.next();
            continue;
        }


        // Alphanumeric strings: keywords or identifiers
        if c.is_ascii_alphabetic() {
            let mut word = String::new();
            while let Some(&ch) = chars.peek() {
                if ch.is_ascii_alphanumeric() || ch == '_' {
                    word.push(ch);
                    chars.next();
                } else {
                    break;
                }
            }

            let token = match word.as_str() {
                "if" => Token::If,
                "else" => Token::Else,
                "while" => Token::While,
                "break" => Token::Break,
                "continue" => Token::Continue,
                "fun" => Token::Function,
                "return" => Token::Return,
                "print" => Token::Print,
                "let" => Token::Let,
                "int" => Token::Int,
                "bool" => Token::Bool,
                _ => Token::Identifier(word),
            };  
            tokens.push(token); 
        } 
        
        // Numbers
        else if c.is_ascii_digit() {
            let mut num_str = String::new();
            while let Some(&ch) = chars.peek() {
                if ch.is_ascii_digit() {
                    num_str.push(ch);
                    chars.next();
                } else {
                    break;
                }
            }
            let value = num_str.parse::<i32>().unwrap();
            tokens.push(Token::IntLiteral(value));
        }

        else if c == '=' {
            chars.next();
            if chars.peek() == Some(&'=') {
                chars.next();
                tokens.push(Token::Equals);
            } else {
                tokens.push(Token::Assign);
            }
        }

        else if c == '-' {
            chars.next();
            if chars.peek() == Some(&'>') {
                chars.next();
                tokens.push(Token::RightArrow);
            } else {
                tokens.push(Token::Minus);
            }
        }

        else {
            // Processing single character stuff
            let token = match c {
                '+' => Token::Plus,
                '*' => Token::Multiply,
                ';' => Token::Semicolon,
                '(' => Token::LeftParen,
                ')' => Token::RightParen,
                '{' => Token::LeftBrace,
                '}' => Token::RightBrace,
                '<' => Token::Less,
                '%' => Token::Modulo,
                ',' => Token::Comma,
                ':' => Token::Colon,
                _ => {panic!("Unexpected character: {}",c)},
            };
            chars.next();
            tokens.push(token);
        }
        
    } 
    tokens.push(Token::EOF);
    tokens
}


pub struct Parser {
    tokens: Vec<Token>,
    position: usize,
    loop_nest_counter: usize,
    within_function_body:bool,
    defined_funcs: Vec<Function>,
    glob_vars: Vec<String>,
    loc_vars: Vec<String>,
}


impl Parser {
    
    pub fn new(tokens: Vec<Token>) -> Self {
        Parser {
            tokens: tokens,
            position: 0,
            loop_nest_counter:0,
            within_function_body: false,
            defined_funcs: Vec::new(),
            glob_vars: Vec::new(),
            loc_vars: Vec::new(),
        }
    }
    
    fn is_at_end(&self) -> bool {
        self.position >= self.tokens.len() 
        || matches!(self.peek(), Token::EOF)
    }
    
    fn peek(&self) -> &Token {
        &self.tokens[self.position]
    }

    fn consume(&mut self) -> Token {
        let token = self.tokens[self.position].clone();
        self.position += 1;
        token
    }
    
    fn expect_unparametric_token(&mut self, expected_token: Token) {
        if self.peek() != &expected_token {
            panic!("Expected token {:?} at position {}, got token {:?}.", expected_token, self.position, self.peek()); 
        }
        self.consume();
    }
    
    // TODO: add result
    fn expect_identifier_token(&mut self) -> String {
        match self.consume() {
            Token::Identifier(name) => {
                return name;
            }
            other => {
                panic!("Expected identifier token, got token: {:?}", other);
            }
        }
    }

    fn is_expression_start(&self) -> bool {
        matches!(self.peek(), Token::IntLiteral(_) | Token::Identifier(_) | Token::LeftParen)
    }
    
    fn parse_funccall_args(&mut self) -> Vec<Box<Expression>> {
        if self.peek() == &Token::RightParen {
            println!("Boo");                                        // TODO: ?????
        }
        let mut args = Vec::new();
        if self.is_expression_start() {
            args.push(Box::new(self.parse_expression()));
            while self.peek() == &Token::Comma {
                self.consume();
                args.push(Box::new(self.parse_expression()));
            }
        } 
        args
    }

    fn parse_primitive(&mut self) -> Expression {
        
         match self.consume() {
            Token::IntLiteral(int) => {Expression::IntLiteral(int)},
            Token::Identifier(name) => {
                if self.peek() == &Token::LeftParen {        // Function call 
                    self.consume();
                    let args = self.parse_funccall_args(); 
                    self.expect_unparametric_token(Token::RightParen);
                    Expression::FuncCall { funcname: name, args: args}
                } else {
                    Expression::Variable(name)
                }
            },
            Token::LeftParen => {
                let paren_expr = self.parse_expression();
                self.expect_unparametric_token(Token::RightParen);
                paren_expr
            },
            _ => {
                let tok = self.consume();
                panic!("Unexpected token {:?} during expression parsing at position {:?}", tok, self.position);
            },
        }
    }

    // TODO: maybe we should make a big binop info table with rows like: Token, Binoptype,precedence
    fn get_binop_precedence(&mut self, op_token: Token) -> i8 {
        match op_token {
            Token::Plus| Token::Minus => 1,
            Token::Multiply | Token::Modulo => 2,
            Token::Equals | Token::Less => 0,
            _ => -1, 
        }
    }

    fn map_binop_token(&mut self, op_token: Token) -> BinaryOperator {
        match op_token {
            Token::Plus => BinaryOperator::Add,
            Token::Minus => BinaryOperator::Sub,
            Token::Multiply => BinaryOperator::Mul,
            Token::Equals => BinaryOperator::Equals,
            Token::Less => BinaryOperator::Less,
            Token::Modulo => BinaryOperator::Modulo,
            _ => panic!("Expected binary operator toke"),
        }
    }
   
    fn parse_expression_with_precedence(&mut self, current_level: i8) -> Expression {
        
        let mut current_expr = self.parse_primitive(); 

        while matches!(self.peek(), Token::Plus | Token::Minus | Token::Multiply | Token::Equals | Token::Less | Token::Modulo) {
            let prec = self.get_binop_precedence(self.peek().clone());
            if prec < current_level {
                break;
            }
            let optoken = self.consume();
            let op = self.map_binop_token(optoken);
            let next_expr = self.parse_expression_with_precedence(prec+1);
            current_expr = Expression::BinOp { op, left: Box::new(current_expr), right: Box::new(next_expr)};

        }
        current_expr 
    }

    fn parse_expression(&mut self) -> Expression {
        self.parse_expression_with_precedence(0)
    }

    fn parse_block(&mut self) -> Vec<Statement> {
       let mut statements = Vec::new();
        
        while !matches!(self.peek(), Token::RightBrace){
            statements.push(self.parse_statement());
        }
        statements
    }

    fn parse_type(&mut self) -> Type {
        match self.consume() {
            Token::Int => {Type::Integer},
            Token::Bool => {Type::Bool},
            _ => {panic!("Unexpected token in typing");}
        }
    }

    fn parse_statement(&mut self) -> Statement {
        
        if self.is_expression_start() {
            // TODO: later we can do trailing-expr return here
            let expr = self.parse_expression();
            self.expect_unparametric_token(Token::Assign);
            let assign_value = self.parse_expression();
            self.expect_unparametric_token(Token::Semicolon);
            return Statement::Assign { target: expr, value: assign_value};
        }
        match self.consume() {
                        
            Token::If => {
                let cond = self.parse_expression();
                self.expect_unparametric_token(Token::LeftBrace);
                let body = self.parse_block();
                self.expect_unparametric_token(Token::RightBrace);
                
                if matches!(self.peek(), Token::Else) {
                    self.expect_unparametric_token(Token::LeftBrace);
                    let else_body = self.parse_block();
                    self.expect_unparametric_token(Token::RightBrace);
                    return Statement::If{
                        condition: cond,
                        if_body: body,
                        else_body: Some(else_body),
                    }
   
                } else {
                    return Statement::If { 
                        condition: cond, 
                        if_body: body,
                        else_body: None,
                    }
                }
            }

            Token::While => {
                let cond = self.parse_expression();
                self.expect_unparametric_token(Token::LeftBrace);
                self.loop_nest_counter = self.loop_nest_counter + 1;
                let body = self.parse_block();
                self.expect_unparametric_token(Token::RightBrace);
                self.loop_nest_counter = self.loop_nest_counter - 1;
                Statement::While { 
                    condition: cond, 
                    body: body,
                }
            }

            Token::Break => {         
                if self.loop_nest_counter > 0 {
                    self.expect_unparametric_token(Token::Semicolon);
                    Statement::Break
                } else {
                    panic!("Break statement detected outside of any loop body at position {}", self.position);
                }
            }
            
            Token::Continue => {         
                if self.loop_nest_counter > 0 {
                    self.expect_unparametric_token(Token::Semicolon);
                    Statement::Continue
                } else {
                    panic!("Continue statement detected outside of any loop body at position {}", self.position);
                }
            }

            Token::Return => {
                if !self.within_function_body {
                    panic!("Return statement detected outside of a function body at position {}", self.position);
                }
                let return_expr = self.parse_expression();
                self.expect_unparametric_token(Token::Semicolon);
                Statement::Return(return_expr)
            }

            Token::Print => {
                self.expect_unparametric_token(Token::LeftParen);
                let expr = self.parse_expression();
                self.expect_unparametric_token(Token::RightParen);
                self.expect_unparametric_token(Token::Semicolon);        
                Statement::Print(expr)
            },

            Token::Let => {
                let varname = self.expect_identifier_token();         
                self.expect_unparametric_token(Token::Colon);
                let typ = self.parse_type();
                let var = VariableInfo{name: varname, typ: typ};
                self.expect_unparametric_token(Token::Assign);
                let value = self.parse_expression();
                self.expect_unparametric_token(Token::Semicolon);
                Statement::Let { var: var, value}
            },
            
            other => {                                      
                // we assume it'll be an Assign statement otherwise
                // could do it better
                // eg. somehow dry-run expression recog?
                if self.is_expression_start() {
                    // Trying to parse it as an Assign statement 
                    let target = self.parse_expression();
                    self.expect_unparametric_token(Token::Assign);
                    let value = self.parse_expression();
                    self.expect_unparametric_token(Token::Semicolon);
                    Statement::Assign {
                        target: target, 
                        value: value 

                    }
                }
                else {
                    panic!("Cannot recognize valid statement at position {:?} with token {:?}", self.position, other);
                }
            }
        }
    }    

    fn parse_function_declaration(&mut self) -> Function {
        let funcname = match self.consume(){
            Token::Identifier(name) => {name},
            other => {panic!("Expected a function name, got token {:?} at {}", other, self.position);}
        };

        let mut args = Vec::new();
        self.expect_unparametric_token(Token::LeftParen);
        if self.peek() == &Token::RightParen {
            self.consume();
        } else {
            let argname = self.expect_identifier_token();
            self.expect_unparametric_token(Token::Colon);
            let argtype = self.parse_type();
            args.push(VariableInfo{name: argname, typ: argtype});
                            
            while self.peek() == &Token::Comma {
                self.consume();  
                let argname = self.expect_identifier_token();
                self.expect_unparametric_token(Token::Colon);
                let argtype = self.parse_type();
                args.push(VariableInfo{name: argname, typ: argtype});

            }
            self.expect_unparametric_token(Token::RightParen);
        }
        self.expect_unparametric_token(Token::RightArrow);
        let ret_type = self.parse_type();
        self.expect_unparametric_token(Token::LeftBrace);
        self.within_function_body = true;
        let body = self.parse_block();
        self.expect_unparametric_token(Token::RightBrace);
        self.within_function_body = false;
        Function {name: funcname, args: args, body: body, ret_type}
    }

    pub fn parse_program(mut self) -> RawAST {
        let mut statements = Vec::new();
        while !self.is_at_end() {
            if self.peek() == &Token::Function {
                self.consume();
                let func_decl = self.parse_function_declaration();
                self.defined_funcs.push(func_decl);
            } else {
                statements.push(self.parse_statement());
            }
        }
        RawAST { functions: self.defined_funcs, main_statements: statements}
    }
}


