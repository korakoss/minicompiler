use std::collections::BTreeMap;
use std::collections::HashMap;
use std::iter::Peekable;

use crate::shared::tokens::*;
use crate::shared::typing::*;
use crate::stages::ast::*;

pub struct Parser {
    tokens: Peekable<std::vec::IntoIter<Token>>, 
    new_types: HashMap<TypeIdentifier, DeferredDerivType>,
    functions: HashMap<FuncSignature<DeferredType>, UASTFunction>,
}


impl Parser {

    pub fn parse_program(tokens: Vec<Token>) -> UASTProgram {
        let mut parser = Parser {
            tokens: tokens.into_iter().peekable(),
            new_types: HashMap::new(),
            functions: HashMap::new(),              
        };
        while !parser.tokens.peek().is_none() {
            match parser.tokens.peek().unwrap() {
                &Token::Struct => {
                    parser.process_struct_typedef(); 
                }
                &Token::Function => {
                    parser.process_function_definition();
                }
                _ => {panic!("Invalid token, expected struct or func def");}
            }
        }
        UASTProgram { 
            new_types: parser.new_types,
            functions: parser.functions,
        } 
    }
    
    fn process_struct_typedef(&mut self) {
        self.expect_unparametric_token(Token::Struct);
        let struct_identifier = TypeIdentifier(self.expect_identifier());
        self.expect_unparametric_token(Token::LeftBrace);
        let mut fields = BTreeMap::new();
        while self.tokens.peek() != Some(&Token::RightBrace) {
            let field_name = self.expect_identifier();
            self.expect_unparametric_token(Token::Colon);
            let field_type = self.expect_deferred_type();
            self.expect_unparametric_token(Token::Comma);
            fields.insert(field_name, field_type);
        }
        self.expect_unparametric_token(Token::RightBrace);
        let struct_type = DeferredDerivType::Struct { 
            fields: fields 
        };
        self.new_types.insert(struct_identifier, struct_type); 
    }
       
    fn process_function_definition(&mut self) {
        self.expect_unparametric_token(Token::Function);
        let funcname = self.expect_identifier(); 

        self.expect_unparametric_token(Token::LeftParen);
        let args:HashMap<String, DeferredType> = match self.tokens.peek().unwrap() {
            &Token::RightParen => {
                HashMap::new()
            }
            &Token::Identifier(_) => {
                let name1 = self.expect_identifier();
                self.expect_unparametric_token(Token::Colon);
                let typ1 = self.expect_deferred_type();
                let mut args = HashMap::new();
                args.insert(name1, typ1);

                while self.tokens.peek().unwrap() == &Token::Comma {
                    self.tokens.next();
                    let arg_name = self.expect_identifier();
                    self.expect_unparametric_token(Token::Colon);
                    let arg_type = self.expect_deferred_type();
                    args.insert(arg_name, arg_type);
                }
                args
            }
            _ => {
                panic!("Unexpected token encountered during function arg parsing");
            }
        };
        self.expect_unparametric_token(Token::RightParen);

        let ret_type_id = match self.tokens.peek().unwrap() {
            Token::RightArrow => {
                self.tokens.next();
                self.expect_deferred_type()
            },
            _ => {
                DeferredType::Prim(PrimitiveType::None)
            }
        };
        let body = self.parse_statement_block();
        let func = UASTFunction {name: funcname, args: args, body: body, ret_type: ret_type_id};
        let sgn: FuncSignature<DeferredType> = func.get_signature();
        self.functions.insert(sgn, func);
    }
    
    fn parse_statement_block(&mut self) -> Vec<UASTStatement> {
        let mut statements = Vec::new();
        self.expect_unparametric_token(Token::LeftBrace);
        while !matches!(self.tokens.peek(), Some(Token::RightBrace)){
            statements.push(self.parse_statement());
        }
        self.expect_unparametric_token(Token::RightBrace);
        statements
    }
    
    fn parse_statement(&mut self) -> UASTStatement {
        match self.tokens.peek().unwrap() {           
            &Token::Let => {
                self.tokens.next();
                let var_name = self.expect_identifier();         
                self.expect_unparametric_token(Token::Colon);
                let var_type = self.expect_deferred_type();
                let var = DeferredTypeVariable{
                    name: var_name, 
                    typ: var_type
                };
                self.expect_unparametric_token(Token::Assign);
                let value = self.parse_expression();
                self.expect_unparametric_token(Token::Semicolon);
                UASTStatement::Let{var, value}
            }
            &Token::Identifier(_)  => {                
                // Assignment 
                // TODO: an identifier could actually mean funccalls too
                let target = self.parse_lvalue();
                self.expect_unparametric_token(Token::Assign);
                let assign_value = self.parse_expression();
                self.expect_unparametric_token(Token::Semicolon);
                UASTStatement::Assign { 
                    target,
                    value: assign_value
                }
            }
            &Token::If => {
                self.tokens.next();
                let condition = self.parse_expression();
                let if_body = self.parse_statement_block();
                let else_body =  if matches!(self.tokens.peek(), Some(Token::Else)) {
                    self.tokens.next();
                    Some(self.parse_statement_block())
                } else {None};
                UASTStatement::If {condition, if_body, else_body}
            }
            &Token::While => {
                self.tokens.next();
                let cond = self.parse_expression();
                let body = self.parse_statement_block();
                UASTStatement::While { 
                    condition: cond, 
                    body: body,
                }
            }
            &Token::Break => {         
                self.tokens.next();
                self.expect_unparametric_token(Token::Semicolon);
                UASTStatement::Break
            }
            &Token::Continue => {         
                self.tokens.next();
                self.expect_unparametric_token(Token::Semicolon);
                UASTStatement::Continue
            }
            &Token::Return => {
                self.tokens.next();
                let return_expr = self.parse_expression();
                self.expect_unparametric_token(Token::Semicolon);
                UASTStatement::Return(return_expr)
            }
            &Token::Print => {
                self.tokens.next();
                self.expect_unparametric_token(Token::LeftParen);
                let expr = self.parse_expression();
                self.expect_unparametric_token(Token::RightParen);
                self.expect_unparametric_token(Token::Semicolon);        
                UASTStatement::Print(expr)
            },
            _ => {                                      
                panic!("Cannot recognize valid statement starting with token {:?}", self.tokens.next());              
            }
        }
    } 

