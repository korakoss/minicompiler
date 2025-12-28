use std::collections::HashMap;
use std::iter::Peekable;

use crate::lex::*;
use crate::uast::*;
use crate::common::*;


pub struct Parser {
    tokens: Peekable<std::vec::IntoIter<Token>>, 
    defined_funcs: Vec<UASTFunction>,
    defined_structs: HashMap<TypeIdentifier, UASTStructDef>
}


impl Parser {
    
    pub fn new(tokens: Vec<Token>) -> Self {
        Parser {
            tokens: tokens.into_iter().peekable(),
            defined_funcs: Vec::new(),              // Could be a hashmap indexed by sign?
            defined_structs: HashMap::new(),
        }
    }

    pub fn parse_program(mut self) -> UASTProgram {
        while !self.tokens.peek().is_none() {
            match self.tokens.peek().unwrap() {
                &Token::Struct => {
                    self.tokens.next();
                    let struct_name = self.expect_type_identifier();
                    let struct_def = self.parse_struct_def();
                    self.defined_structs.insert(struct_name, struct_def);
                }
                &Token::Function => {
                    let func = self.parse_function();
                    self.defined_funcs.push(func);
                }
                _ => {panic!("Invalid token, expected struct or func def");}
            }
        }
        UASTProgram { 
            struct_defs: self.defined_structs,
            functions: self.defined_funcs,
        } 
    }

    fn expect_type_identifier(&mut self) -> TypeIdentifier {
        match self.tokens.next() {
            Some(Token::Identifier(type_name)) => TypeIdentifier(type_name),
            Some(Token::Int) => TypeIdentifier("int".to_string()),
            Some(Token::Bool) => TypeIdentifier("int".to_string()),
            _ => {panic!("Expected type identifier");}
        }
    }

    fn parse_struct_def(&mut self) -> UASTStructDef {
        self.expect_unparametric_token(Token::LeftBrace);
        let mut fields = HashMap::new();
        while self.tokens.peek() != Some(&Token::RightBrace) {
            let field_name = self.expect_identifier();
            self.expect_unparametric_token(Token::Colon);
            let field_type = self.expect_type_identifier();
            self.expect_unparametric_token(Token::Comma);
            fields.insert(field_name, field_type);
        }
        self.expect_unparametric_token(Token::RightBrace);
        UASTStructDef { 
            fields 
        }
    }
    
    fn parse_function(&mut self) -> UASTFunction {
        self.expect_unparametric_token(Token::Function);
        let funcname = self.expect_identifier(); 
        let mut args = Vec::new();
        self.expect_unparametric_token(Token::LeftParen);
        while self.tokens.peek() != Some(&Token::RightParen) {         // TODO: much later, this might not vibe with tuples maybe?
            let name = self.expect_identifier();
            self.expect_unparametric_token(Token::Colon);
            let type_id = self.expect_type_identifier(); 
            args.push(UASTVariable{name, retar_type: type_id});
            if self.tokens.peek() == Some(&Token::Comma) {
                self.tokens.next();
            }
        }
        self.expect_unparametric_token(Token::RightParen);
        let ret_type_id = match self.tokens.peek() {
            Some(Token::RightArrow) => {
                self.tokens.next();
                self.expect_type_identifier()
            }
            _ => {
               TypeIdentifier("none".to_string()) 
            }
        };
        let body = self.expect_block();
        UASTFunction {name: funcname, args: args, body: body, ret_type: ret_type_id}
    }
    
    fn expect_block(&mut self) -> Vec<UASTStatement> {
        let mut statements = Vec::new();
        self.expect_unparametric_token(Token::LeftBrace);
        while !matches!(self.tokens.peek(), Some(Token::RightBrace)){
            statements.push(self.parse_statement());
        }
        self.expect_unparametric_token(Token::RightBrace);
        statements
    }
    
    fn parse_statement(&mut self) -> UASTStatement {
        if self.is_expression_start() {
            let expr = self.parse_expression();
            self.expect_unparametric_token(Token::Assign);
            let assign_value = self.parse_expression();
            self.expect_unparametric_token(Token::Semicolon);
            return UASTStatement::Assign { target: expr, value: assign_value};
        }
        match self.tokens.next() {           
            Some(Token::If) => {
                let condition = self.parse_expression();
                let if_body = self.expect_block();
                let else_body =  if matches!(self.tokens.peek(), Some(Token::Else)) {
                    Some(self.expect_block())
                } else {None};
                UASTStatement::If {condition, if_body, else_body}
            }
            Some(Token::While) => {
                let cond = self.parse_expression();
                let body = self.expect_block();
                UASTStatement::While { 
                    condition: cond, 
                    body: body,
                }
            }
            Some(Token::Break) => {         
                self.expect_unparametric_token(Token::Semicolon);
                UASTStatement::Break
            }
            Some(Token::Continue) => {         
                self.expect_unparametric_token(Token::Semicolon);
                UASTStatement::Continue
            }
            Some(Token::Return) => {
                let return_expr = self.parse_expression();
                self.expect_unparametric_token(Token::Semicolon);
                UASTStatement::Return(return_expr)
            }
            Some(Token::Print) => {
                self.expect_unparametric_token(Token::LeftParen);
                let expr = self.parse_expression();
                self.expect_unparametric_token(Token::RightParen);
                self.expect_unparametric_token(Token::Semicolon);        
                UASTStatement::Print(expr)
            },
            Some(Token::Let) => {
                let varname = self.expect_identifier();         
                self.expect_unparametric_token(Token::Colon);
                let type_identifier = self.expect_type_identifier();
                let var = UASTVariable{name: varname, retar_type: type_identifier};
                self.expect_unparametric_token(Token::Assign);
                let value = self.parse_expression();
                self.expect_unparametric_token(Token::Semicolon);
                UASTStatement::Let { var: var, value}
            },
            other => {                                      
                println!("{:#?}", self.defined_funcs);
                panic!("Cannot recognize valid statement starting with token {:?}", other);              
            }
        }
    } 
    
    fn parse_funccall_args(&mut self) -> Vec<Box<UASTExpression>> {
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
    
    fn parse_expression(&mut self) -> UASTExpression {
        self.parse_expression_with_precedence(0)
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
    
    fn parse_expression_with_precedence(&mut self, current_level: u8) -> UASTExpression {
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
            current_expr = UASTExpression::BinOp { op, left: Box::new(current_expr), right: Box::new(next_expr) };
        }
        current_expr
    }
    
    fn parse_expression_atom(&mut self) -> UASTExpression {
         match self.tokens.next() {
            Some(Token::IntLiteral(int)) => {UASTExpression::IntLiteral(int)},
            Some(Token::Identifier(name)) => {
                if self.tokens.peek() == Some(&Token::LeftParen) {        // Function call 
                    self.tokens.next();
                    let args = self.parse_funccall_args(); 
                    self.expect_unparametric_token(Token::RightParen);
                    UASTExpression::FuncCall { funcname: name, args: args}
                } else {
                    UASTExpression::Variable(name)
                }
            },
            Some(Token::LeftParen) => {
                let paren_expr = self.parse_expression();
                self.expect_unparametric_token(Token::RightParen);
                paren_expr
            },
            Some(Token::True) => {UASTExpression::BoolTrue},
            Some(Token::False) => {UASTExpression::BoolFalse},
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

