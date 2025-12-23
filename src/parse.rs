use std::iter::Peekable;

use crate::lex::*;
use crate::ast::*;
use crate::common::*;


pub struct Parser {
    tokens: Peekable<std::vec::IntoIter<Token>>, 
    defined_funcs: Vec<ASTFunction>,
}


impl Parser {
    
    pub fn new(tokens: Vec<Token>) -> Self {
        Parser {
            tokens: tokens.into_iter().peekable(),
            defined_funcs: Vec::new(),
        }
    }

    pub fn parse_program(mut self) -> ASTProgram {
        let mut statements = Vec::new();
        while !self.tokens.peek().is_none() {
            if self.tokens.peek() == Some(&Token::Function) {
                self.tokens.next();
                let func_decl = self.parse_function();
                self.defined_funcs.push(func_decl);
            } else {
                statements.push(self.parse_statement());
            }
        }
        ASTProgram { functions: self.defined_funcs, main_statements: statements}
    }
    
    fn parse_function(&mut self) -> ASTFunction {
        let funcname = self.expect_identifier(); 
        let mut args = Vec::new();
        self.expect_unparametric_token(Token::LeftParen);
        while self.tokens.peek() != Some(&Token::RightParen) {         // TODO: much later, this might not vibe with tuples maybe?
            let name = self.expect_identifier();
            self.expect_unparametric_token(Token::Colon);
            let typ = self.parse_type();
            args.push(Variable{name,typ});
            if self.tokens.peek() == Some(&Token::Comma) {
                self.tokens.next();
            }
        }
        self.expect_unparametric_token(Token::RightParen);
        self.expect_unparametric_token(Token::RightArrow);
        let ret_type = self.parse_type();
        let body = self.expect_block();
        ASTFunction {name: funcname, args: args, body: body, ret_type}
    }
    
    fn expect_block(&mut self) -> Vec<ASTStatement> {
        let mut statements = Vec::new();
        self.expect_unparametric_token(Token::LeftBrace);
        while !matches!(self.tokens.peek(), Some(Token::RightBrace)){
            statements.push(self.parse_statement());
        }
        self.expect_unparametric_token(Token::RightBrace);
        statements
    }
    
    fn parse_statement(&mut self) -> ASTStatement {
        if self.is_expression_start() {
            let expr = self.parse_expression();
            self.expect_unparametric_token(Token::Assign);
            let assign_value = self.parse_expression();
            self.expect_unparametric_token(Token::Semicolon);
            return ASTStatement::Assign { target: expr, value: assign_value};
        }
        match self.tokens.next() {           
            Some(Token::If) => {
                let condition = self.parse_expression();
                let if_body = self.expect_block();
                let else_body =  if matches!(self.tokens.peek(), Some(Token::Else)) {
                    Some(self.expect_block())
                } else {None};
                ASTStatement::If {condition, if_body, else_body}
            }
            Some(Token::While) => {
                let cond = self.parse_expression();
                let body = self.expect_block();
                ASTStatement::While { 
                    condition: cond, 
                    body: body,
                }
            }
            Some(Token::Break) => {         
                self.expect_unparametric_token(Token::Semicolon);
                ASTStatement::Break
            }
            Some(Token::Continue) => {         
                self.expect_unparametric_token(Token::Semicolon);
                ASTStatement::Continue
            }
            Some(Token::Return) => {
                let return_expr = self.parse_expression();
                self.expect_unparametric_token(Token::Semicolon);
                ASTStatement::Return(return_expr)
            }
            Some(Token::Print) => {
                self.expect_unparametric_token(Token::LeftParen);
                let expr = self.parse_expression();
                self.expect_unparametric_token(Token::RightParen);
                self.expect_unparametric_token(Token::Semicolon);        
                ASTStatement::Print(expr)
            },
            Some(Token::Let) => {
                let varname = self.expect_identifier();         
                self.expect_unparametric_token(Token::Colon);
                let typ = self.parse_type();
                let var = Variable{name: varname, typ: typ};
                self.expect_unparametric_token(Token::Assign);
                let value = self.parse_expression();
                self.expect_unparametric_token(Token::Semicolon);
                ASTStatement::Let { var: var, value}
            },
            other => {                                      
                println!("{:#?}", self.defined_funcs);
                panic!("Cannot recognize valid statement starting with token {:?}", other);              
            }
        }
    } 
    