    fn parse_lvalue(&mut self) -> ASTLValue {
        let root = self.expect_identifier();
        let mut curr_lvalue = ASTLValue::Variable(root);
        while self.tokens.peek().unwrap() == &Token::Dot {
            self.tokens.next();
            let curr_field = self.expect_identifier();
            curr_lvalue = ASTLValue::FieldAccess { 
                of: Box::new(curr_lvalue), 
                field: curr_field 
            };
        }
        curr_lvalue
    }
        
    fn parse_expression(&mut self) -> UASTExpression {
        self.parse_expression_with_precedence(0)
    }
              
    fn parse_expression_with_precedence(&mut self, current_level: usize) -> UASTExpression {
        let mut current_expr = self.parse_expression_atom();
        loop {
            let prec = match self.tokens.peek() {
                Some(Token::Plus | Token::Minus | Token::Multiply | Token::Equals | Token::Less | Token::Modulo | Token::Dot) => {
                    get_connector_precedence(self.tokens.peek().unwrap())
                }
                _ => break,
            };
            if prec < current_level {
                break;
            }
            let conn_token = self.tokens.next().unwrap();  
            current_expr = match conn_token.clone() {
                Token::Dot => {
                    let field = self.expect_identifier();
                    UASTExpression::FieldAccess { 
                        expr: Box::new(current_expr), 
                        field 
                    }
                }
                _ => {
                    let op = map_binop_token(&conn_token);
                    let next_expr = self.parse_expression_with_precedence(prec + 1);
                    UASTExpression::BinOp { op, left: Box::new(current_expr), right: Box::new(next_expr) }
                }
            };
        }
        current_expr
    }
    
    fn parse_expression_atom(&mut self) -> UASTExpression {
        let token = self.tokens.next().unwrap();  
        match token {
            Token::IntLiteral(int) => UASTExpression::IntLiteral(int),
            Token::Identifier(name) => {
                match self.tokens.peek().unwrap() {
                    &Token::LeftParen => {                                                      // FuncCall
                        self.tokens.next();
                        let args: Vec<UASTExpression> = match self.tokens.peek().unwrap() {
                            &Token::RightParen => Vec::new(),
                            _ => {
                                let mut collected_args: Vec<UASTExpression> = Vec::new();
                                collected_args.push(self.parse_expression());
                                while self.tokens.peek().unwrap() == &Token::Comma {
                                    self.tokens.next();
                                    collected_args.push(self.parse_expression());
                                }
                                collected_args
                            }
                        };
                        self.expect_unparametric_token(Token::RightParen);
                        UASTExpression::FuncCall { funcname: name, args: args}
                    }

                    &Token::LeftBrace => {                                                  // StructLiteral
                        if self.new_types.contains_key(&TypeIdentifier(name.clone())) {
                            UASTExpression::StructLiteral{
                                typ: DeferredType::Symbolic(TypeIdentifier(name)),
                                fields: self.parse_struct_literal_internals(),
                            }
                        } else {
                            UASTExpression::Variable(name)
                        }
                    }
                    
                    _ => UASTExpression::Variable(name)
                }
            },
            Token::LeftParen => {
                let paren_expr = self.parse_expression();
                self.expect_unparametric_token(Token::RightParen);
                paren_expr
            },
            Token::True => UASTExpression::BoolTrue,
            Token::False => UASTExpression::BoolFalse,
            _ => {
                panic!("Unexpected token {:?} during expression parsing", token);
            },
        }
    }
    
    fn parse_struct_literal_internals(&mut self) -> HashMap<String, UASTExpression>{
        self.expect_unparametric_token(Token::LeftBrace);
        let mut fields = HashMap::new();
        while self.tokens.peek() != Some(&Token::RightBrace) {
            let field_name = self.expect_identifier();
            self.expect_unparametric_token(Token::Colon);
            let field_value = self.parse_expression();
            self.expect_unparametric_token(Token::Comma);
            fields.insert(field_name, field_value);
        }
        self.expect_unparametric_token(Token::RightBrace);
        fields 
    }

    
    fn expect_unparametric_token(&mut self, expected_token: Token) {
        let token = self.tokens.peek().unwrap();
        if token != &expected_token {
            panic!("Expected token {:?}, got token {:?}.", expected_token, token);
        }
        self.tokens.next();
    }
    
    fn expect_identifier(&mut self) -> String {
        let token = self.tokens.next();
        let Some(Token::Identifier(name)) = token else {
                panic!("Expected identifier token, got token: {:?}", token);
        };
        name
    }
    
    fn expect_deferred_type(&mut self) -> DeferredType {
        let token = self.tokens.next().unwrap(); 
        match token {
            Token::Int => DeferredType::Prim(PrimitiveType::Integer),
            Token::Bool => DeferredType::Prim(PrimitiveType::Bool),
            Token::Identifier(type_id) => DeferredType::Symbolic(TypeIdentifier(type_id)),
            _ => {panic!("Expected primitive type or identifer token, got {:?}", token);}
        }
    }
}



     
