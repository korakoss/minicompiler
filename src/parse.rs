use std::collections::BTreeMap;
use std::collections::HashMap;
use std::iter::Peekable;

use crate::shared::tokens::*;
use crate::shared::typing::*;
use crate::stages::ast::*;

pub struct Parser {
    tokens: Peekable<std::vec::IntoIter<Token>>, 
    new_types: HashMap<TypeIdentifier, TypeConstructor>,
    functions: HashMap<FuncSignature, ASTFunction>,
}


impl Parser {

    pub fn parse_program(tokens: Vec<Token>) -> ASTProgram {
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
        ASTProgram { 
            typetable: TypeTable::make(parser.new_types),
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
            let field_type = self.expect_type();
            self.expect_unparametric_token(Token::Comma);
            fields.insert(field_name, field_type);
        }
        self.expect_unparametric_token(Token::RightBrace);
        let struct_type = TypeConstructor::Struct { 
            fields: fields 
        };
        self.new_types.insert(struct_identifier, struct_type); 
    }
       
    fn process_function_definition(&mut self) {
        self.expect_unparametric_token(Token::Function);
        let funcname = self.expect_identifier(); 

        self.expect_unparametric_token(Token::LeftParen);
        let args:HashMap<String, Type> = match self.tokens.peek().unwrap() {
            &Token::RightParen => {
                HashMap::new()
            }
            &Token::Identifier(_) => {
                let name1 = self.expect_identifier();
                self.expect_unparametric_token(Token::Colon);
                let typ1 = self.expect_type();
                let mut args = HashMap::new();
                args.insert(name1, typ1);

                while self.tokens.peek().unwrap() == &Token::Comma {
                    self.tokens.next();
                    let arg_name = self.expect_identifier();
                    self.expect_unparametric_token(Token::Colon);
                    let arg_type = self.expect_type();
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
                self.expect_type()
            },
            _ => {
                Type::Prim(PrimType::None)
            }
        };
        let body = self.parse_statement_block();
        let func = ASTFunction {name: funcname, args: args, body: body, ret_type: ret_type_id};
        let sgn = func.get_signature();
        self.functions.insert(sgn, func);
    }
    
    fn parse_statement_block(&mut self) -> Vec<ASTStatement> {
        let mut statements = Vec::new();
        self.expect_unparametric_token(Token::LeftBrace);
        while !matches!(self.tokens.peek(), Some(Token::RightBrace)){
            statements.push(self.parse_statement());
        }
        self.expect_unparametric_token(Token::RightBrace);
        statements
    }
    
    fn parse_statement(&mut self) -> ASTStatement {
        match self.tokens.peek().unwrap() {           
            &Token::Let => {
                self.tokens.next();
                let var_name = self.expect_identifier();         
                self.expect_unparametric_token(Token::Colon);
                let var_type = self.expect_type();
                let var = Variable{
                    name: var_name, 
                    typ: var_type
                };
                self.expect_unparametric_token(Token::Assign);
                let value = self.parse_expression();
                self.expect_unparametric_token(Token::Semicolon);
                ASTStatement::Let{var, value}
            }
            &Token::Identifier(_)  => {                
                // Assignment 
                // TODO: an identifier could actually mean funccalls too
                let target = self.parse_lvalue();
                self.expect_unparametric_token(Token::Assign);
                let assign_value = self.parse_expression();
                self.expect_unparametric_token(Token::Semicolon);
                ASTStatement::Assign { 
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
                ASTStatement::If {condition, if_body, else_body}
            }
            &Token::While => {
                self.tokens.next();
                let cond = self.parse_expression();
                let body = self.parse_statement_block();
                ASTStatement::While { 
                    condition: cond, 
                    body: body,
                }
            }
            &Token::Break => {         
                self.tokens.next();
                self.expect_unparametric_token(Token::Semicolon);
                ASTStatement::Break
            }
            &Token::Continue => {         
                self.tokens.next();
                self.expect_unparametric_token(Token::Semicolon);
                ASTStatement::Continue
            }
            &Token::Return => {
                self.tokens.next();
                let return_expr = self.parse_expression();
                self.expect_unparametric_token(Token::Semicolon);
                ASTStatement::Return(return_expr)
            }
            &Token::Print => {
                self.tokens.next();
                self.expect_unparametric_token(Token::LeftParen);
                let expr = self.parse_expression();
                self.expect_unparametric_token(Token::RightParen);
                self.expect_unparametric_token(Token::Semicolon);        
                ASTStatement::Print(expr)
            },
            _ => {                                      
                panic!("Cannot recognize valid statement starting with token {:?}", self.tokens.next());              
            }
        }
    } 

    fn parse_lvalue(&mut self) -> ASTLValue {
        let root = self.expect_identifier();
        let mut curr_lvalue = ASTLValue::Variable(root);

        if self.tokens.peek().unwrap() == &Token::Deref {
            self.tokens.next();
            let refd_expr = self.parse_expression();
            return ASTLValue::Deref(refd_expr)
        }

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
        
    fn parse_expression(&mut self) -> ASTExpression {
        self.parse_expression_with_precedence(0)
    }
              
    fn parse_expression_with_precedence(&mut self, current_level: usize) -> ASTExpression {
        let mut current_expr = self.parse_unary();
        loop {
            let token = self.tokens.peek().unwrap();
            let prec = match token {
                Token::Plus | Token::Minus | Token::Multiply | Token::Equals | Token::Less | Token::Modulo  => {
                    get_connector_precedence(self.tokens.peek().unwrap())
                }
                _ => break,
            };
            if prec < current_level {
                break;
            }
            let conn_token = self.tokens.next().unwrap();  
            current_expr = match conn_token {
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

    fn parse_unary(&mut self) -> ASTExpression {
        match self.tokens.peek().unwrap() {
            &Token::Ref => {
                self.tokens.next();
                let refd = self.parse_unary();
                ASTExpression::Reference(Box::new(refd))
            }
            &Token::Deref => {
                self.tokens.next();
                let derefd = self.parse_unary();
                ASTExpression::Dereference(Box::new(derefd))
            }
            _ => self.parse_postfix(),
        }
    }

    fn parse_postfix(&mut self) -> ASTExpression {
        let mut curr_expr = self.parse_expression_atom();
        while self.tokens.peek().unwrap() == &Token::Dot {
            self.tokens.next();
            let field = self.expect_identifier();
            curr_expr = ASTExpression::FieldAccess { 
                expr: Box::new(curr_expr), 
                field 
            };
        }
        curr_expr
    }
    
    fn parse_expression_atom(&mut self) -> ASTExpression {
        let token = self.tokens.next().unwrap();  
        match token {
            Token::IntLiteral(int) => ASTExpression::IntLiteral(int),
            Token::Identifier(name) => {
                match self.tokens.peek().unwrap() {
                    &Token::LeftParen => {                                                      // FuncCall
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
                    }

                    &Token::LeftBrace => {                                                  // StructLiteral
                        if self.new_types.contains_key(&TypeIdentifier(name.clone())) {
                            ASTExpression::StructLiteral{
                                typ: Type::NewType(TypeIdentifier(name)),
                                fields: self.parse_struct_literal_internals(),
                            }
                        } else {
                            ASTExpression::Variable(name)
                        }
                    }
                    
                    _ => ASTExpression::Variable(name)
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
    
    fn parse_struct_literal_internals(&mut self) -> HashMap<String, ASTExpression>{
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
    
    fn expect_type(&mut self) -> Type {
        match self.tokens.next().unwrap() {
            Token::Int => {
                Type::Prim(PrimType::Integer)
            }
            Token::Bool => {
                Type::Prim(PrimType::Bool)
            }
            Token::Identifier(type_id) => {
                Type::NewType(TypeIdentifier(type_id))
            }
            Token::Ref => {
                let refd_type = self.expect_type();
                Type::Reference(Box::new(refd_type))
            }
            _ => {
                panic!("Unexpected token while parsing type annotation");
            }
        }
    }
}



     
