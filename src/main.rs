use std::collections::HashMap;
use std::fs;
use std::env;


#[derive(Debug)]
enum BinaryOperationType {
    Add, 
    Sub, 
    Mul, 
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

    Branch {
        cond: Box<Expression>,
        then_branch: Box<Statement>,
        else_branch: Box<Statement>,
    }

    //Print(Box<Expression>),
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
    Multiply,

    // Delimiters
    Semicolon,
    LeftParen,
    RightParen,
    LeftBrace,
    RightBrace,
    
    // Values 
    IntLiteral(i32),
    Identifier(String),
    
    // Keywords
    //Print,
    If,
    
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
        } else if c=='*' {
            tokens.push(Token::Multiply);
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
        } else if c == '{' {
            tokens.push(Token::LeftBrace);
            chars.next();
        } else if c == '}' {
            tokens.push(Token::RightBrace);
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

        // Identifiers
        // TODO: keywords would come here later
        else if c.is_ascii_alphabetic() {
            let mut word = String::new();
            while let Some(&ch) = chars.peek() {
                if ch.is_ascii_alphanumeric() || ch == '_' {
                    word.push(ch);
                    chars.next();
                } else {
                    break;
                }
            }
            if word == "if" {
                tokens.push(Token::If);
            } else {
                tokens.push(Token::Identifier(word));
            } 
        }

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

    fn parse_primitive(&mut self) -> Expression {
        
         match self.consume() {
            
            Token::IntLiteral(int) => {Expression::IntLiteral(int)}

            Token::Identifier(name) => {Expression::Variable(name)}
            
            Token::LeftParen => {
                let paren_expr = self.parse_expression();
                if !matches!(self.consume(), Token::RightParen) {
                    panic!("Expected ')'");
                } 
                paren_expr
            }

            _ => {
                let tok = self.consume();
                panic!("Unexpected token while parsing expression: {:?}", tok);
            }
        }
    }

    fn get_binop_precedence(&mut self, op_token: Token) -> i8 {
        match op_token {
            Token::Plus| Token::Minus => 1,
            Token::Multiply => 2,
            _ => -1, 
        }
    }

    fn convert_binop(&mut self, op_token: Token) -> BinaryOperationType {

        // Can we subtype these enums?
        match op_token {
            Token::Plus => BinaryOperationType::Add,
            Token::Minus => BinaryOperationType::Sub,
            Token::Multiply => BinaryOperationType::Mul,
            _ => panic!("Expected binary operator toke"),
        }
    }
   
    fn parse_expression_with_precedence(&mut self, current_level: i8) -> Expression {
        
        let mut current_expr = self.parse_primitive(); 

        while matches!(self.peek(), Token::Plus | Token::Minus | Token::Multiply) {
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

    fn parse_statement(&mut self) -> Statement {
        
        match self.peek() {
                        
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
            
            other => {panic!("Expected statement, got: {:?} at position {}", other, self.position);}
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

    fn new() -> Self {
        Compiler { output: String::new(), stack_offsets: HashMap::new()}
    }

    fn emit(&mut self, line: &str) {
        self.output.push_str(line);
        self.output.push('\n');
    }

    fn compile_expression(&mut self, expression: &Expression) {
       
        match expression {
     
            Expression::IntLiteral(n) => {
                self.emit(&format!("    mov r0,#{}", n));   // Load the value into the main
                                                            // register 

            }

            Expression::Variable(varname) => {
                let offset = self.stack_offsets.get(varname).expect("Undefined variable");
                self.emit(&format!("    ldr r0, [fp, #-{}]", offset));    // 
            }

            Expression::BinOp{op, left, right} => {
                self.compile_expression(left);
                self.emit("    push {r0}");// Store left's value on stack
                self.compile_expression(right);
                self.emit("    pop {r1}");

                match op {
                    BinaryOperationType::Add => {
                        self.emit("    add r0, r1, r0");
                    }
                    BinaryOperationType::Sub => {
                        self.emit("    sub r0, r1, r0");   // x1-x0 because x1: left x0:right
                    }
                    BinaryOperationType::Mul => {
                        self.emit("    mul r0, r1, r0");   
                    }

                }
            }
        }

    }

    fn compile_statement(&mut self, statement: &Statement) {
            
        match statement {
            
            Statement::Assign { varname, value } => {

                self.compile_expression(value);

                if let Some(&var_offset) = self.stack_offsets.get(varname) {
                    // The variable already exists -> we use its address
                    self.emit(&format!("    str r0, [fp, #-{}]", var_offset));
                } else {
                    // New variable: allocate it space on the stack
                    let new_offset = self.stack_offsets.values().max().unwrap_or(&0) + 8;
                    self.stack_offsets.insert(varname.clone(), new_offset);
                    self.emit(&format!("    str r0, [fp, #-{}]", new_offset));
                }
            }
        }
    }

    fn compile_program(&mut self, program: Program) -> String {
        // Header
        self.emit(".global main");
        self.emit(".align 4");
        self.emit("main:");

        // Prologue
        self.emit("    push {fp, lr}");     //save fp and return address
        self.emit("    mov fp, sp");                  //fp = sp
        self.emit("    sub sp, sp, #256");             //reserving space (TBD: actually count the variables
        
        // Compiling statements
        for statement in &program.statements {
            self.compile_statement(statement);
        }

        // Epilogue
        self.emit("    add sp, sp, #256");         // TBD: actual variable offsets!
        self.emit("    pop {fp, lr}");
        self.emit("    bx lr");
        // Reset fp
        // Put x0 in RA
        // Clean up stack
        self.output.clone()  
    }
}


fn main() {
    
    let args: Vec<String> = env::args().collect();
    let code_filename = &args[1];
    let assembly_filename = &args[2];
    let program_text = &fs::read_to_string(code_filename).unwrap();
    let tokens = lex(program_text);
    let mut parser = Parser {tokens, position: 0};
    let program = parser.parse_program();
    let mut compiler = Compiler::new();
    let assembly = compiler.compile_program(program);
    fs::write(assembly_filename, assembly).unwrap();
}


