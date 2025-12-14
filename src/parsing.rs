use crate::ast::*;


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
    
    // Inteded use is for unparametric Tokens (normally delimiters). TODO: might check if we can make this work for parametrics, dunno if needed
    fn expect_token(&mut self, expected_token: Token) {
        if self.peek() != &expected_token {
            panic!("Expected token {:?} at position {}, got token {:?}.", expected_token, self.position, self.peek()); 
        }
        self.consume();
    }

    fn is_expression_start(&self) -> bool {
        matches!(self.peek(), Token::IntLiteral(_) | Token::Identifier(_) | Token::LeftParen)
    }
    
    fn parse_funccall_args(&mut self) -> Vec<Box<Expression>> {
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
                    self.expect_token(Token::RightParen);
                    Expression::FuncCall { funcname: name, args: args}
                } else {
                    Expression::Variable(name)
                }
            },
            Token::LeftParen => {
                let paren_expr = self.parse_expression();
                self.expect_token(Token::RightParen);
                paren_expr
            },
            _ => {
                let tok = self.consume();
                panic!("Unexpected token {:?} during expression parsing at position {:?}", tok, self.position);   // WRONG -- already consumed
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

    fn convert_binop(&mut self, op_token: Token) -> BinaryOperator {
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
            let op = self.convert_binop(optoken);
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

    fn parse_statement(&mut self) -> Statement {
        
        match self.consume() {
                        
            Token::Identifier(varname) => {
                self.expect_token(Token::Assign); 
                let expr = self.parse_expression(); 
                self.expect_token(Token::Semicolon);
                if self.within_function_body {
                    self.loc_vars.push(varname.clone());                
                } else {
                    self.glob_vars.push(varname.clone());
                }

                Statement::Assign {
                    varname,
                    value: expr
                }
            }
            Token::If => {
                let cond = self.parse_expression();
                self.expect_token(Token::LeftBrace);
                let body = self.parse_block();
                self.expect_token(Token::RightBrace);
                
                if matches!(self.peek(), Token::Else) {
                    self.expect_token(Token::LeftBrace);
                    let else_body = self.parse_block();
                    self.expect_token(Token::RightBrace);
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
                self.expect_token(Token::LeftBrace);
                self.loop_nest_counter = self.loop_nest_counter + 1;
                let body = self.parse_block();
                self.expect_token(Token::RightBrace);
                self.loop_nest_counter = self.loop_nest_counter - 1;
                Statement::While { 
                    condition: cond, 
                    body: body,
                }
            }

            Token::Break => {         
                if self.loop_nest_counter > 0 {
                    self.expect_token(Token::Semicolon);
                    Statement::Break
                } else {
                    panic!("Break statement detected outside of any loop body at position {}", self.position);
                }
            }
            
            Token::Continue => {         
                if self.loop_nest_counter > 0 {
                    self.expect_token(Token::Semicolon);
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
                self.expect_token(Token::Semicolon);
                Statement::Return(return_expr)
            }

            Token::Print => {
                self.expect_token(Token::LeftParen);
                let expr = self.parse_expression();
                self.expect_token(Token::RightParen);
                self.expect_token(Token::Semicolon);        
                Statement::Print(expr)
            }

            other => {panic!("Expected statement, got: {:?} at position {}", other, self.position);}
        }
    }    

    fn parse_function_declaration(&mut self) -> Function {
        let funcname = match self.consume(){
            Token::Identifier(name) => {name},
            other => {panic!("Expected a function name, got token {:?} at {}", other, self.position);}
        };

        let mut args = Vec::new();
        self.expect_token(Token::LeftParen);
        if self.peek() == &Token::RightParen {
            self.consume();
        } else {
            match self.consume() {
                Token::Identifier(name) => args.push(name),
                other => panic!("Expected parameter name, got {:?}", other),
            }
                
            while self.peek() == &Token::Comma {
                self.consume();  
                match self.consume() {
                    Token::Identifier(name) => args.push(name),
                    other => panic!("Expected parameter name after comma, got {:?}", other),
                }
            }
            self.expect_token(Token::RightParen);
        }
        self.expect_token(Token::LeftBrace);
        self.within_function_body = true;
        let body = self.parse_block();
        self.expect_token(Token::RightBrace);
        self.within_function_body = false;
        Function {name: funcname, args: args, body: body}
    }

    pub fn parse_program(mut self) -> Program {
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
        Program { functions: self.defined_funcs, main_statements: statements}
    }
}

