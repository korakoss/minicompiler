use std::fs;
use std::env;
use std::path::Path;

mod shared;
mod stages;
mod passes;

use passes::{
    preproc::{lex::lex, parse::Parser},
    make_hir::lower_ast,
    hir_to_mir::*,
    concretize_mir::concretize_mir,
    cmir_to_lir::*,
    lir_codegen::*,
};


fn main() {
    
    let args: Vec<String> = env::args().collect();
    println!("\n \n \n {:?}", args);
    let source_path = Path::new(&args[1]);
    let target_dir = Path::new(&args[2]);

    let program_text = &fs::read_to_string(source_path).unwrap();

    let tokens = lex(program_text);
    fs::write(target_dir.join("tok.txt"), format!("{:#?}", tokens)).unwrap();

    let ast = Parser::parse_program(tokens);
    fs::write(target_dir.join("ast.txt"), format!("{:#?}", ast)).unwrap();

    let hir = lower_ast(ast);
    fs::write(target_dir.join("hir.txt"), format!("{:#?}", hir)).unwrap();
    
    let mir = MIRBuilder::lower_hir(hir);
    fs::write(target_dir.join("mir.txt"), format!("{:#?}", mir)).unwrap();

    let cmir = concretize_mir(mir).unwrap();
    fs::write(target_dir.join("cmir.txt"), format!("{:#?}", cmir)).unwrap();

    let lir = lower_cmir(cmir);
    fs::write(target_dir.join("lir.txt"), format!("{:#?}", lir)).unwrap();

    let assembly = LIRCompiler::compile(lir);
    fs::write(target_dir.join("asm.txt"), assembly).unwrap();
}


