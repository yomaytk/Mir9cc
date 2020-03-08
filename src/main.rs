use std::env;

pub mod token;
pub mod parse;
pub mod ir;
pub mod regalloc;
pub mod codegen;
pub mod lib;

use token::*;
use parse::*;
use ir::*;
use regalloc::*;
use codegen::*;

#[macro_use]
extern crate lazy_static;

#[allow(dead_code)]
fn print_typename<T>(_: T) {
    println!("{}", std::any::type_name::<T>());
}

fn main() {
	let args: Vec<String> = env::args().collect();
	
	let mut dump_ir1 = false;
	let mut dump_ir2 = false;

	if args.len() == 4 && args[1] == "-dump-ir1" && args[2] == "-dump-ir2" {
		dump_ir1 = true;
		dump_ir2 = true;
	} else if args.len() == 3 && args[1] == "-dump-ir1" {
		dump_ir1 = true;
	} else if args.len() == 3 {
		dump_ir2 = true;
	} else if args.len() == 2 {
	} else {
		panic!("Usage: mir9cc [-dump-ir1] [-dump-ir2] <code>");
	}

	let p:String = (&args[args.len()-1][..]).chars().collect();
	
	// lexical analysis
	let tokens = tokenize(&p);
	// for token in &tokens {
	// 	println!("{:?}", token);
	// }

	// parsing analysis
	let node = parse(&tokens, &mut 0);
	// println!("{:#?}", &node);

	
	// alloc index for register
	let mut irv = gen_ir(&node);
	if dump_ir1 {
		IrInfo::dump_ir(&irv, "-dump-ir1");
	}
	// for ir in &irv {
	// 	println!("{:?}", ir);
	// }
	let mut reg_map: [i32; 10000] = [-1; 10000];
	let mut used: [bool; 8] = [false; 8];
	alloc_regs(&mut reg_map, &mut used, &mut irv);
	if dump_ir2 {
		IrInfo::dump_ir(&irv, "-dump-ir2");
	}

    println!(".intel_syntax noprefix");
    println!(".global main");
    println!("main:");
	
	// code generator
	gen_x86(&irv);
}
