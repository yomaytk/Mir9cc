use super::token::{*, TokenType::*};
use IrOp::*;
use IrType::*;
use super::parse::*;

use std::sync::Mutex;
use std::collections::HashMap;
use std::fmt;

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


lazy_static! {
	pub static ref REGNO: Mutex<usize> = Mutex::new(1);
	pub static ref IRINFO: Mutex<HashMap<IrOp, IrInfo>> = Mutex::new(hash![
		(IrOp::IrAdd, IrInfo::new("ADD", IrType::RegReg)),
		(IrOp::IrSub, IrInfo::new("SUB", IrType::RegReg)),
		(IrOp::IrMul, IrInfo::new("MUL", IrType::RegReg)),
		(IrOp::IrDiv, IrInfo::new("DIV", IrType::RegReg)),
		(IrOp::IrLt, IrInfo::new("LT", IrType::RegReg)),
		(IrOp::IrImm, IrInfo::new("MOV", IrType::RegImm)),
		(IrOp::IrSubImm, IrInfo::new("SUB", IrType::RegImm)),
		(IrOp::IrMov, IrInfo::new("MOV", IrType::RegReg)),
		(IrOp::IrLabel, IrInfo::new("", IrType::Label)),
		(IrOp::IrUnless, IrInfo::new("UNLESS", IrType::RegLabel)),
		(IrOp::IrRet, IrInfo::new("RET", IrType::Reg)),
		(IrOp::IrLoad8, IrInfo::new("LOAD8", IrType::RegReg)),
		(IrOp::IrLoad32, IrInfo::new("LOAD32", IrType::RegReg)),
		(IrOp::IrLoad64, IrInfo::new("LOAD64", IrType::RegReg)),
		(IrOp::IrStore8, IrInfo::new("STORE8", IrType::RegReg)),
		(IrOp::IrStore32, IrInfo::new("STORE32", IrType::RegReg)),
		(IrOp::IrStore64, IrInfo::new("STORE64", IrType::RegReg)),
		(IrOp::IrJmp, IrInfo::new("JMP", IrType::Label)),
		(IrOp::IrCall { name: format!(""), len: 0, args: vec![] }, IrInfo::new("CALL", IrType::Call)),
		(IrOp::IrStoreArgs8, IrInfo::new("STOREARGS8", IrType::ImmImm)),
		(IrOp::IrStoreArgs32, IrInfo::new("STOREARGS32", IrType::ImmImm)),
		(IrOp::IrStoreArgs64, IrInfo::new("STOREARGS64", IrType::ImmImm)),
		(IrOp::IrLabelAddr(String::new()), IrInfo::new("LABELADDR", IrType::LabelAddr)),
		(IrOp::IrEqEq, IrInfo::new("EqEq", IrType::RegReg)),
		(IrOp::IrNe, IrInfo::new("Neq", IrType::RegReg)),
		(IrOp::IrIf, IrInfo::new("If", IrType::Reg)),
		(IrOp::IrKill, IrInfo::new("KILL", IrType::Reg)),
		(IrOp::IrNop, IrInfo::new("NOP", IrType::NoArg))
	]);
	pub static ref LABEL: Mutex<usize> = Mutex::new(1);
	pub static ref RETURN_LABEL: Mutex<usize> = Mutex::new(0);
	pub static ref RETURN_REG: Mutex<usize> = Mutex::new(0);
}

#[allow(dead_code)]
#[derive(Debug, PartialEq, std::cmp::Eq, std::hash::Hash)]
pub enum IrOp {
	IrImm,
	IrMov,
	IrAdd,
	IrSubImm,
	IrSub,
	IrMul,
	IrDiv,
	IrRet,
	IrExpr,
	IrStore8,
	IrStore32,
	IrStore64,
	IrLoad8,
	IrLoad32,
	IrLoad64,
	IrLabel,
	IrUnless,
	IrJmp,
	IrCall { name: String, len: usize, args: Vec<usize> },
	IrStoreArgs8,
	IrStoreArgs32,
	IrStoreArgs64,
	IrLt,
	IrEqEq, 
	IrNe,
	IrIf,
	IrLabelAddr(String),
	IrKill,
	IrNop,
}

