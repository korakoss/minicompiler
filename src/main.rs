use std::fs;
use std::env;

mod common;
mod ast;

mod lex;
use lex::*;

mod parse;
use parse::*;

mod convert_ast;
use convert_ast::*;

mod hir;

mod make_hir;
use make_hir::*;

/*
use hir_builder::*;

mod hir_codegen;
use hir_codegen::*;
*/

fn main() {
    
    let args: Vec<String> = env::args().collect();
    let code_filename = &args[1];
    let assembly_filename = &args[2];
    let tokens_filepath = &args[3];
    let uast_filepath = &args[4];
    let tast_filepath = &args[5];
    let hir_filepath = &args[6];

    let program_text = &fs::read_to_string(code_filename).unwrap();
    let tokens = lex(program_text);
    
    fs::write(tokens_filepath, format!("{:#?}", tokens)).unwrap();

    let uast = Parser::parse_program(tokens);

    fs::write(uast_filepath, format!("{:#?}", uast)).unwrap();

    let tast = ASTConverter::convert_uast(uast);
    fs::write(tast_filepath, format!("{:#?}", tast)).unwrap();

    let hir = lower_ast(tast);
    fs::write(hir_filepath, format!("{:#?}", hir)).unwrap();

    /* 
        
    let mut hircomp = HIRCompiler::new(hir);
    let assembly = hircomp.compile();

    fs::write(assembly_filename, assembly).unwrap();
    */
}


