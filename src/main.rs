use std::fs;
use std::env;

mod ast;

mod build_ast;
use crate::build_ast::*;

mod codegen;
use crate::codegen::*;

mod hir;
use crate::hir::*;

mod ast_to_hir;
use ast_to_hir::*;

mod common;
use crate::common::*;

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