    fn parse_funccall_args(&mut self) -> Vec<Box<ASTExpression>> {
        let mut args = Vec::new();
        if self.is_expression_start() {
            args.push(Box::new(self.parse_expression()));
            while self.tokens.peek() == Some(&Token::Comma) {
                self.tokens.next();
                args.push(Box::new(self.parse_expression()));
            }
        } 
        args
    }
    
    fn parse_expression(&mut self) -> ASTExpression {
        self.parse_expression_with_precedence(0)
    }

    fn parse_type(&mut self) -> Type {
        match self.tokens.next() {
            Some(Token::Int) => {Type::Integer},
            Some(Token::Bool) => {Type::Bool},
            _ => {panic!("Unexpected token in typing");}
        }
    }

    fn expect_unparametric_token(&mut self, expected_token: Token) {
        if self.tokens.peek() != Some(&expected_token) {
            panic!("Expected token {:?}, got token {:?}.", expected_token, self.tokens.peek().unwrap()); 
        }
        self.tokens.next();
    }
    
    fn expect_identifier(&mut self) -> String {
        match self.tokens.next() {
            Some(Token::Identifier(name)) => {
                return name;
            }
            other => {
                panic!("Expected identifier token, got token: {:?}", other);
            }
        }
    }

    fn is_expression_start(&mut self) -> bool {
        matches!(self.tokens.peek(), Some(Token::IntLiteral(_) | Token::Identifier(_) | Token::LeftParen))
    }
    
    fn parse_expression_with_precedence(&mut self, current_level: u8) -> ASTExpression {
        let mut current_expr = self.parse_expression_atom();
        loop {
            let prec = match self.tokens.peek() {
                Some(Token::Plus | Token::Minus | Token::Multiply | Token::Equals | Token::Less | Token::Modulo) => {
                    get_binop_precedence(self.tokens.peek().unwrap())
                }
                _ => break,
            };
            if prec < current_level {
                break;
            }
            let optoken = self.tokens.next().unwrap();  
            let op = map_binop_token(&optoken);
            let next_expr = self.parse_expression_with_precedence(prec + 1);
            current_expr = ASTExpression::BinOp { op, left: Box::new(current_expr), right: Box::new(next_expr) };
        }
        current_expr
    }
    
    fn parse_expression_atom(&mut self) -> ASTExpression {
         match self.tokens.next() {
            Some(Token::IntLiteral(int)) => {ASTExpression::IntLiteral(int)},
            Some(Token::Identifier(name)) => {
                if self.tokens.peek() == Some(&Token::LeftParen) {        // Function call 
                    self.tokens.next();
                    let args = self.parse_funccall_args(); 
                    self.expect_unparametric_token(Token::RightParen);
                    ASTExpression::FuncCall { funcname: name, args: args}
                } else {
                    ASTExpression::Variable(name)
                }
            },
            Some(Token::LeftParen) => {
                let paren_expr = self.parse_expression();
                self.expect_unparametric_token(Token::RightParen);
                paren_expr
            },
            Some(Token::True) => {ASTExpression::BoolTrue},
            Some(Token::False) => {ASTExpression::BoolFalse},
            other => {
                panic!("Unexpected token {:?} during expression parsing", other);
            },
        }
    }
}


// TODO: instead of these two funcs, maybe we should make a big binop info table with rows like: Token, Binoptype,precedence 

fn get_binop_precedence(op_token: &Token) -> u8 {
    match op_token {
        &Token::Plus| Token::Minus => 1,
        &Token::Multiply | Token::Modulo => 2,
        &Token::Equals | Token::Less => 0,
        _ => panic!("Unexpected token found where binary operator was expected"), 
    }
}

fn map_binop_token(op_token: &Token) -> BinaryOperator {
    match op_token {
        &Token::Plus => BinaryOperator::Add,
        &Token::Minus => BinaryOperator::Sub,
        &Token::Multiply => BinaryOperator::Mul,
        &Token::Equals => BinaryOperator::Equals,
        &Token::Less => BinaryOperator::Less,
        &Token::Modulo => BinaryOperator::Modulo,
        _ => panic!("Expected binary operator token"),
    }
}

