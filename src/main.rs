use std::collections::HashMap;


#[derive(Debug)]
enum BinaryOperationType {
    Add, 
    Sub, 
    //Mul, 
    //Div,
    //Less, 
    //Greater, 
    //Equal, 
    //NotEqual
}

#[derive(Debug)]
enum Expression {
    IntLiteral(i32),

    Variable(String),

    BinOp {
       op: BinaryOperationType,
       left: Box<Expression>,
       right: Box<Expression>,
    },
}

#[derive(Debug)]
enum Statement {
    
    Assign {
        varname: String,
        value: Box<Expression>
    },

    Print(Box<Expression>),
}

#[derive(Debug)]
struct Program {
    statements: Vec<Statement>
}


#[derive(Debug, Clone)]
enum Token {
    // Operators
    Assign,
    Plus,
    Minus,
    
    // Delimiters
    Semicolon,
    LeftParen,
    RightParen,
    
    // Values (with associated data)
    IntLiteral(i32),
    Identifier(String),
    
    // Keywords
    Print,
    
    // Special
    EOF,
}


fn lex(program: &str) -> Vec<Token> {
    let mut tokens: Vec<Token> = Vec::new();
    let mut chars = program.chars().peekable();
    
    while let Some(&c) = chars.peek() {
        // Skip whitespace
        if c.is_whitespace() {
            chars.next();
            continue;
        }
        
        // Single-character tokens
        if c == '=' {
            tokens.push(Token::Assign);
            chars.next();
        } else if c == '+' {
            tokens.push(Token::Plus);
            chars.next();
        } else if c == '-' {
            tokens.push(Token::Minus);
            chars.next();
        } else if c == ';' {
            tokens.push(Token::Semicolon);
            chars.next();
        } else if c == '(' {
            tokens.push(Token::LeftParen);
            chars.next();
        } else if c == ')' {
            tokens.push(Token::RightParen);
            chars.next();
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
        // Identifiers and keywords
        else if c.is_alphabetic() {
            let mut ident = String::new();
            while let Some(&ch) = chars.peek() {
                if ch.is_alphanumeric() || ch == '_' {
                    ident.push(ch);
                    chars.next();
                } else {
                    break;
                }
            }
            
            // Check if it's a keyword
            let token = match ident.as_str() {
                "print" => Token::Print,
                _ => Token::Identifier(ident),
            };
            tokens.push(token);
        }
        // Unknown character
        else {
            panic!("Unexpected character: {}", c);
        }
    }
    
    tokens.push(Token::EOF);
    tokens
}


struct Parser {
    tokens: Vec<Token>,
    position: usize,
}


impl Parser {
    
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

    fn parse_expression(&mut self) -> Expression {

        let expr = match self.peek() {
            
            Token::LeftParen => {
                self.consume();
                let expr = self.parse_expression();
                if !matches!(self.consume(), Token::RightParen) {
                    panic!("Expected ')'");
                } 
                expr
            }

            Token::IntLiteral(_) => {
                let Token::IntLiteral(int) = self.consume() else {panic!("Expected int literal")};
                Expression::IntLiteral(int)
            }
            
            Token::Identifier(_) => {
                let Token::Identifier(name) = self.consume() else {panic!("Expected identifier")};
                Expression::Variable(name)
            }

            _ => {
                let tok = self.consume();
                panic!("Unexpected token while parsing expression: {:?}", tok);
            }
        };

        let result = match self.peek() {

            Token::Plus => {
                self.consume();
                let second_expr = self.parse_expression();
                Expression::BinOp { op: BinaryOperationType::Add , left: Box::new(expr), right: Box::new(second_expr)}
            }
            
            Token::Minus => {
                self.consume();
                let second_expr = self.parse_expression();
                Expression::BinOp { op: BinaryOperationType::Sub, left: Box::new(expr), right: Box::new(second_expr)}
            }
            _ => {
                expr 
            }
        };
        result
    }

    fn parse_statement(&mut self) -> Statement {
        match self.peek() {
            Token::Print => {
                self.consume();  
                
                if !matches!(self.consume(), Token::LeftParen) {
                    panic!("Expected '(' after print");
                }

                let expr = self.parse_expression();
                
                if !matches!(self.consume(), Token::RightParen) {
                    panic!("Expected ')'");
                }

                if !matches!(self.consume(), Token::Semicolon) {
                    panic!("Expected ';'");
                }

                Statement::Print(Box::new(expr))
            }
            
            Token::Identifier(_) => {
                let varname = match self.consume() {
                    Token::Identifier(name) => name,
                    _ => unreachable!()
                };
                
                if !matches!(self.consume(), Token::Assign) {
                    panic!("Expected '='");
                }

                let expr = self.parse_expression();

                if !matches!(self.consume(), Token::Semicolon) {
                    panic!("Expected ';'");
                }

                Statement::Assign {
                    varname,
                    value: Box::new(expr)
                }
            }
            
            _ => panic!("Expected statement")
        }
    }    

    fn parse_program(&mut self) -> Program {
        let mut statements = Vec::new();
        
        while !self.is_at_end() {
            statements.push(self.parse_statement());
        }
        
        Program { statements }
    }
}


struct Compiler {
    output: String,
    stack_offsets: HashMap<String, i32>,
}

impl Compiler {

    fn emit(&mut self, line: &str) {
        self.output.push_str(line);
        self.output.push('\n');
    }

    fn compile_expression(&mut self, expression: &Expression) {
       
        match expression {
     
            Expression::IntLiteral(n) => {
                self.emit(&format!("    mov x0,#{}", n));   // Load the value into the main
                                                            // register 

            }

            Expression::Variable(varname) => {
                let offset = self.stack_offsets.get(varname).expect("Undefined variable");
                self.emit(&format!("    ldr x0, [x29, #-{}]", offset));    // 
            }

            Expression::BinOp{op, left, right} => {
                self.compile_expression(left);
                self.emit("     str x0, [sp, #-16]!"); // Store left's value on stack
                self.compile_expression(right);
                self.emit("     ldr x1, [sp], #16");

                match op {
                    BinaryOperationType::Add => {
                        self.emit("     add x0, x1, x0");
                    }

                    BinaryOperationType::Sub => {
                        self.emit("     sub x0, x1, x0");   // x1-x0 because x1: left x0:right
                    }
                }
            }
        }

    }

    fn compile_statement(&mut self, statement: &Statement) {
            
        match statement {
            
            Statement::Assign { varname, value } => {
                let var_offset = self.stack_offsets.get(varname).expect("Undefined variable");
                self.compile_expression(expression);
                self.emit(&format!("     str x0, [x29, #-{}]", var_offset));
            }
        }
    }

    fn compile_program(&mut self, program: Program) -> String {
        // emit header
        //emit prologue
        
        for statement in &program.statements {
            self.compile_statement(statement);
        }

        //emit epilogue
        self.output  
    }
}


fn main() {

    let program_text = "x=5; y = x + 3; print(42);"; 
    let tokens = lex(program_text);
    //println!("Tokens:");
    //for token in &tokens {
    //    println!("  {:?}", token);
    //}

    let mut parser = Parser {tokens, position: 0};
    let program = parser.parse_program();
    println!(" {:?}", program);
}


