use std::fs;
use std::env;

mod ast;
mod common;
mod hir;

mod lex;
use crate::lex::*;

mod parse;
use crate::parse::*;

mod hir_builder;
use hir_builder::*;

mod hir_codegen;
use hir_codegen::*;


fn main() {
    
    let args: Vec<String> = env::args().collect();
    let code_filename = &args[1];
    let assembly_filename = &args[2];
    let tokens_filepath = &args[3];
    let ast_filepath = &args[4];
    let hir_filepath = &args[5];

    let program_text = &fs::read_to_string(code_filename).unwrap();
    let tokens = lex(program_text);
    
    fs::write(tokens_filepath, format!("{:#?}", tokens)).unwrap();

    let parser = Parser::new(tokens);
    let ast = parser.parse_program();

    fs::write(ast_filepath, format!("{:#?}", ast)).unwrap();
    
    let hir = lower_ast(ast);

    fs::write(hir_filepath, format!("{:#?}", hir)).unwrap();
    
    let mut hircomp = HIRCompiler::new(hir);
    let assembly = hircomp.compile();

    fs::write(assembly_filename, assembly).unwrap();
}


