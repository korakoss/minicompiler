use std::collections::HashMap;
use std::fs;
use std::env;
use std::hash::Hash;


#[derive(Debug)]
enum BinaryOperator {
    Add, 
    Sub, 
    Mul, 
    Equals,
    Less,       // left < right 
    Modulo
    //Greater, 
    //Div (later, when floats ig?),
    //NotEqual
}

#[derive(Debug)]
enum Expression {
    IntLiteral(i32),

    Variable(String),

    BinOp {
       op: BinaryOperator,
       left: Box<Expression>,
       right: Box<Expression>,
    },

    FuncCall {
        funcname: String,
        args: Vec<Box<Expression>>,
    }
    
    // UnaryOp (eg. negation)
}

#[derive(Debug)]
enum Statement {
    
    // NOTE: do we need to box exprs here???


    Assign {
        varname: String,
        value: Expression
    },

    If {
        condition: Expression,
        if_body: Vec<Statement>,
        else_body: Option<Vec<Statement>>,
    },
    
    While {
        condition: Expression,
        body: Vec<Statement>,
    },

    Break,
    Continue,

    Return(Expression),
    //Print(Box<Expression>),
}

#[derive(Debug)]
struct Function {
    name: String,
    args: Vec<String>,
    body: Vec<Statement>,
}

#[derive(Debug)]
struct Program {
    functions: Vec<Function>,
    main_statements: Vec<Statement>
}


#[derive(Debug, Clone, PartialEq)]
enum Token {
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
    
    // Values 
    IntLiteral(i32),
    Identifier(String),
    
    // Keywords
    //Print,
    If,
    Else,
    While,
    Break,
    Continue,
    Function,
    Return,
    
    // Special
    EOF,
}


fn lex(program: &str) -> Vec<Token> {
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

        else {
            // Processing single character stuff
            let token = match c {
                '+' => Token::Plus,
                '-' => Token::Minus,
                '*' => Token::Multiply,
                ';' => Token::Semicolon,
                '(' => Token::LeftParen,
                ')' => Token::RightParen,
                '{' => Token::LeftBrace,
                '}' => Token::RightBrace,
                '<' => Token::Less,
                '%' => Token::Modulo,
                ',' => Token::Comma,
                _ => {panic!("Unexpected character: {}",c)},
            };
            chars.next();
            tokens.push(token);
        }
        
    } 
    tokens.push(Token::EOF);
    tokens
}


struct Parser {
    tokens: Vec<Token>,
    position: usize,
    loop_nest_counter: usize,
    within_function_body:bool,
    defined_funcs: Vec<Function>,
    glob_vars: Vec<String>,
    loc_vars: Vec<String>,
}


impl Parser {
    
    fn new(tokens: Vec<Token>) -> Self {
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
        self.expect_token(Token::RightParen);
        args
    }