#[derive(Debug)]
pub struct Ir {
	pub op: IrOp,
	pub lhs: usize,
	pub rhs: usize,
}

impl Ir {
	fn new(ty: IrOp, lhs: usize, rhs: usize) -> Self {
		Self {
			op: ty,
			lhs: lhs,
			rhs: rhs,
		}
	}
	fn fouroperator2irop(ty: TokenType) -> IrOp {
		match ty {
			TokenAdd => { IrAdd },
			TokenSub => { IrSub },
			TokenStar => { IrMul },
			TokenDiv => { IrDiv },
			TokenLt => { IrLt },
			TokenEof => { panic!("tokeneof!!!"); }
			_ => { panic!("fouroperator2irop error."); }
		}
	}
	pub fn get_irinfo(&self) -> IrInfo {
		match &self.op {
			IrCall { name, len, args } => {
				let _name = name;
				let _len = len;
				let _args = args;
				return IRINFO.lock().unwrap().get(&IrOp::IrCall { name: format!(""), len: 0, args: vec![] }).unwrap().clone();
			},
			IrLabelAddr(_) => {
				return IRINFO.lock().unwrap().get(&IrOp::IrLabelAddr(String::new())).unwrap().clone();
			}
			_ => {
				return IRINFO.lock().unwrap().get(&self.op).unwrap().clone();
			}
		}
	}
	fn tostr(&self) -> String {
		let irinfo = self.get_irinfo();
		match irinfo.ty.clone() {
			NoArg => { format!("Nop") },
			Reg => { format!("{}, r{}", irinfo.ty, self.lhs) },
			Label => { format!("{}", self.lhs) },
			RegReg => { format!("{} r{}, r{}", irinfo.name, self.lhs, self.rhs) },
			RegImm => { format!("{} r{}, {}", irinfo.name, self.lhs, self.rhs) },
			RegLabel => { format!("{} r{}, .L{}", irinfo.name, self.lhs, self.rhs) },
			Call => {
				match &self.op {
					IrCall{ name, len, args } => {
						let _len = len;
						let mut s = String::from(format!("{} {}(", irinfo.name, name));
						for arg in args {
							s += &format!(", {}", arg);
						}
						s += &format!("), {}, {})", self.lhs, self.rhs);
						return s;
					}
					_ => { panic!("tostr call error {}"); }
				}
			}
			Imm => { format!("{} {}", irinfo.name, self.lhs) },
			ImmImm => { format!("{} {} {}", irinfo.name, self.lhs, self.rhs) }
			LabelAddr => { format!("{} r{} .L.str{}", irinfo.name, self.lhs, self.rhs) }
		}
	}
}

#[derive(Debug, Clone, PartialEq)]
pub enum IrType {
	NoArg,
	Reg,
	Label,
	RegReg,
	RegImm,
	RegLabel,
	Call,
	Imm,
	ImmImm,
	LabelAddr,
}

impl fmt::Display for IrType {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		match self {
			NoArg => { write!(f, "NoArg") },
			Reg => { write!(f, "Reg") },
			Label => { write!(f, "Label") },
			RegReg => { write!(f, "RegReg") },
			RegImm => { write!(f, "RegImm") },
			RegLabel => { write!(f, "RegLabel") },
			Call => { write!(f, "Call") },
			Imm => { write!(f, "Imm") },
			ImmImm => { write!(f, "ImmImm") },
			LabelAddr => { write!(f, "LabelAddr") },
		}
	}
}

#[derive(Debug, PartialEq, Clone)]
pub struct IrInfo {
	pub name: &'static str,
	pub ty: IrType,
}

impl IrInfo {
	fn new(name: &'static str, ty: IrType) -> Self {
		Self {
			name,
			ty,
		}
	}
	pub fn dump_ir(irv: &Vec<Function>, dump_option: &str){
		println!("{}: ", dump_option);
		for fun in irv {
			println!("{}():", fun.name);
			for ir in &fun.irs {
				println!("{}", ir.tostr());
			}
		}
	}
}

