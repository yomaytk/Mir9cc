use super::gen_ir::{*, IrOp::*};
use super::parse::roundup;
use super::mir::*;

use std::collections::HashMap;
use std::sync::Mutex;

// This pass generates x86-64 assembly from IR.

macro_rules! hash {
	( $( $t:expr),* ) => {
		{
			let mut temp_hash = HashMap::new();
			$(
				temp_hash.insert($t.0, $t.1);
			)*
			temp_hash
		}
	};
}

macro_rules! emit{
    ($fmt:expr) => (print!(concat!("\t", $fmt, "\n")));
    ($fmt:expr, $($arg:tt)*) => (print!(concat!("\t", $fmt, "\n"), $($arg)*));
}

pub static REG8: [&str; 7] = ["r10b", "r11b", "bl", "r12b", "r13b", "r14b", "r15b"];
pub static REG32: [&str; 7] = ["r10d", "r11d", "ebx", "r12d", "r13d", "r14d", "r15d"];
pub static REG64: [&str; 7] = ["r10", "r11", "rbx", "r12", "r13", "r14", "r15"];
pub static ARGREG8: [&str; 6] = ["dil", "sil", "dl", "cl", "r8b", "r9b"];
pub static ARGREG32: [&str; 6] = ["edi", "esi", "edx", "ecx", "r8d", "r9d"];
pub static ARGREG64: [&str; 6] = ["rdi", "rsi", "rdx", "rcx", "r8", "r9"];

lazy_static! {
	pub static ref BACKSLASH_ESCAPED: Mutex<HashMap<char, char>> = Mutex::new(hash![
		/*('b', '\b'), ('f', '\f'),*/ ('\n', 'n'), ('\r', 'r'),
		('\t', 't'), ('\\', '\\'), ('\'', '\''), ('\"', '\"')
	]);
}

fn escape(strname: String, len: i32) -> String {
	let mut p = strname.chars();
	let mut name = String::new();

	for _ in 0..len {
		if let Some(c) = p.next(){
			if let Some(c2) = BACKSLASH_ESCAPED.lock().unwrap().get(&c) {
				name.push('\\');
				name.push(*c2);
			} else if c.is_ascii_graphic() || c == ' ' {
				name.push(c);
			} else {
				name.push_str(&format!("\\{:o}", c as i8));
			}
		} else {
			name.push_str("\\000");
		}
	}
	name.push_str("\\000");
	return name;
}

fn emit_cmp(ir: &Ir, insn: String) {
	let r0 = ir.r0 as usize;
	let r2 = ir.r2 as usize;
	emit!("cmp {}, {}", REG64[r0], REG64[r2]);
	emit!("{} {}", insn, REG8[r0]);
	emit!("movzb {}, {}", REG64[r0], REG8[r0]);
}

fn reg(size: i32, r: usize) -> &'static str {
	if size == 1 { return REG8[r]; }
	else if size == 4 { return REG32[r]; }
	else { return REG64[r]; }
}

fn argreg(size: i32, r: usize) -> &'static str {
	if size == 1 { return ARGREG8[r]; }
	else if size == 4 { return ARGREG32[r]; }
	else { return ARGREG64[r]; }
}

