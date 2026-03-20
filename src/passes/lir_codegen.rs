use std::collections::HashMap;

use crate::stages::lir::*;
use crate::shared::definitions::{BinaryOperator, BlockId, FuncId, Id, VregId};


pub fn compile_lir(lir_program: LIRProgram) -> String {
    let mut comp = LIRCompiler::new();
    comp.emit_mult(&vec![
        ".global main",
        ".extern printf",
        ".align 8",
        ".data",
        r#"fmt: .asciz "%d\n""#,
        ".text",
    ]);

    for (f_id, func) in lir_program.functions.into_iter() {
        comp.compile_function(f_id, func);
    }

    comp.emit_mult(&vec![
        "main:",
        "    push {fp, lr}",
        "    mov fp, sp",  
        "    sub sp, sp, #16",
        "    sub r12, fp, #8",
        &format!("    bl func_{}", lir_program.entry.raw()),
        "    ldr r0, [r12]",
        "    add sp, sp, #16",
        "    pop {fp, lr}",
        "    bx lr",
    ]);
    comp.output
}


pub struct LIRCompiler {
    output: String,
    vreg_offsets: HashMap<VregId, usize>,
}

impl LIRCompiler {

    fn new() -> Self {
        Self { output: String::new(), vreg_offsets: HashMap::new() }
    }

    fn compile_function(&mut self, func_id: FuncId,lir_func: LIRFunction) {

        self.vreg_offsets = HashMap::new();

        let LIRFunction { blocks, entry, frame, args } = lir_func;

        let frame_size = frame.size();
        
        self.emit(&format!("func_{}:", func_id.raw()));
        self.emit("    push {fp, lr}");     
        self.emit("    mov fp, sp");     
        self.emit(&format!("    sub sp, sp, #{}", frame_size)); 

        // NEW
        for (i, arg) in args.iter().rev().enumerate() {
            let offset = (i+1) * 8;
            self.vreg_offsets.insert(*arg, offset);
        }

        self.emit(&format!("    b block_{}", entry.raw()));

        for (id, block) in blocks.into_iter() {
            self.compile_block(id, block, &frame, func_id);
        }

        self.emit(&format!("ret_{}:", func_id.raw()));        
        self.emit("    str r0, [r12]");
        self.emit(&format!("    add sp, sp, #{}", frame_size));         
        self.emit("    pop {fp, lr}");
        self.emit("    bx lr");

    }

    
    fn compile_block(&mut self, id: BlockId, block: LIRBlock, frame: &StackFrame, func: FuncId) {
        self.emit(&format!("block_{}:", id.raw()));
        let LIRBlock {statements, terminator} = block;
        for stmt in statements {
            self.compile_stmt(stmt, frame);
        }
        self.compile_terminator(terminator, frame, func);
    }

    fn compile_terminator(&mut self, term: LIRTerminator, frame: &StackFrame, func_id: FuncId) {
        match term {
            LIRTerminator::Goto{dest} => {
                self.emit(&format!("    b block_{}", dest.raw()));
            }
            LIRTerminator::Branch { condition, then_block, else_block } => {
                self.emit_operand_load(condition, frame);
                self.emit("    cmp r0, #1");
                self.emit(&format!("    beq block_{}", then_block.raw()));
                self.emit(&format!("    b block_{}", else_block.raw()));
            }
            LIRTerminator::Return(operand_opt) => {
                if let Some(operand) = operand_opt {
                    self.emit_operand_load(operand, frame);
                }
                self.emit(&format!("    b ret_{}", func_id.raw()));
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
                self.emit_mult(&vec![
                    "    cmp r1, r0",
                    "    mov r0, #0",
                    "    moveq r0, #1",
                ]);
            }
            BinaryOperator::Less=> {
                self.emit_mult(&vec![
                    "    cmp r1, r0",
                    "    mov r0, #0",
                    "    movlt r0, #1",
                ]);
            }
            BinaryOperator::Modulo => {
                self.emit_mult(&vec![
                    "    sdiv r2, r1, r0",
                    "    mul r2, r0, r2",
                    "    sub r0, r1, r2",
                ]);
            }
        }
    }

    fn compile_stmt(&mut self, stmt: LIRStatement, frame: &StackFrame) {
        
        match stmt {
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

                for arg in args.into_iter().rev() {
                    self.emit_operand_load(LIRValue::Reference(arg), frame);
                    self.emit("     push {r0}");
                }
                
                self.emit_operand_load(LIRValue::Reference(dest), frame);
                self.emit("     push {r0}");
                self.emit(&format!("    bl func_{}", func.raw()));                       
            }
            LIRStatement::Print(operand) => {
                self.emit_operand_load(operand, frame);
                self.emit_mult(&[
                    "    mov r1, r0",
                    "    ldr r0, =fmt",
                    "    push {r12}",
                    "    bl printf",
                    "    pop {r12}",
                ]);
            }
        }
    }

    fn emit_operand_load(&mut self, operand: LIRValue, frame: &StackFrame) {
        match operand {
            LIRValue::Place(place) => {
                let LIRPlace { base, offset } = place;
                self.emit_address_calc(base, frame);
                self.emit(&format!("    ldr r0, [r1, #-{}]]", offset));
            }
            LIRValue::IntLiteral(num) => {
                self.emit(&format!("     ldr r0, ={}", num));
            }
            LIRValue::BoolTrue => {
                self.emit("    ldr r0, =1");   
            }
            LIRValue::BoolFalse => {
                self.emit("    ldr r0, =0");   
            }
            LIRValue::Reference(ref_place) => {
                let LIRPlace { base, offset } = ref_place;
                self.emit_address_calc(base, frame);
                self.emit(&format!("    sub r0, r1, #{}", offset));  
            }
        }
    }

    fn emit_place_store(&mut self, place: LIRPlace, frame: &StackFrame) {
        let LIRPlace { base, offset } = place;
        self.emit_address_calc(base, frame);
        self.emit(&format!("    str r0, [r1, #-{}]]", offset));
    }

    fn emit_address_calc(&mut self, address: Address, frame: &StackFrame) {
        match address {
            Address::Chunk(chunk) => {
                self.emit_memory_chunk_calc(chunk, frame);
            }
            Address::Dereference(ref_chunk) => {
                self.emit_memory_chunk_calc(ref_chunk, frame);
                self.emit("    ldr r1, [r1]");
            }
        }
    }

    fn emit_memory_chunk_calc(&mut self, chunk: MemoryChunk, frame: &StackFrame) {
        match chunk {
            MemoryChunk::Local(cell_id) => {
                let cell_offset = frame.get_offset(&cell_id)
                  .unwrap_or_else(|| panic!("Unsuccessful offset lookup for cell ID {:?}", cell_id));
                self.emit(&format!("    sub r1, fp, #{}", cell_offset));
            },
            MemoryChunk::VReg(vreg_id) => {
                let offs = self.vreg_offsets[&vreg_id];
                self.emit(&format!("    add r1, fp, #{}", offs));
            }
        }
    }
    
    fn emit(&mut self, line: &str) {        
        self.output.push_str(line);
        self.output.push('\n');
    }

    fn emit_mult(&mut self, lines: &[&str]) {
        for line in lines {
            self.emit(line);
        }
    }
}


