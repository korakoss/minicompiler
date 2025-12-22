use crate::lex::*;
use crate::ast::*;
use crate::common::*;


pub struct Parser {
    tokens: Vec<Token>,
    position: usize,
    defined_funcs: Vec<ASTFunction>,
}


impl Parser {
    
    pub fn new(tokens: Vec<Token>) -> Self {
        Parser {
            tokens: tokens,
            position: 0,
            defined_funcs: Vec::new(),
        }
    }

    pub fn parse_program(mut self) -> ASTProgram {
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
        ASTProgram { functions: self.defined_funcs, main_statements: statements}
    }

    // TODO: can't these 3 functions be substituted by .peekable of whatever? 
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
    
    fn parse_funccall_args(&mut self) -> Vec<Box<ASTExpression>> {
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

    fn parse_primitive(&mut self) -> ASTExpression {
        
         match self.consume() {
            Token::IntLiteral(int) => {ASTExpression::IntLiteral(int)},
            Token::Identifier(name) => {
                if self.peek() == &Token::LeftParen {        // Function call 
                    self.consume();
                    let args = self.parse_funccall_args(); 
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
            _ => panic!("Expected binary operator token"),
        }
    }
    
    // TODO: clarify logic
    fn parse_expression_with_precedence(&mut self, current_level: i8) -> ASTExpression {
        
        let mut current_expr = self.parse_primitive(); 

        while matches!(self.peek(), Token::Plus | Token::Minus | Token::Multiply | Token::Equals | Token::Less | Token::Modulo) {
            let prec = self.get_binop_precedence(self.peek().clone());
            if prec < current_level {
                break;
            }
            let optoken = self.consume();
            let op = self.map_binop_token(optoken);
            let next_expr = self.parse_expression_with_precedence(prec+1);
            current_expr = ASTExpression::BinOp { op, left: Box::new(current_expr), right: Box::new(next_expr)};

        }
        current_expr 
    }

    fn parse_expression(&mut self) -> ASTExpression {
        self.parse_expression_with_precedence(0)
    }
    
    fn expect_block(&mut self) -> Vec<ASTStatement> {
        let mut statements = Vec::new();
        self.expect_unparametric_token(Token::LeftBrace);
        while !matches!(self.peek(), Token::RightBrace){
            statements.push(self.parse_statement());
        }
        self.expect_unparametric_token(Token::RightBrace);
        statements
    }

    fn parse_type(&mut self) -> Type {
        match self.consume() {
            Token::Int => {Type::Integer},
            Token::Bool => {Type::Bool},
            _ => {panic!("Unexpected token in typing");}
        }
    }

    fn parse_statement(&mut self) -> ASTStatement {
        
        if self.is_expression_start() {
            // TODO: later we can do trailing-expr return here
            let expr = self.parse_expression();
            self.expect_unparametric_token(Token::Assign);
            let assign_value = self.parse_expression();
            self.expect_unparametric_token(Token::Semicolon);
            return ASTStatement::Assign { target: expr, value: assign_value};
        }
        match self.consume() {
                        
            Token::If => {
                let cond = self.parse_expression();
                let body = self.expect_block();
                
                if matches!(self.peek(), Token::Else) {
                    let else_body = self.expect_block();
                    return ASTStatement::If{
                        condition: cond,
                        if_body: body,
                        else_body: Some(else_body),
                    }
   
                } else {
                    return ASTStatement::If { 
                        condition: cond, 
                        if_body: body,
                        else_body: None,
                    }
                }
            }

            Token::While => {
                let cond = self.parse_expression();
                let body = self.expect_block();
                ASTStatement::While { 
                    condition: cond, 
                    body: body,
                }
            }

            Token::Break => {         
                self.expect_unparametric_token(Token::Semicolon);
                ASTStatement::Break
            }
            
            Token::Continue => {         
                self.expect_unparametric_token(Token::Semicolon);
                ASTStatement::Continue
            }

            Token::Return => {
                let return_expr = self.parse_expression();
                self.expect_unparametric_token(Token::Semicolon);
                ASTStatement::Return(return_expr)
            }

            Token::Print => {
                self.expect_unparametric_token(Token::LeftParen);
                let expr = self.parse_expression();
                self.expect_unparametric_token(Token::RightParen);
                self.expect_unparametric_token(Token::Semicolon);        
                ASTStatement::Print(expr)
            },

            Token::Let => {
                let varname = self.expect_identifier_token();         
                self.expect_unparametric_token(Token::Colon);
                let typ = self.parse_type();
                let var = Variable{name: varname, typ: typ};
                self.expect_unparametric_token(Token::Assign);
                let value = self.parse_expression();
                self.expect_unparametric_token(Token::Semicolon);
                ASTStatement::Let { var: var, value}
            },
            
            other => {                                      
                panic!("Cannot recognize valid statement at position {:?} with token {:?}", self.position, other);
            }
        }
    }    

    fn parse_function_declaration(&mut self) -> ASTFunction {
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
            args.push(Variable{name: argname, typ: argtype});
                            
            while self.peek() == &Token::Comma {
                self.consume();  
                let argname = self.expect_identifier_token();
                self.expect_unparametric_token(Token::Colon);
                let argtype = self.parse_type();
                args.push(Variable{name: argname, typ: argtype});

            }
            self.expect_unparametric_token(Token::RightParen);
        }
        self.expect_unparametric_token(Token::RightArrow);
        let ret_type = self.parse_type();
        let body = self.expect_block();
        ASTFunction {name: funcname, args: args, body: body, ret_type}
    }

}