pub struct Function {
	pub name: String,
	pub irs: Vec<Ir>,
	pub stacksize: usize,
}

impl Function {
	fn new(name: String, irs: Vec<Ir>, stacksize: usize) -> Self {
		Self {
			name,
			irs,
			stacksize,
		}
	}
}

fn kill(r: usize, code: &mut Vec<Ir>) {
	code.push(Ir::new(IrKill, r, 0));
}

fn label(r: usize, code: &mut Vec<Ir>) {
	code.push(Ir::new(IrLabel, r, 0));
}

fn gen_lval(node: &Node, code: &mut Vec<Ir>) -> usize {
	
	match &node.op {
		NodeType::Deref(_, expr) => {
			return gen_expr(expr, code);
		}
		NodeType::Lvar(_, off) => {
			*REGNO.lock().unwrap() += 1;
			let r1 = *REGNO.lock().unwrap();
			code.push(Ir::new(IrMov, r1, 0)); 
			code.push(Ir::new(IrSubImm, r1, *off));
			return r1;
		}
		NodeType::Gvar(_, label) => {
			*REGNO.lock().unwrap() += 1;
			let r = *REGNO.lock().unwrap();
			code.push(Ir::new(IrLabelAddr(label.clone()), r, 0));
			return r;
		}
		_ => { panic!("not an lvalue")}
	}
}

