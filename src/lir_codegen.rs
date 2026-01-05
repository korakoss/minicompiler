
use std::collections::HashMap;

use crate::{lir::*, shared::binops::BinaryOperator};
use crate::{hir::FuncId}; 


struct StackFrame {
    size: usize,
    offsets: HashMap<VRegId, usize>,
}

pub struct LIRCompiler {
    output: String,
}

impl LIRCompiler {
    
    pub fn compile(lir_program: LIRProgram) -> String {
        let mut comp = LIRCompiler {
            output: String::new(),
        };
        comp.compile_program(lir_program)
    }


    fn compile_program(&mut self, program: LIRProgram) -> String {

        self.emit(".global main");
        self.emit(".extern printf");
        self.emit(".align 4");
        self.emit(".data");
        self.emit(r#"fmt: .asciz "%d\n""#);
        self.emit(".text");
        
        for (f_id, func) in program.functions.into_iter() {
            self.compile_function(f_id, func);
        }
        
        self.emit("main:");
        self.emit("    push {lr}");
        self.emit(&format!("    bl func_{}", program.entry.0));
        self.emit("    pop {lr}");
        self.emit("    bx lr");

        self.output.clone()

       
    }


    fn compile_function(&mut self, func_id: FuncId,lir_func: LIRFunction) {
        let LIRFunction { blocks, entry, vregs, args } = lir_func;
        let frame = self.make_stack_frame(&vregs);
        
        self.emit(&format!("func_{}:", func_id.0));
        self.emit("    push {fp, lr}");     
        self.emit("    mov fp, sp");     
        self.emit(&format!("    sub sp, sp, #{}", frame.size)); 

        for (i,arg) in args.iter().enumerate() {
            let arg_offset = frame.offsets[&arg];
            self.emit(&format!("    str r{}, [fp, #-{}]", i+1, arg_offset));
}

        self.emit(&format!("    b block_{}", entry.0));

        for (id, block) in blocks.into_iter() {
            self.compile_block(id, block, &frame);
        }
        
        self.emit(&format!("    add sp, sp, #{}", frame.size));         
        self.emit("    pop {fp, lr}");
        self.emit("    bx lr");

    }

    fn make_stack_frame(&self, vregs: &HashMap<VRegId, VRegInfo>) -> StackFrame {
        // TODO: do alignment later
        let mut offsets: HashMap<VRegId, usize> = HashMap::new();
        let mut curr_offset = 0;
        for (id, reg_info) in vregs {
            offsets.insert(id.clone(), curr_offset);
            curr_offset = curr_offset + reg_info.size;
        }
        StackFrame {
            size: curr_offset,
            offsets,
        }
    }

    fn compile_block(&mut self, id: BlockId, block: LIRBlock, frame: &StackFrame) {
        self.emit(&format!("block_{}:", id.0));
        let LIRBlock {statements, terminator} = block;
        for stmt in statements {
            self.compile_stmt(stmt, frame);
        }
        self.compile_terminator(terminator, frame);
    }

    fn compile_stmt(&mut self, stmt: LIRStatement, frame: &StackFrame) {
        
        match stmt {
            LIRStatement::Load { dest, from } => {
                self.emit_place_load(from, frame);
                let vreg_offset = frame.offsets
                    .get(&dest)
                    .expect(&format!("Register {:?} not found", dest))
                    .clone();
                self.emit(&format!("    str r0, [fp, #-{}]", vreg_offset));
            }
LIRStatement::Store { dest, value } => {
                self.emit_operand_load(value, frame);
                self.emit_place_store(dest, frame);
            }
            LIRStatement::BinOp { dest, op, left, right } => {
self.emit_operand_load(left, frame);
                self.emit("    mov r1, r0");
                self.emit_operand_load(right, frame);
                self.compile_binop(op);
                self.emit_place_store(dest, frame);
            }
            LIRStatement::Call { dest, func, args } => {
                // TODO: change this for stack usage
                // This is a quick solution to check LIR at all

                if args.len() > 3 {
                    panic!("Only up to 3 args supported at the moment");
                }

                for (i, arg) in args.into_iter().enumerate() {
                    let arg_offset = frame.offsets[&arg].clone();
                    self.emit(&format!("     ldr r{}, [fp, #-{}]", i+1, arg_offset));
                }
                self.emit(&format!("    bl func_{}", func.0));
                self.emit_place_store(dest, frame);
            }
            LIRStatement::Print(operand) => {
                self.emit_operand_load(operand, frame);
                self.emit("    mov r1, r0");
                self.emit("    ldr r0, =fmt");
                self.emit("    bl printf");
            }
        }

    }

    fn compile_binop(&mut self, op: BinaryOperator) {
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

    fn compile_terminator(&mut self, term: LIRTerminator, frame: &StackFrame) {
        match term {
            LIRTerminator::Goto{dest} => {
                self.emit(&format!("    b block_{}", dest.0));
            }
            LIRTerminator::Branch { condition, then_block, else_block } => {
                self.emit_operand_load(condition, frame);
                self.emit("    cmp r0, #0");
                self.emit(&format!("    beq block_{}", then_block.0));
                self.emit(&format!("    b block_{}", else_block.0));
            }
            LIRTerminator::Return(operand_opt) => {
                if let Some(operand) = operand_opt {
                    self.emit_operand_load(operand, frame);
                    self.emit("    bx lr");
                }
            }
        }
    }

    fn emit_operand_load(&mut self, operand: Operand, frame: &StackFrame) {
        match operand {
            Operand::Register(vreg) => {
                let vreg_offset = frame.offsets[&vreg].clone();
                self.emit(&format!("    ldr r0, [fp, #-{}]", vreg_offset));
            }
            Operand::IntLiteral(num) => {
                self.emit(&format!("     ldr r0, ={}", num));
            }
            Operand::BoolTrue => {
                self.emit(&"    ldr r0, =1");   
            }
            Operand::BoolFalse => {
                self.emit(&"    ldr r0, =0");   
            }
        }
    }

    fn emit_place_store(&mut self, place: LIRPlace, frame: &StackFrame) {
        match place {
            LIRPlace::VReg(vreg) => {
                let vreg_offset = frame.offsets
                    .get(&vreg)
                    .expect(&format!("Register {:?} not found", vreg))
                    .clone();
                self.emit(&format!("    str r0, [fp, #-{}]", vreg_offset));
            }
            LIRPlace::Deref { base, offset } => {
                let base_reg_offset = frame.offsets.clone()[&base];
                self.emit(&format!("    str r0, [fp, #-{}]", base_reg_offset-offset));    
            }
        }
    }
    
    fn emit_place_load(&mut self, place: LIRPlace, frame: &StackFrame) {
        match place {
            LIRPlace::VReg(vreg) => {
                let vreg_offset = frame.offsets[&vreg].clone();
                self.emit(&format!("    ldr r0, [fp, #-{}]", vreg_offset));
            }
            LIRPlace::Deref { base, offset } => {
                let base_reg_offset = frame.offsets[&base].clone();
                self.emit(&format!("    ldr r0, [fp, #-{}]", base_reg_offset-offset));    
            }
        }
    }

    fn emit(&mut self, line: &str) {        
        self.output.push_str(line);
        self.output.push('\n');
    }


}