    fn parse_primitive(&mut self) -> Expression {
        
         match self.consume() {
            Token::IntLiteral(int) => {Expression::IntLiteral(int)},
            Token::Identifier(name) => {
                if self.peek() == &Token::LeftParen {        // Function call 
                    self.consume();
                    let args = self.parse_funccall_args(); 
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
        }
        self.expect_token(Token::RightParen);
        self.expect_token(Token::LeftBrace);
        self.within_function_body = true;
        let body = self.parse_block();
        self.expect_token(Token::RightBrace);
        self.within_function_body = false;
        Function {name: funcname, args: args, body: body}
    }

    fn parse_program(mut self) -> Program {
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


#[derive(Clone)]
struct Context {
    stack_offsets: HashMap<String, i32>,
    loop_start_label_stack: Vec<String>,
    loop_end_label_stack: Vec<String>,
}


impl Context {
    fn new() -> Context {
        Context {
            stack_offsets: HashMap::new(),
            loop_start_label_stack: Vec::new(),
            loop_end_label_stack: Vec::new(),
        }
    }
}


struct Compiler {
    output: String,
    label_counter: u32,  
}

impl Compiler {

    fn new() -> Self {
        Compiler { output: String::new(), label_counter: 0}
    }

    fn emit(&mut self, line: &str) {
        self.output.push_str(line);
        self.output.push('\n');
    }

    fn assign_labelcount(&mut self) -> u32 {
        self.label_counter = self.label_counter + 1;
        self.label_counter
    }

    fn compile_expression(&mut self, context: &Context, expression: &Expression) {  
       
        match expression {
            
            Expression::FuncCall { funcname, args } => {unimplemented!("FuncCall unimplemented")}
            
            Expression::IntLiteral(n) => {
                self.emit(&format!("    ldr r0, ={}", n));   // Load the value into the main register 

            }

            Expression::Variable(varname) => {
                let offset = context.stack_offsets.get(varname).expect(&format!("Undefined variable: {}", &varname));
                self.emit(&format!("    ldr r0, [fp, #-{}]", offset));    // 
            }

            Expression::BinOp{op, left, right} => {
                self.compile_expression(context, left);
                self.emit("    push {r0}");// Store left's value on stack
                self.compile_expression(context, right);
                self.emit("    pop {r1}");

                match op {
                    BinaryOperator::Add => {
                        self.emit("    add r0, r1, r0");
                    }
                    BinaryOperator::Sub => {
                        self.emit("    sub r0, r1, r0");   // x1-x0 because x1: left x0:right
                    }
                    BinaryOperator::Mul => {
                        self.emit("    mul r0, r1, r0");   
                    }
                    BinaryOperator::Equals => {
                        self.emit("    cmp r1, r0");
                        self.emit("    mov r0, #0");
                        self.emit("    moveq r0, #1");
                    }
                    BinaryOperator::Less=> {
                        self.emit("    cmp r1, r0");
                        self.emit("    mov r0, #0");
                        self.emit("    movlt r0, #1");
                    }
                    BinaryOperator::Modulo => {
                        self.emit("    sdiv r2, r1, r0"); // r2 <- int(left/right)  [-upwards for negative]  
                        self.emit("    mul r2, r0, r2");   // r2 <- right * int(left/right)
                        self.emit("    sub r0, r1, r2");
                    }
                }
            }
        }

    }

    fn compile_statement_block(&mut self, external_context: &mut Context, block: &Vec<Statement>){
        let mut block_context = external_context.clone();
        for stmt in block {
            self.compile_statement(&mut block_context, stmt);
        }
    }


    fn compile_statement(&mut self, context: &mut Context, statement: &Statement) {

        match statement {
            
            Statement::Assign { varname, value } => {

                self.compile_expression(context, value);
                if let Some(&var_offset) = context.stack_offsets.get(varname) {
                    // The variable already exists -> we use its address
                    self.emit(&format!("    str r0, [fp, #-{}]", var_offset));
                } else {
                    // New variable: allocate it space on the stack
                    let new_offset = context.stack_offsets.values().max().unwrap_or(&0) + 8;
                    context.stack_offsets.insert(varname.clone(), new_offset);
                    self.emit(&format!("    str r0, [fp, #-{}]", new_offset));
                }
            }
            Statement::If {condition, if_body, else_body} => {
                let counter_str = format!("{}", self.assign_labelcount());
                let branching_end_label = format!("branching_end_{}", counter_str);

                self.compile_expression(context, condition);
                self.emit("    cmp r0, #0");
                
                match else_body {
                    
                    Some(else_statements) => {
                        let else_start_label = format!("else_start_{}", counter_str);
                        self.emit(&format!("    beq {}", else_start_label));
                        self.compile_statement_block(context, if_body); 
                        self.emit(&format!("    b {}", branching_end_label));
                        self.emit(&format!("{}:", else_start_label));
                        self.compile_statement_block(context, else_statements); 
                    }
                    None => {
                        self.emit(&format!("    beq {}", branching_end_label));
                        self.compile_statement_block(context, if_body);
                    } 

                }
                self.emit(&format!("{}:",branching_end_label));
            }
            
            Statement::While {condition, body} => {
                let counter_str = format!("{}", self.assign_labelcount());
                let start_label = format!("while_start_{}", counter_str);
                let end_label = format!("while_end_{}", counter_str);

                context.loop_start_label_stack.push(start_label.clone());
                context.loop_end_label_stack.push(end_label.clone());

                self.emit(&format!("{}:", start_label));
                
                self.compile_expression(context, condition);
                self.emit("    cmp r0, #0");
                self.emit(&format!("    beq {}", end_label));
                
                self.compile_statement_block(context, body);
                
                self.emit(&format!("    b {}", start_label));

                self.emit(&format!("{}:", end_label));
                context.loop_start_label_stack.pop();
                context.loop_end_label_stack.pop();
            }
            
            Statement::Break => {
                
                match context.loop_end_label_stack.last() {
                    None => {unreachable!("Break outside loop at compilation")},
                    Some(end_label) => {
                        self.emit(&format!("    b {}", end_label));
                    }
                } 
            }
            
            Statement::Continue => {
                
                match context.loop_start_label_stack.last() {
                    None => {unreachable!("Continue outside loop at compilation")},
                    Some(start_label) => {
                        self.emit(&format!("    b {}", start_label));
                    }
                } 
            }

            Statement::Return(value) => {unimplemented!("Return unimplemented")}
        }
    }

    
    fn compile_function(&mut self, function: Function) {
        self.emit(&format!("{}:", function.name));
        self.emit("    push {fp, lr}");     
        self.emit("    mov fp, sp");     
        self.emit("    sub sp, sp, #256"); // TBD: do properly        
        
        self.emit("    str r0, [fp, #-4]"); // save the argument (NOTE: 1 argument assumed for now)
         
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
        let mut global_context = Context::new();
        for stmt in &program.main_statements {
            self.compile_statement(&mut global_context, stmt);
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

    for tok in &tokens {
        println!("{:?}", tok);
    }

    let parser = Parser::new(tokens);
    let program = parser.parse_program();
    println!("{:?}", program);
    let mut compiler = Compiler::new();
    let assembly = compiler.compile_program(program);
    fs::write(assembly_filename, assembly).unwrap();
}


