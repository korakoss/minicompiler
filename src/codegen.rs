use std::collections::HashMap;
use crate::ast::*;


#[derive(Clone)]
struct Context {
    stack_offsets: HashMap<String, i32>,
    loop_start_label_stack: Vec<String>,
    loop_end_label_stack: Vec<String>,
    return_label: Option<String>,           // TODO: later not Option, because mainfunc
}


impl Context {
    fn new(return_label: Option<String>) -> Context {
        Context {
            stack_offsets: HashMap::new(),
            loop_start_label_stack: Vec::new(),
            loop_end_label_stack: Vec::new(),
            return_label: return_label
        }
    }
}


pub struct Compiler {
    output: String,
    label_counter: u32,  
}

impl Compiler {

    pub fn new() -> Self {
        Compiler { output: String::new(), label_counter: 0}
    }

    fn emit(&mut self, line: &str) {        // TODO: let this do the formatting
        self.output.push_str(line);
        self.output.push('\n');
    }

    fn assign_labelcount(&mut self) -> u32 {
        self.label_counter = self.label_counter + 1;
        self.label_counter
    }

    fn compile_expression(&mut self, context: &Context, expression: Expression) {  
       
        match expression {
            
            Expression::FuncCall { funcname, args } => {
                // TODO: validating function exists in the first place
                // NOTE: only for 1 arg, TODO: general solution
               
                // What if empty? TODO
                let num_args = args.len();
                for arg_expr in  args{
                    self.compile_expression(context, *arg_expr);
                    self.emit("    push {r0}");
                }
                for i in 0..num_args {
                    let register = format!("r{}", num_args-i-1);
                    self.emit(&format!("    pop {{{}}}", register));    
                }
                self.emit(&format!("    bl {}", funcname));
            }
            
            Expression::IntLiteral(n) => {
                self.emit(&format!("    ldr r0, ={}", n));   // Load the value into the main register 

            }

            Expression::Variable(varname) => {
                let offset = context.stack_offsets.get(&varname).expect(&format!("Undefined variable: {}", &varname));
                self.emit(&format!("    ldr r0, [fp, #-{}]", offset));    // 
            }

            Expression::BinOp{op, left, right} => {
                self.compile_expression(context, *left);
                self.emit("    push {r0}");// Store left's value on stack
                self.compile_expression(context, *right);
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

    fn compile_statement_block(&mut self, external_context: &mut Context, block: Vec<Statement>){
        let mut block_context = external_context.clone();
        for stmt in block {
            self.compile_statement(&mut block_context, stmt);
        }
    }


    fn compile_statement(&mut self, context: &mut Context, statement: Statement) {

        match statement {

            Statement::Let {var, value} => {
                self.compile_expression(context, value);     
                let new_offset = context.stack_offsets.values().max().unwrap_or(&0) + 8;
                context.stack_offsets.insert(var.name, new_offset);
                self.emit(&format!("    str r0, [fp, #-{}]", new_offset));
            }
 
            Statement::Assign { target, value } => {
                let Expression::Variable(varname) = target else {
                    panic!("Invalid target for assignment");
                };
                self.compile_expression(context, value);
                let &var_offset = context.stack_offsets.get(&varname).unwrap();
                self.emit(&format!("    str r0, [fp, #-{}]", var_offset));
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

            Statement::Return(return_expr) => {
                self.compile_expression(context, return_expr);
                match &context.return_label {
                    None => panic!("Return without active return label set"),
                    Some(label) => {
                        self.emit(&format!("    b {}", label));
                    }
                }
            }
            Statement::Print(print_expr) => {
                self.compile_expression(context, print_expr);
                self.emit("    mov r1, r0");
                self.emit("    ldr r0, =fmt");
                self.emit("    bl printf");
            }
        }
    }

    
    fn compile_function(&mut self, function: Function) {
        let Function{name, args, body, ret_type} = function;
        self.emit(&format!("{}:", name));
        self.emit("    push {fp, lr}");     
        self.emit("    mov fp, sp");     
        self.emit("    sub sp, sp, #256"); // TBD: do properly        
        
        let ret_label = format!("{}_epilogue", name);
        let mut func_context = Context::new(Some(ret_label.clone()));

        if args.len() > 0 {

            // TODO: implement stack-based
            if args.len() > 4 {
                panic!("Only up to 4 args supported at the moment");
            }
            
            for (i,arg) in args.iter().enumerate() {
                let current_offset = (i+1)*8; 
                self.emit(&format!("    str r{}, [fp, #-{}]", i,current_offset)); 
                func_context.stack_offsets.insert(arg.name.clone(), current_offset as i32);
            }
        }

        for stmt in body {
            self.compile_statement(&mut func_context, stmt);
        }
        
        self.emit(&format!("{}:", ret_label));
        self.emit("    add sp, sp, #256");         // TBD: actual variable offsets!
        self.emit("    pop {fp, lr}");
        self.emit("    bx lr");
    }
    
    pub fn compile_program(&mut self, program: RawAST) -> String {
        // Header
        self.emit(".global main");
        self.emit(".extern printf");
        self.emit(".align 4");
        self.emit(".data");
        self.emit(r#"fmt: .asciz "%d\n""#);
        self.emit(".text");
        
        let RawAST{functions, main_statements} = program;
        for func in functions {
            self.compile_function(func);
        }


        self.emit("main:");

        // Prologue
        self.emit("    push {fp, lr}");     //save fp and return address
        self.emit("    mov fp, sp");                  //fp = sp
        self.emit("    sub sp, sp, #256");             //reserving space (TBD: actually count the variables
        
                
        let mut global_context = Context::new(None);
        for stmt in main_statements {
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