fn emit_ir(ir: &Ir, ret: &str) {
	
	let r0 = ir.r0 as usize;
	let r2 = ir.r2 as usize;
	match &ir.op {
		IrImm => {
			emit!("mov {}, {}", REG64[r0], ir.imm);
		}
		IrMov => {
			emit!("mov {}, {}", REG64[r0], REG64[r2]);
		}
		IrAdd => {
			emit!("add {}, {}", REG64[r0], REG64[r2]);
		}
		IrSub => {
			emit!("sub {}, {}", REG64[r0], REG64[r2]);
		}
		IrBpRel => {
			emit!("lea {}, [rbp-{}]", REG64[r0], ir.imm);
		}
		IrMul => {
			emit!("mov rax, {}", REG64[r2]);
			emit!("imul {}", REG64[r0]);
			emit!("mov {}, rax", REG64[r0]);
		}
		IrDiv => {
			emit!("mov rax, {}", REG64[r0]);
			emit!("cqo");
			emit!("idiv {}", REG64[r2]);
			emit!("mov {}, rax", REG64[r0]);
		}
		IrRet => {
			emit!("mov rax, {}", REG64[r0]);
			emit!("jmp {}", ret);
		}
		IrStore(size) => {
			emit!("mov [{}], {}", REG64[r0], reg(*size, r2));
		}
		IrLoad(size) => {
			emit!("mov {}, [{}]", reg(*size, r0), REG64[r2]);
			if *size == 1 {
				emit!("movzb {}, {}", REG64[r0], REG8[r2]);
			}
		}
		IrBr => {
			emit!("cmp {}, 0", REG64[r0]);
			emit!("jne .L{}", ir.bb1_label);
			emit!("jmp .L{}", ir.bb2_label);
		}
		IrJmp => {
			emit!("jmp .L{}", ir.bb1_label);
		}
		IrCall { name, len , args } => {

			for i in 0..*len {
				emit!("mov {}, {}", ARGREG64[i], REG64[args[i] as usize]);
			}
			
			emit!("push r10");
			emit!("push r11");
			emit!("mov rax, 0");
			emit!("call {}", name);
			emit!("pop r11");
			emit!("pop r10");
			
			emit!("mov {}, rax", REG64[r0]);
		}
		IrStoreArg(size) => {
			emit!("mov [rbp-{}], {}", ir.imm, argreg(*size, ir.imm2 as usize));
		}
		IrLt => {
			emit_cmp(ir, String::from("setl"));
		}
		IrLe => {
			emit_cmp(ir, String::from("setle"));
		}
		IrEqual => {
			emit_cmp(ir, String::from("sete"));
		}
		IrNe => {
			emit_cmp(ir, String::from("setne"));
		}
		IrLabelAddr(label) => {
			emit!("lea {}, {}", REG64[r0], label);
		}
		IrOr => {
			emit!("or {}, {}", REG64[r0], REG64[r2]);
		}
		IrXor => {
			emit!("xor {}, {}", REG64[r0], REG64[r2]);
		}
		IrAnd => {
			emit!("and {}, {}", REG64[r0], REG64[r2]);
		}
		IrShl => {
			emit!("mov cl, {}", REG8[r2]);
			emit!("shl {}, cl", REG64[r0]);
		}
		IrShr => {
			emit!("mov cl, {}", REG8[r2]);
			emit!("shr {}, cl", REG64[r0]);
		}
		IrMod => {
			emit!("mov rax, {}", REG64[r0]);
			emit!("cqo");
			emit!("idiv {}", REG64[r2]);
			emit!("mov {}, rdx", REG64[r0]);
		}
		IrNeg => {
			emit!("neg {}", REG64[r0]);
		}
	}
}

pub fn gen(fun: &Function, label: usize) {

	// program
	println!(".text");
	println!(".global {}", fun.name);
	println!("{}:", fun.name);
	emit!("push rbp");
	emit!("mov rbp, rsp");
	emit!("sub rsp, {}", roundup(fun.stacksize, 16));
	emit!("push r12");
	emit!("push r13");
	emit!("push r14");
	emit!("push r15");

	let ret = format!(".Lend{}", label);

	for bb in &fun.bbs {
		println!(".L{}:", bb.label);
		for ir in &bb.irs {
			emit_ir(ir, &ret);
		}
	}
	
	println!("{}:", ret);
	emit!("pop r15");
	emit!("pop r14");
	emit!("pop r13");
	emit!("pop r12");
	emit!("mov rsp, rbp");
	emit!("pop rbp");
	emit!("ret");
}

pub fn gen_x86(program: Program) {
	
	println!(".intel_syntax noprefix");
	
	// global variable
	for gvar in program.gvars {
		if let Some(s) = gvar.strname {
			println!(".data");
			println!("{}:", gvar.labelname.unwrap());
			emit!(".ascii \"{}\"", escape(s, gvar.ctype.size));
		} else {
			println!(".bss");
			println!("{}:", gvar.labelname.unwrap());
			emit!(".zero {}", gvar.ctype.size);
		}
	}

	for i in 0..program.funs.len() {
		gen(&program.funs[i], i);
	}
}