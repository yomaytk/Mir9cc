use super::gen_ir::{*, IrOp::*};

pub static REG: [&str; 8] = ["rbp", "r10", "r11", "rbx", "r12", "r13", "r14", "r15"];
pub static REG8: [&str; 8] = ["bpl", "r10b", "r11b", "bl", "r12b", "r13b", "r14b", "r15b"];
pub static REG32: [&str; 8] = ["ebp", "r10d", "r11d", "ebx", "r12d", "r13d", "r14d", "r15d"];
pub static ARGREG64: [&str; 6] = ["rdi", "rsi", "rdx", "rcx", "r8", "r9"];
pub static ARGREG32: [&str; 6] = ["edi", "esi", "edx", "ecx", "r8d", "r9d"];

pub fn gen(fun: &Function, label: usize) {

	println!(".global {}", fun.name);
	println!("{}:", fun.name);
	println!("\tpush rbp");
	println!("\tmov rbp, rsp");
	println!("\tsub rsp, {}", fun.stacksize);
	println!("\tpush r12");
	println!("\tpush r13");
	println!("\tpush r14");
	println!("\tpush r15");

	let ret = format!(".Lend{}", label);

	for ir in &fun.irs {
		match &ir.op {
			IrImm => {
				println!("\tmov {}, {}", REG[ir.lhs], ir.rhs);
			}
			IrMov => {
				println!("\tmov {}, {}", REG[ir.lhs], REG[ir.rhs]);
			}
			IrAdd => {
				println!("\tadd {}, {}", REG[ir.lhs], REG[ir.rhs]);
			}
			IrSub => {
				println!("\tsub {}, {}", REG[ir.lhs], REG[ir.rhs]);
			}
			IrSubImm => {
				println!("\tsub {}, {}", REG[ir.lhs], ir.rhs);
			}
			IrMul => {
				println!("\tmov rax, {}", REG[ir.rhs]);
				println!("\tmul {}", REG[ir.lhs]);
				println!("\tmov {}, rax", REG[ir.lhs]);
			}
			IrDiv => {
				println!("\tmov rax, {}", REG[ir.lhs]);
				println!("\tcqo");
				println!("\tdiv {}", REG[ir.rhs]);
				println!("\tmov {}, rax", REG[ir.lhs]);
			}
			IrRet => {
				*LABEL.lock().unwrap() += 1;
				println!("\tmov rax, {}", REG[ir.lhs]);
				println!("\tjmp {}", ret);
			}
			IrStore32 => {
				println!("\tmov [{}], {}", REG[ir.lhs], REG32[ir.rhs]);
			}
			IrStore64 => {
				println!("\tmov [{}], {}", REG[ir.lhs], REG[ir.rhs]);
			}
			IrLoad32 => {
				println!("\tmov {}, [{}]", REG32[ir.lhs], REG[ir.rhs]);
			}
			IrLoad64 => {
				println!("\tmov {}, [{}]", REG[ir.lhs], REG[ir.rhs]);
			}
			IrUnless => {
				println!("\tcmp {}, 0", REG[ir.lhs]);
				println!("\tje .L{}", ir.rhs);
			}
			IrLabel => {
				println!(".L{}:", ir.lhs);
			}
			IrJmp => {
				println!("\tjmp .L{}", ir.lhs);
			}
			IrCall { name, len , args } => {

				for i in 0..*len {
					println!("\tmov {}, {}", ARGREG64[i], REG[args[i]]);
				}
				
				println!("\tpush r10");
				println!("\tpush r11");
				println!("\tmov rax, 0");
				println!("\tcall {}", name);
				println!("\tpop r11");
				println!("\tpop r10");
				
				println!("\tmov {}, rax", REG[ir.lhs]);
			}
			IrStoreArgs32 => {
				println!("\tmov [rbp-{}], {}", ir.lhs, ARGREG32[ir.rhs]);
			}
			IrStoreArgs64 => {
				println!("\tmov [rbp-{}], {}", ir.lhs, ARGREG64[ir.rhs]);
			}
			IrLt => {
				println!("\tcmp {}, {}", REG[ir.lhs], REG[ir.rhs]);
				println!("\tsetl {}", REG8[ir.lhs]);
				println!("\tmovzb {}, {}", REG[ir.lhs], REG8[ir.lhs]);
			}
			IrNop => {},
			_ => { panic!("unexpected IrOp in gen_x86"); }
		}
	}
	
	println!("{}:", ret);
	println!("\tpop r15");
	println!("\tpop r14");
	println!("\tpop r13");
	println!("\tpop r12");
	println!("\tmov rsp, rbp");
	println!("\tpop rbp");
	println!("\tret");
}

pub fn gen_x86(funcs: &Vec<Function>) {
	
    println!(".intel_syntax noprefix");

	for i in 0..funcs.len() {
		gen(&funcs[i], i);
	}
}