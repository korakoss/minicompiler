use std::collections::HashMap;

use crate::shared::binops::*;
use crate::hir::*;

pub struct HIRCompiler {
    output: String,
    hir_program: HIRProgram,
    jumplabel_counter: usize,
    looplabel_stack: Vec<String>,
    layout_stack: Vec<HashMap<VarId, usize>>,
    active_ret_label: Option<String>,
}

impl HIRCompiler {

    pub fn new(hir_program: HIRProgram) -> Self {
        HIRCompiler{
            output: String::new(), 
            hir_program: hir_program,
            jumplabel_counter: 0,
            looplabel_stack: Vec::new(),
            layout_stack: Vec::new(),
            active_ret_label: None
        }
    }

    fn get_var_offset(&self, var_id: VarId) -> usize {
        let offsets: HashMap<VarId, usize> = self.layout_stack
            .iter()
            .flat_map(|map| map.iter())
            .map(|(&k, &v)| (k, v))
            .collect();
        offsets.get(&var_id).expect(&format!("Didn't find variable of ID {:?} in scope", var_id)).clone()
    }
    
    pub fn compile(&mut self) -> String {
        
        self.emit(".global main");
        self.emit(".extern printf");
        self.emit(".align 4");
        self.emit(".data");
        self.emit(r#"fmt: .asciz "%d\n""#);
        self.emit(".text");
        
        for (f_id, func) in self.hir_program.functions.clone() {
            self.compile_function(f_id, func);
        }
        
        self.emit("main:");
        self.emit("    push {lr}");
        let flabel = format!("func_{}", self.hir_program.entry.0);
        self.emit(&format!("    bl {}", flabel));
        self.emit("    pop {lr}");
        self.emit("    bx lr");

        self.output.clone()
    }
    
    fn compile_function(&mut self, id: FuncId, function: HIRFunction) {

        let flabel = format!("func_{}", id.0);
        let retlabel = format!("ret_{}", id.0);
        self.active_ret_label = Some(retlabel.clone());
    
        let HIRFunction{name, args, body_variables, body, ret_type} = function;

        let all_vars = [args.clone(), body_variables].concat();
        let (top_var_offsets, reserved_space) = self.determine_scope_var_offsets(all_vars);

        self.layout_stack.push(top_var_offsets.clone());
        if args.len() > 4 {
                panic!("Only up to 4 args supported at the moment");
        }

        self.emit_prologue(reserved_space, flabel);

        for (i,arg) in args.iter().enumerate() {
            let arg_offset = top_var_offsets.get(&arg).expect(&format!("Arg variable {:?} not found when compiling function {}", arg, name));
            self.emit(&format!("    str r{}, [fp, #-{}]", i, arg_offset));
        }

        for stmt in body {
            self.compile_statement(stmt);
        }
        
        self.emit(&format!("{}:", retlabel));
        self.emit_epilogue(reserved_space);
        self.active_ret_label = None;
        self.layout_stack.pop();
    }
    
    fn compile_block(&mut self, block: Vec<HIRStatement>) {
        for stmt in block {
            self.compile_statement(stmt);
        }
    }

    fn determine_scope_var_offsets(&self, var_ids: Vec<VarId>) -> (HashMap<VarId, usize>, usize) {
        let mut offsets: HashMap<VarId, usize> = HashMap::new();

        let mut curr_offset: usize = 0;
        for var in var_ids {
            let vartype = self.hir_program.variables[&var].typ.clone();
            let varsize = self.hir_program.layouts.get_layout(vartype).size();
            offsets.insert(var, curr_offset);
            curr_offset = curr_offset + varsize;
        }
        (offsets, curr_offset)
    }

            
    fn compile_statement(&mut self, statement: HIRStatement) {
        match statement {
            HIRStatement::Let{var: target, value} | HIRStatement::Assign { target, value } => {
                self.compile_expression(value);
                let Place::Variable(var_id) = target; 
                let var_offset = self.get_var_offset(var_id);
                self.emit(&format!("    str r0, [fp, #-{}]", var_offset));
            }
            HIRStatement::While { condition, body } => {
                let jlabel = self.get_jumplabel();
                let start_label = format!("while_start_{}", jlabel);
                let end_label = format!("while_end_{}", jlabel);
                
                self.emit(&format!("{}:", start_label.clone()));
                self.compile_expression(condition);
                self.emit("    cmp r0, #0");
                self.emit(&format!("    beq {}", end_label.clone()));
                
                self.looplabel_stack.push(jlabel);
                
                self.compile_block(body);
                self.emit(&format!("    b {}", start_label));
                self.emit(&format!("{}:", end_label));

                self.looplabel_stack.pop();
            }
            HIRStatement::If {condition, if_body, else_body} => {
                let jlabel = self.get_jumplabel();
                let end_label = format!("branching_end_{}", jlabel);
                
                self.compile_expression(condition);
                self.emit("    cmp r0, #0");

                match else_body {
                    None => {
                        self.emit(&format!("    beq {}", end_label));
                        self.compile_block(if_body);
                    }
                    Some(else_block) => {
                        let else_start_label = format!("else_start_{}", jlabel);
                        self.emit(&format!("    beq {}", else_start_label));
                        self.compile_block(if_body); 
                        self.emit(&format!("    b {}", end_label));
                        self.emit(&format!("{}:", else_start_label));
                        self.compile_block(else_block); 
                    }
                }
                self.emit(&format!("{}:",end_label));
            }
            // TODO: if
            HIRStatement::Break => {
                let curr_loop_end = format!("while_end_{}", self.looplabel_stack.last().unwrap());
                self.emit(&format!("    b {}", curr_loop_end));
            }
            HIRStatement::Continue=> {
                let curr_loop_start = format!("while_start_{}", self.looplabel_stack.last().unwrap());
                self.emit(&format!("    b {}", curr_loop_start));
            }
            HIRStatement::Return(expr) => {
                self.compile_expression(expr);
                self.emit(&format!("    b {}", self.active_ret_label.clone().unwrap()));
            }
            HIRStatement::Print(expr) => {
                self.compile_expression(expr );
                self.emit("    mov r1, r0");
                self.emit("    ldr r0, =fmt");
                self.emit("    bl printf");
            }
        }
    }

    
    fn compile_expression(&mut self, expression: HIRExpression) {
        match expression {
            HIRExpression::BinOp {op, left, right} => {
                self.compile_binop(op, *left, *right); 
            }
            HIRExpression::Variable(varid) => {
                self.emit(&format!("    ldr r0, [fp, #-{}]", self.get_var_offset(varid))); 
            }
            HIRExpression::IntLiteral(n) => {
                self.emit(&format!("    ldr r0, ={}", n));   
            }
            HIRExpression::FuncCall{id, args} => {
                for arg in args.clone() {
                    self.compile_expression(arg); 
                    self.emit("    push {r0}");
                }
                for i in 0..args.len() {
                    let register = format!("r{}", args.len()-i-1);
                    self.emit(&format!("    pop {{{}}}", register));    
                }
                
                // TODO: could do this more professionally
                let flabel = format!("func_{}", id.0);
                self.emit(&format!("    bl {}", flabel));
            }
            HIRExpression::BoolTrue => {
                self.emit(&"    ldr r0, =1");   
            }
            HIRExpression::BoolFalse => {
                self.emit(&"    ldr r0, =0");   
            }
            HIRExpression::StructLiteral{typ, fields} => {
                unimplemented!();
            }
            HIRExpression::FieldAccess { expr, field } => {
                unimplemented!();
            }
        }
    }

    fn compile_binop(&mut self, op: BinaryOperator, left: HIRExpression, right:HIRExpression) {
        self.compile_expression(left);
        self.emit("    push {r0}");
        self.compile_expression(right);
        self.emit("    pop {r1}");

        match op {
            BinaryOperator::Add => {
                self.emit("    add r0, r1, r0");
            }
            BinaryOperator::Sub => {
                self.emit("    sub r0, r1, r0");
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
                self.emit("    sdiv r2, r1, r0"); 
                self.emit("    mul r2, r0, r2"); 
                self.emit("    sub r0, r1, r2");
            }
        }
    }
    
    fn emit(&mut self, line: &str) {        
        self.output.push_str(line);
        self.output.push('\n');
    }
    
    fn emit_prologue(&mut self, reserved_stack: usize, ret_label: String) {
        self.emit(&format!("{}:", ret_label));
        self.emit("    push {fp, lr}");     
        self.emit("    mov fp, sp");     
        self.emit(&format!("    sub sp, sp, #{}", reserved_stack)); 
    }

    fn emit_epilogue(&mut self, reserved_stack: usize) {
        self.emit(&format!("    add sp, sp, #{}", reserved_stack));         
        self.emit("    pop {fp, lr}");
        self.emit("    bx lr");
    }
    
    fn get_jumplabel(&mut self) -> String {
        let label = format!("{}", self.jumplabel_counter);
        self.jumplabel_counter = self.jumplabel_counter + 1;
        label
    }
} 




