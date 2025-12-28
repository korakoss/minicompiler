use std::collections::BTreeMap;
use std::collections::HashMap;
use std::iter::Peekable;

use crate::ast::*;
use crate::common::*;


pub struct Parser {
    tokens: Peekable<std::vec::IntoIter<Token>>, 
    defined_funcs: Vec<UASTFunction>,           // Could be a hashmap indexed by sign?
    new_types: HashMap<TypeIdentifier, DeferredNewType>
}


impl Parser {

    pub fn parse_program(tokens: Vec<Token>) -> UASTProgram {
        let mut parser = Parser {
            tokens: tokens.into_iter().peekable(),
            defined_funcs: Vec::new(),              
            new_types: HashMap::new(),
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
            functions: parser.defined_funcs,
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
        let struct_type = DeferredNewType::Struct { 
            fields: fields 
        };
        self.new_types.insert(struct_identifier, struct_type); 
    }
       
    fn process_function_definition(&mut self) {
        self.expect_unparametric_token(Token::Function);
        let funcname = self.expect_identifier(); 
        let mut args = Vec::new();
        self.expect_unparametric_token(Token::LeftParen);

        while self.tokens.peek() != Some(&Token::RightParen) {         // TODO: much later, this might not vibe with tuples maybe?
            let arg_name = self.expect_identifier();
            self.expect_unparametric_token(Token::Colon);
            let arg_type = self.expect_deferred_type(); 
            args.push(DeferredTypeVariable{
                name: arg_name, 
                typ: arg_type,
            });
            if self.tokens.peek() == Some(&Token::Comma) {
                self.tokens.next();
            }
        }
        self.expect_unparametric_token(Token::RightParen);
        let ret_type_id = match self.tokens.peek() {
            Some(Token::RightArrow) => {
                self.tokens.next();
                self.expect_deferred_type()
            },
            _ => {
                DeferredType::Resolved(Type::Primitive(PrimitiveType::None)) 
            }
        };
        let body = self.parse_statement_block();
        let func = UASTFunction {name: funcname, args: args, body: body, ret_type: ret_type_id};
        self.defined_funcs.push(func);
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
        let token = self.tokens.next().unwrap();  
        match token {           
            Token::Let => {
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
            Token::Identifier(_) | Token::LeftParen => {
                // Assignment
                let expr = self.parse_expression();
                self.expect_unparametric_token(Token::Assign);
                let assign_value = self.parse_expression();
                self.expect_unparametric_token(Token::Semicolon);
                UASTStatement::Assign { target: expr, value: assign_value}
            }
            Token::If => {
                let condition = self.parse_expression();
                let if_body = self.parse_statement_block();
                let else_body =  if matches!(self.tokens.peek(), Some(Token::Else)) {
                    Some(self.parse_statement_block())
                } else {None};
                UASTStatement::If {condition, if_body, else_body}
            }
            Token::While => {
                let cond = self.parse_expression();
                let body = self.parse_statement_block();
                UASTStatement::While { 
                    condition: cond, 
                    body: body,
                }
            }
            Token::Break => {         
                self.expect_unparametric_token(Token::Semicolon);
                UASTStatement::Break
            }
            Token::Continue => {         
                self.expect_unparametric_token(Token::Semicolon);
                UASTStatement::Continue
            }
            Token::Return => {
                let return_expr = self.parse_expression();
                self.expect_unparametric_token(Token::Semicolon);
                UASTStatement::Return(return_expr)
            }
            Token::Print => {
                self.expect_unparametric_token(Token::LeftParen);
                let expr = self.parse_expression();
                self.expect_unparametric_token(Token::RightParen);
                self.expect_unparametric_token(Token::Semicolon);        
                UASTStatement::Print(expr)
            },
            _ => {                                      
                panic!("Cannot recognize valid statement starting with token {:?}", token);              
            }
        }
    } 
        
    fn parse_expression(&mut self) -> ASTExpression {
        match self.tokens.peek().unwrap() {
            Token::LeftBrace => {
                self.parse_struct_literal()
            }
            _ => {
                self.parse_expression_with_precedence(0)
            }
        }
    }

    fn parse_struct_literal(&mut self) -> ASTExpression {
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
        ASTExpression::StructLiteral{fields} 
    }
          
    fn parse_expression_with_precedence(&mut self, current_level: usize) -> ASTExpression {
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
                    ASTExpression::FieldAccess { 
                        expr: Box::new(current_expr), 
                        field 
                    }
                }
                _ => {
                    let op = map_binop_token(&conn_token);
                    let next_expr = self.parse_expression_with_precedence(prec + 1);
                    ASTExpression::BinOp { op, left: Box::new(current_expr), right: Box::new(next_expr) }
                }
            };
        }
        current_expr
    }
    
    fn parse_expression_atom(&mut self) -> ASTExpression {
        let token = self.tokens.next().unwrap();  
        match token {
            Token::IntLiteral(int) => ASTExpression::IntLiteral(int),
            Token::Identifier(name) => {
                if self.tokens.peek() == Some(&Token::LeftParen) {        // Function call 
                    self.tokens.next();
                    let args: Vec<ASTExpression> = match self.tokens.peek().unwrap() {
                        &Token::RightParen => Vec::new(),
                        _ => {
                            let mut collected_args: Vec<ASTExpression> = Vec::new();
                            collected_args.push(self.parse_expression());
                            while self.tokens.peek().unwrap() == &Token::Comma {
                                self.tokens.next();
                                collected_args.push(self.parse_expression());
                            }
                            collected_args
                        }
                    };
                    self.expect_unparametric_token(Token::RightParen);
                    ASTExpression::FuncCall { funcname: name, args: args}
                } else {
                    ASTExpression::Variable(name)
                }
            },
            Token::LeftParen => {
                let paren_expr = self.parse_expression();
                self.expect_unparametric_token(Token::RightParen);
                paren_expr
            },
            Token::True => ASTExpression::BoolTrue,
            Token::False => ASTExpression::BoolFalse,
            _ => {
                panic!("Unexpected token {:?} during expression parsing", token);
            },
        }
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
        match token {
            Some(Token::Identifier(name)) => {
                return name;
            }
            _ => {
                panic!("Expected identifier token, got token: {:?}", token);
            }
        }
    }

    
    fn expect_deferred_type(&mut self) -> DeferredType {
        let token = self.tokens.next().unwrap(); 
        match token {
            Token::Int => DeferredType::Resolved(Type::Primitive(PrimitiveType::Integer)),
            Token::Bool => DeferredType::Resolved(Type::Primitive(PrimitiveType::Bool)),
            Token::Identifier(type_id) => DeferredType::Unresolved(TypeIdentifier(type_id)),
            _ => {panic!("Expected primitive type or identifer token, got {:?}", token);}
        }
    }
}



     