// allocate of index for register to NodeNum
fn gen_expr(node: &Node, code: &mut Vec<Ir>) -> usize {

	match &node.op {
		NodeType::Num(val) => {
			*REGNO.lock().unwrap() += 1;
			let r = *REGNO.lock().unwrap();
			let ir = Ir::new(IrImm, r, *val as usize);
			code.push(ir);
			return r;
		},
		NodeType::LogAnd(lhs, rhs) => {
			let r1 = gen_expr(lhs, code);
			*LABEL.lock().unwrap() += 1;
			let x = *LABEL.lock().unwrap();
			code.push(Ir::new(IrUnless, r1, x));
			let r2 = gen_expr(rhs, code);
			code.push(Ir::new(IrMov, r1, r2));
			kill(r2, code);
			code.push(Ir::new(IrUnless, r1, x));
			code.push(Ir::new(IrImm, r1, 1));
			label(x, code);
			return r1;
		}
		NodeType::LogOr(lhs, rhs) => {
			let r1 = gen_expr(lhs, code);
			*LABEL.lock().unwrap() += 2;
			let x = *LABEL.lock().unwrap()-1;
			let y = x+1;
			code.push(Ir::new(IrUnless, r1, x));
			code.push(Ir::new(IrImm, r1, 1));
			code.push(Ir::new(IrJmp, y, 0));
			label(x, code);
			let r2 = gen_expr(rhs, code);
			code.push(Ir::new(IrMov, r1, r2));
			kill(r2, code);
			code.push(Ir::new(IrUnless, r1, y));
			code.push(Ir::new(IrImm, r1, 1));
			label(y, code);
			return r1;
		}
		NodeType::BinaryTree(ctype, ty, lhs, rhs) => {
			let lhi = gen_expr(lhs.as_ref().unwrap(), code);
			let rhi = gen_expr(rhs.as_ref().unwrap(), code);
			match ty {
				TokenAdd | TokenSub if ctype.ty == Ty::PTR => {
					if let Some(lhs2) = lhs {
						match &lhs2.op {
							NodeType::Lvar(ctype, _) | NodeType::BinaryTree(ctype, _, _, _) 
							| NodeType::Deref(ctype, _) | NodeType::Addr(ctype, _) => {
								let size_of = ctype.ptr_of.as_ref().unwrap().size_of();
								*REGNO.lock().unwrap() += 1;
								let r1 = *REGNO.lock().unwrap();
								code.push(Ir::new(IrImm, r1, size_of));
								code.push(Ir::new(IrMul, rhi, r1));
								kill(r1, code);
							}
							_ => { panic!("operand must be pointer."); }
						}
					}
				}
				_ => {}
			}
			code.push(Ir::new(Ir::fouroperator2irop(ty.clone()), lhi, rhi));
			kill(rhi, code);
			return lhi;
		},
		NodeType::Lvar(ctype, _) | NodeType::Gvar(ctype, _) => {
			let lhi = gen_lval(node, code);
			match ctype.ty {
				Ty::CHAR => { code.push(Ir::new(IrLoad8, lhi, lhi)); }
				Ty::INT => { code.push(Ir::new(IrLoad32, lhi, lhi)); }
				_ => { code.push(Ir::new(IrLoad64, lhi, lhi)); }
			}
			return lhi;
		},
		NodeType::EqTree(ctype, lhs, rhs) => {
			let lhi = gen_lval(lhs, code);
			let rhi = gen_expr(rhs, code);
			match ctype.ty {
				Ty::CHAR => { code.push(Ir::new(IrStore8, lhi, rhi)); }
				Ty::INT => { code.push(Ir::new(IrStore32, lhi, rhi)); }
				_ => { code.push(Ir::new(IrStore64, lhi, rhi)); }
			}
			kill(rhi, code);
			return lhi;
		},
		NodeType::Call(ident, callarg) => {
			let mut args = vec![];
			for arg in callarg {
				args.push(gen_expr(arg, code));
			}
			*REGNO.lock().unwrap() += 1;
			let r = *REGNO.lock().unwrap();
			code.push(Ir::new(IrCall{ 
				name: (*ident).clone(), 
				len: args.len(),
				args: args.clone()
			} , r, 0));
			for arg in args {
				kill(arg, code);
			}
			return r;
		}
		NodeType::Deref(_, lhs) => {
			let r = gen_expr(lhs, code);
			if lhs.hasctype() {
				match lhs.nodesctype().ptr_of.unwrap().ty{
					Ty::CHAR => { code.push(Ir::new(IrLoad8, r, r)); }
					Ty::INT => { code.push(Ir::new(IrLoad32, r, r)); }
					_ => { code.push(Ir::new(IrLoad64, r, r)); }
				}
			}
			return r;
		}
		NodeType::Addr(_, lhs) => {
			return gen_lval(lhs, code);
		}
		NodeType::EqEq(lhs, rhs) => {
			let r1 = gen_expr(lhs, code);
			let r2 = gen_expr(rhs, code);
			code.push(Ir::new(IrEqEq, r1, r2));
			kill(r2, code);
			return r1;
		}
		NodeType::Ne(lhs, rhs) => {
			let r1 = gen_expr(lhs, code);
			let r2 = gen_expr(rhs, code);
			code.push(Ir::new(IrNe, r1, r2));
			kill(r2, code);
			return r1;
		}
		NodeType::StmtExpr(_, body) => {
			let orig_label = *RETURN_LABEL.lock().unwrap();
			let orig_reg = *RETURN_REG.lock().unwrap();
			*LABEL.lock().unwrap() += 1;
			*REGNO.lock().unwrap() += 1;
			*RETURN_LABEL.lock().unwrap() = *LABEL.lock().unwrap();
			*RETURN_REG.lock().unwrap() = *REGNO.lock().unwrap();
			let r = *RETURN_REG.lock().unwrap();

			gen_stmt(body, code);
			label(*RETURN_LABEL.lock().unwrap(), code);

			*RETURN_LABEL.lock().unwrap() = orig_label;
			*RETURN_REG.lock().unwrap() = orig_reg;

			return r;
		}
		_ => { panic!("gen_expr NodeType error at {:?}", node.op); }
	}

}


