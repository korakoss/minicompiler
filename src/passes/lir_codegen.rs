use crate::stages::lir::*;
use crate::shared::definitions::{BinaryOperator, BlockId, FuncId, Id};


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
        self.emit_mult(&vec![
            ".global main",
            ".extern printf",
            ".align 8",
            ".data",
            r#"fmt: .asciz "%d\n""#,
            ".text",
        ]);
                
        for (f_id, func) in program.functions.into_iter() {
            self.compile_function(f_id, func);
        }

        self.emit_mult(&vec![
            "main:",
            "    push {fp, lr}",
            "    mov fp, sp",  
            "    sub sp, sp, #16",
            "    sub r12, fp, #8",
            &format!("    bl func_{}", program.entry.raw()),
            "    ldr r0, [r12]",
            "    add sp, sp, #16",
            "    pop {fp, lr}",
            "    bx lr",
        ]);
        self.output.clone()

       
    }


    fn compile_function(&mut self, func_id: FuncId,lir_func: LIRFunction) {

        let LIRFunction { blocks, entry, chunks, args } = lir_func;

        let frame_size = chunks.size();
        
        self.emit(&format!("func_{}:", func_id.raw()));
        self.emit("    push {fp, lr}");     
        self.emit("    mov fp, sp");     
        self.emit(&format!("    sub sp, sp, #{}", frame_size)); 

        for (i,arg) in args.iter().enumerate() {
            let arg_offset = chunks.get_offset(arg).unwrap();
            self.emit(&format!("    str r{}, [fp, #-{}]", i+1, arg_offset));
}

        self.emit(&format!("    b block_{}", entry.raw()));

        for (id, block) in blocks.into_iter() {
            self.compile_block(id, block, &chunks, func_id);
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
                // TODO: change this for stack usage
                // This is a quick solution to check LIR at all

                if args.len() > 3 {
                    panic!("Only up to 3 args supported at the moment");
                }

                for (i, arg) in args.into_iter().enumerate() {
                    self.emit_operand_load(LIRValue{ size: arg.size, value: LIRValueKind::Place(arg)}, frame);
                    self.emit(&format!("     mov r{}, r0", i+1));
                }
                
                self.emit("    push {r12}"); 
                match dest.place {
                    LIRPlaceKind::Local { base, offset } => {
                        let base_offset = frame.get_offset(&base)
                            .unwrap_or_else(|| panic!("Cell ID {:?} not found in frame", base));
                        let target_offset = base_offset + offset;
                        self.emit(&format!("    sub r12, fp, #{}", target_offset));
                        self.emit(&format!("    bl func_{}", func.raw()));
                    }
                    LIRPlaceKind::Deref { pointer, offset } => {
                        let pointer_offset = frame.get_offset(&pointer)
                            .unwrap_or_else(|| panic!("Cell ID {:?} not found in frame", pointer));
                        self.emit(&format!("    ldr r0, [fp, #-{}]", pointer_offset));  
                        self.emit(&format!("    ldr r0, [r0, #-{}]", offset));  
                        self.emit(&format!("    bl func_{}", func.raw()));                       
                    }
                }
                self.emit("    pop {r12}"); 

            }
            LIRStatement::Print(operand) => {
                self.emit_operand_load(operand, frame);
                self.emit("    mov r1, r0");
                self.emit("    ldr r0, =fmt");
                self.emit("    push {r12}"); 
                self.emit("    bl printf");
                self.emit("    pop {r12}"); 
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

    fn emit_operand_load(&mut self, operand: LIRValue, frame: &StackFrame) {
        match operand.value {
            LIRValueKind::Place(place) => {
                match place.place {
                    LIRPlaceKind::Local { base, offset } => {
                        let base_offset = frame.get_offset(&base)
                           .unwrap_or_else(|| panic!("Cell ID {:?} not found in frame", base));
                        let place_offset = base_offset + offset;
                        self.emit(&format!("    ldr r0, [fp, #-{}]", place_offset));
                    }
                    LIRPlaceKind::Deref { pointer, offset } => {
                        let pointer_offset = frame.get_offset(&pointer)
                            .unwrap_or_else(|| panic!("Cell ID {:?} not found in frame", pointer));
                        self.emit(&format!("    ldr r0, [fp, #-{}]", pointer_offset));  
                        self.emit(&format!("    ldr r0, [r0, #-{}]", offset));  
                    }
                }
            }
            LIRValueKind::IntLiteral(num) => {
                self.emit(&format!("     ldr r0, ={}", num));
            }
            LIRValueKind::BoolTrue => {
                self.emit("    ldr r0, =1");   
            }
            LIRValueKind::BoolFalse => {
                self.emit("    ldr r0, =0");   
            }
            LIRValueKind::Reference(refd) => {
                match refd.place {
                    LIRPlaceKind::Local { base, offset } => {
                        let base_offset = frame.get_offset(&base)
                            .unwrap_or_else(|| panic!("Unsuccessful offset lookup for cell ID {:?}", base));
                        let place_offset = base_offset + offset;
                        self.emit(&format!("    sub r0, fp, #{}", place_offset));  
                    }
                    LIRPlaceKind::Deref {..} => {
                        unimplemented!();       // Shouldn't really happen, maybe refactor stuff
                    }
                }
            }
        }
    }

    fn emit_place_store(&mut self, place: LIRPlace, frame: &StackFrame) {
        match place.place {
            LIRPlaceKind::Local { base, offset } => {
                let base_offset = frame.get_offset(&base)
                    .unwrap_or_else(|| panic!("Unsuccessful offset lookup for cell ID {:?}", base));
                let place_offset = base_offset + offset;
                self.emit(&format!("    str r0, [fp, #-{}]", place_offset));
            }
            LIRPlaceKind::Deref { pointer, offset } => {
                // TODO: this fails for >8B values probably
                let pointer_st_offs = frame.get_offset(&pointer)
                    .unwrap_or_else(|| panic!("Unsuccessful offset lookup for cell ID {:?}", pointer));
                self.emit(&format!("    ldr r1, [fp, #-{}]", pointer_st_offs));  
                self.emit(&format!("    str r0, [r1, #-{}]", offset));  
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


