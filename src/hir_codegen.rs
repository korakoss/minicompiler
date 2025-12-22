use std::collections::HashMap;
use crate::hir::*;


pub struct HIRCompiler {
    output: String,
}

impl HIRCompiler {

    pub fn new() -> Self {
        HIRCompiler{output: String::new()}
    }
    
    fn emit(&mut self, line: &str) {        
        self.output.push_str(line);
        self.output.push('\n');
    }

    pub fn compile_hir_program(&mut self, hir_program: HIRProgram) -> String {
        
        self.emit(".global main");
        self.emit(".extern printf");
        self.emit(".align 4");
        self.emit(".data");
        self.emit(r#"fmt: .asciz "%d\n""#);
        self.emit(".text");
        
        let HIRProgram{scopes, variables, functions, global_scope} = hir_program; 
        
        // TODO: compile functions, beforehands, emit the label

        self.emit("main:");

        // Prologue
        self.emit("    push {fp, lr}");     
        self.emit("    mov fp, sp");       
        self.emit("    sub sp, sp, #256"); 
        
        // TODO: compile global block or sth                

        // Epilogue
        self.emit("    add sp, sp, #256");         // TBD: actual variable offsets!
        self.emit("    pop {fp, lr}");
        self.emit("    bx lr");

        self.output.clone() 
    }

    // Return string or emit?
    fn compile_function(&mut self, func: HIRFunction) {
    }

    fn compile_statement(&mut self, statement: HIRStatement, scope: ScopeId) {
    }
    
    fn compile_block(&mut self, block_id: ScopeId) {
    }

    fn compile_expression(&mut self, expression: TypedExpressionKind) {
        match expression {
            TypedExpressionKind::IntLiteral(n) => {
                self.emit(&format!("    ldr r0, ={}", n));   // Load the value into the main register 
            },
            TypedExpressionKind::Variable(varid)

        }
    }
}
