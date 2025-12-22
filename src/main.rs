use std::fs;
use std::env;

mod ast;

mod build_ast;
use crate::build_ast::*;

//mod codegen;
//use crate::codegen::*;

mod hir;

mod ast_to_hir;
use ast_to_hir::*;

mod common;

mod hir_codegen;
use hir_codegen::*;


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
    let ast = parser.parse_program();
    println!("{:?}", ast);
   
    let mut lowerer = HIRBuilder::new();
    let hir = lowerer.lower_ast(ast.clone());
    println!("{:?}", hir);

    //let mut compiler = Compiler::new();
    //let assembly = compiler.compile_program(ast);

    let mut hircomp = HIRCompiler::new(hir);
    let assembly = hircomp.compile();
    fs::write(assembly_filename, assembly).unwrap();
}