fn gen_stmt(node: &Node, code: &mut Vec<Ir>) {
	match &node.op {
		NodeType::Ret(lhs) => {
			
			let lhi= gen_expr(lhs.as_ref(), code);
			
			if *RETURN_LABEL.lock().unwrap() > 0 {
				code.push(Ir::new(IrMov, *RETURN_REG.lock().unwrap(), lhi));
				kill(lhi, code);
				code.push(Ir::new(IrJmp, *RETURN_LABEL.lock().unwrap(), 0));
				return;
			}

			code.push(Ir::new(IrRet, lhi, 0));
			kill(lhi, code);
		}
		NodeType::Expr(lhs) => {
			let r = gen_expr(lhs.as_ref(), code);
			kill(r, code);
		}
		NodeType::IfThen(cond, then, elthen) => {
			let lhi = gen_expr(cond, code);
			*LABEL.lock().unwrap() += 1;
			code.push(Ir::new(IrUnless, lhi, *LABEL.lock().unwrap()));
			kill(lhi, code);
			gen_stmt(then, code);
			match elthen {
				Some(elnode) => {
					code.push(Ir::new(IrJmp, *LABEL.lock().unwrap(), 0));
					label(*LABEL.lock().unwrap(), code);
					gen_stmt(elnode, code);
					*LABEL.lock().unwrap() += 1;
					label(*LABEL.lock().unwrap(), code);
				},
				None => {
					label(*LABEL.lock().unwrap(), code);
				}
			}
		}
		NodeType::CompStmt(lhs) => {
			for stmt in lhs {
				gen_stmt(stmt, code);
			}
		}
		NodeType::For(init, cond, inc, body) => {
			*LABEL.lock().unwrap() += 2;
			let x = *LABEL.lock().unwrap()-1;
			let y = x+1;
			gen_stmt(init, code);
			label(x, code);
			let r2 = gen_expr(cond, code);
			code.push(Ir::new(IrUnless, r2, y));
			kill(r2, code);
			gen_stmt(body, code);
			gen_expr(inc, code);
			code.push(Ir::new(IrJmp, x, 0));
			label(y, code);
		}
		NodeType::DoWhile(body, cond) => {
			*LABEL.lock().unwrap() += 1;
			let x = *LABEL.lock().unwrap();
			label(x, code);
			gen_stmt(body, code);
			let r = gen_expr(cond, code);
			code.push(Ir::new(IrIf, r, x));
			kill(r, code);
		}
		NodeType::VarDef(ctype, _, _, off, init) => {
			if let Some(rhs) = init {
				*REGNO.lock().unwrap() += 1;
				let r1 = *REGNO.lock().unwrap();
				code.push(Ir::new(IrMov, r1, 0));
				code.push(Ir::new(IrSubImm, r1, *off));
				let r2 = gen_expr(rhs, code);
				match ctype.ty {
					Ty::CHAR => { code.push(Ir::new(IrStore8, r1, r2)); }
					Ty::INT => { code.push(Ir::new(IrStore32, r1, r2)); }
					_ => { code.push(Ir::new(IrStore64, r1, r2)); }
				}
				kill(r1, code);
				kill(r2, code);
			}
		}
		enode => { panic!("unexpeceted node {:?}", enode); }
	}
}

// generate IR Vector
pub fn gen_ir(funcs: &Vec<Node>) -> Vec<Function> {
	
	let mut v = vec![];

	for funode in funcs {
		
		let mut code = vec![];
		*REGNO.lock().unwrap() = 1;
		
		match &funode.op {
			NodeType::Func(name, is_extern, args, body, stacksize) => {
				if *is_extern {
					continue;
				}
				for i in 0..args.len() {
					match &args[i].op {
						NodeType::VarDef(ctype, _, _, offset, _) => {
							match ctype.ty {
								Ty::CHAR => { code.push(Ir::new(IrStoreArgs8, *offset, i)); } 
								Ty::INT => { code.push(Ir::new(IrStoreArgs32, *offset, i)); } 
								_ => { code.push(Ir::new(IrStoreArgs64, *offset, i)); }
							}
						}
						_ => { panic!("Illegal function parameter."); }
					}
				}
				gen_stmt(body, &mut code);
				let func = Function::new(name.clone(), code, *stacksize);
				v.push(func);
			}
			NodeType::VarDef(_, _, _, _, _) => {}
			_ => { panic!(" should be func node at gen_ir: {:?}", funode); }
		}
	}

	return v;
}