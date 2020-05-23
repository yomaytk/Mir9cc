use super::token::*;
use super::token::TokenType::*;
// use super::lib::*;
use super::mir::*;
use super::sema::*;

use std::collections::HashMap;
use std::sync::Mutex;

// This is a recursive-descendent parser which constructs abstract
// syntax tree from input tokenset.
//
// This parser knows only about BNF of the C grammer and doesn't care
// about its semantics. Therefore, some invalid expressions, such as
// `1+2=3`, are accepted by this parser, but that's intentional.
// Semantic errors are detected in a later pass.

macro_rules! env_find {
	($s:expr, $m:ident, $null:expr) => {
		{
			let mut target = $null;
			if let Some(t) = ENV.lock().unwrap().$m.get(&$s) {
				target = t.clone()
			}
			if target == $null {
				let mut env = &ENV.lock().unwrap().next;
				while let Some(e) = env {
					if let Some(t) = e.$m.get(&$s) {
						target = t.clone();
						break;
					}
					env = &e.next;
				}
			}
			target
		}
	};
}

lazy_static! {
	pub static ref INT_TY: Type = Type {
		ty: Ty::INT,
		ptr_to: None,
		ary_to: None,
		size: 4,
		align: 4,
		offset: 0,
		len: 0,
		is_extern: false,
	};
	pub static ref CHAR_TY: Type = Type {
		ty: Ty::CHAR,
		ptr_to: None,
		ary_to: None,
		size: 1,
		align: 1,
		offset: 0,
		len: 0,
		is_extern: false,
	};
	pub static ref VOID_TY: Type = Type {
		ty: Ty::VOID,
		ptr_to: None,
		ary_to: None,
		size: 0,
		align: 0,
		offset: 0,
		len: 0,
		is_extern: false,
	};
	pub static ref NULL_TY: Type = Type {
		ty: Ty::NULL,
		ptr_to: None,
		ary_to: None,
		size: 0,
		align: 0,
		offset: 0,
		len: 0,
		is_extern: false,
	};
	pub static ref STRUCT_TY: Type = Type {
		ty: Ty::STRUCT(String::new(), Vec::new()),
		ptr_to: None,
		ary_to: None,
		size: 0,
		align: 0,
		offset: 0,
		len: 0,
		is_extern: false,
	};
	pub static ref NULL_VAR: Var = Var {
		ctype: NULL_TY.clone(),
		offset: 0,
		is_local: true,
		labelname: None,
		strname: None,
		is_extern: false,
	};
	pub static ref ENV: Mutex<Env> = Mutex::new(Env::new_env(None));
	pub static ref STACKSIZE: Mutex<usize> = Mutex::new(0);
	pub static ref GVARS: Mutex<Vec<Var>> = Mutex::new(vec![]);
	pub static ref STRLABEL: Mutex<usize> = Mutex::new(0);
}

#[derive(Debug, Clone, PartialEq)]
pub struct Type {
	pub ty: Ty,
	pub ptr_to: Option<Box<Type>>,
	pub ary_to: Option<Box<Type>>,
	pub size: usize,
	pub align: usize,
	pub offset: usize,
	pub len: usize,
	pub is_extern: bool,
}

impl Type {
	pub fn new(ty: Ty, ptr_to: Option<Box<Type>>, ary_to: Option<Box<Type>>, size: usize, align: usize, offset: usize, len: usize, is_extern: bool) -> Self {
		Self {
			ty,
			ptr_to,
			ary_to,
			size, 
			align,
			offset,
			len,
			is_extern,
		}
	}
	pub fn ptr_to(self) -> Self {
		let is_extern = self.is_extern;
		Self {
			ty: Ty::PTR,
			ptr_to: Some(Box::new(self)),
			ary_to: None,
			size: 8,
			align: 8,
			offset: 0,
			len: 0,
			is_extern,
		}
	}
	pub fn ary_of(self, len: usize) -> Self {
		let size = self.size;
		let align = self.align;
		let is_extern = self.is_extern;
		Self {
			ty: Ty::ARY,
			ptr_to: None,
			ary_to: Some(Box::new(self)),
			size: size * len,
			align,
			offset: 0,
			len,
			is_extern,
		}
	}
}

#[derive(Debug, Clone)]
pub enum Ty {
	INT,
	PTR,
	ARY,
	CHAR,
	STRUCT(String, Vec<Node>),
	VOID,
	NULL,
}

impl PartialEq for Ty {
	fn eq(&self, other: &Self) -> bool {
		match (self, other) {
			(Ty::INT, Ty::INT) | (Ty::PTR, Ty::PTR) | (Ty::ARY, Ty::ARY)
			| (Ty::CHAR, Ty::CHAR) | (Ty::VOID, Ty::VOID) | (Ty::NULL, Ty::NULL) => {
				return true;
			}
			(Ty::STRUCT(tag1, _), Ty::STRUCT(tag2, _)) => {
				if tag1 == tag2 {
					return true;
				} else {
					return false;
				}
			}
			_ => {
				return false;
			}
		}
	}
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub enum NodeType {
	Num(i32),																	// Num(val)
	BinaryTree(Type, TokenType, Box<Node>, Box<Node>),							// BinaryTree(ctype, tk_ty, lhs, rhs)
	Ret(Box<Node>),																// Ret(lhs)
	Expr(Box<Node>),															// Expr(lhs)
	CompStmt(Vec<Node>),														// CompStmt(stmts)
	StmtExpr(Type, Box<Node>),													// StmtExpr(ctype, body)
	Ident(String),																// Ident(s)
	EqTree(Type, Box<Node>, Box<Node>),											// EqTree(ctype, lhs, rhs)
	IfThen(Box<Node>, Box<Node>, Option<Box<Node>>),							// IfThen(cond, then, elthen)
	Call(Type, String, Vec<Node>),												// Call(ctype, ident, args)
	Func(Type, String, Vec<Node>, Box<Node>, usize),							// Func(ctype, ident, is_extern, args, body, stacksize)
	LogAnd(Box<Node>, Box<Node>),												// LogAnd(lhs, rhs)
	LogOr(Box<Node>, Box<Node>),												// LogOr(lhs, rhs)
	For(Box<Node>, Box<Node>, Box<Node>, Box<Node>),							// For(init, cond, inc, body)
	VarDef(String, Var, Option<Box<Node>>),										// VarDef(name, var, init)
	Deref(Type, Box<Node>),														// Deref(ctype, lhs)
	Addr(Type, Box<Node>),														// Addr(ctype, lhs)
	Sizeof(Type, usize, Box<Node>),												// Sizeof(ctype, val, lhs)
	EqEq(Box<Node>, Box<Node>),													// EqEq(lhs, rhs)
	Ne(Box<Node>, Box<Node>),													// Ne(lhs, rhs)
	DoWhile(Box<Node>, Box<Node>),												// Dowhile(boyd, cond)
	Alignof(Box<Node>),															// Alignof(expr)
	Dot(Type, Box<Node>, String),												// Dot(ctype, expr, name)
	Not(Box<Node>),																// Not(expr)
	Ternary(Type, Box<Node>, Box<Node>, Box<Node>),								// Ternary(ctype, cond, then, els)
	TupleExpr(Type, Box<Node>, Box<Node>),										// TupleExpr(ctype, lhs, rhs)
	Neg(Box<Node>),																// Neg(expr)
	IncDec(Type, i32, Box<Node>),												// IncDec(ctype, selector, expr)
	Decl(Type, String, Vec<Node>),												// Decl(ctype, ident, args)
	Var(Var),																	// Var(var)
	Break,																		// Break
	NULL,																		// NULL
}

#[derive(Debug, Clone)]
pub struct Node {
	pub op: NodeType,
}

#[allow(dead_code)]
impl Node {
	
	pub fn nodesctype(&self, basetype: Option<Type>) -> Type {
		match &self.op {
			| NodeType::BinaryTree(ctype, ..) 
			| NodeType::Deref(ctype,..) | NodeType::Addr(ctype, ..) 
			| NodeType::Sizeof(ctype, ..) | NodeType::Dot(ctype, ..) 
			| NodeType::Ternary(ctype, ..) | NodeType::IncDec(ctype, ..) 
			| NodeType::EqTree(ctype, ..) => { 
				return ctype.clone(); 
			}
			| NodeType::Var(var) | NodeType::VarDef(_, var, ..) => {
				return  var.ctype.clone();
			}
			_ => { 
				if let Some(ty) = basetype {
					return ty;
				} else {
					return NULL_TY.clone();
				}
			}
		} 
	}

	pub fn checklval(&self) {
		match &self.op {
			NodeType::Var(..) | NodeType::Deref(..) | NodeType::Dot(..) => {}
			_ => { 
				// error("not an lvalue");
				// for debug.
				panic!("not an lvalue");
			}
		}
	}

	pub fn new_num(val: i32) -> Self {
		Self {
			op: NodeType::Num(val),
		}
	}
	pub fn new_bit(ctype: Type, tk_ty: TokenType, lhs: Node, rhs: Node) -> Self {
		Self {
			op: NodeType::BinaryTree(ctype, tk_ty, Box::new(lhs), Box::new(rhs))
		}
	}
	pub fn new_ret(lhs: Node) -> Self {
		Self {
			op: NodeType::Ret(Box::new(lhs))
		}
	}
	pub fn new_expr(lhs: Node) -> Self {
		Self {
			op: NodeType::Expr(Box::new(lhs))
		}
	}
	pub fn new_stmt(stmts: Vec<Node>) -> Self {
		Self {
			op: NodeType::CompStmt(stmts)
		}
	}
	pub fn new_ident(ident: String) -> Self {
		Self {
			op: NodeType::Ident(ident)
		}
	}
	pub fn new_eq(ctype: Type, lhs: Node, rhs: Node) -> Self {
		Self {
			op: NodeType::EqTree(ctype, Box::new(lhs), Box::new(rhs))
		}
	}
	pub fn new_if(cond: Node, then: Node, elthen: Option<Node>) -> Self {
		Self {
			op: match elthen {
				Some(node) => {
					NodeType::IfThen(Box::new(cond), Box::new(then), Some(Box::new(node)))
				}
				None => {
					NodeType::IfThen(Box::new(cond), Box::new(then), None)
				}
			}
		}
	}
	pub fn new_call(ctype: Type, ident: String, args: Vec<Node>) -> Self {
		Self {
			op: NodeType::Call(ctype, ident, args)
		}
	}
	pub fn new_func(ctype: Type, ident: String, args: Vec<Node>, body: Node, stacksize: usize) -> Self {
		Self {
			op: NodeType::Func(ctype, ident, args, Box::new(body), stacksize)
		}
	}
	pub fn new_and(lhs: Node, rhs: Node) -> Self {
		Self {
			op: NodeType::LogAnd(Box::new(lhs), Box::new(rhs))
		}
	}
	pub fn new_or(lhs: Node, rhs: Node) -> Self {
		Self {
			op: NodeType::LogOr(Box::new(lhs), Box::new(rhs))
		}
	}
	pub fn new_for(init: Node, cond: Node, inc: Node, body: Node) -> Self {
		Self {
			op: NodeType::For(Box::new(init), Box::new(cond), Box::new(inc), Box::new(body))
		}
	}
	pub fn new_vardef(name: String, var: Var, rhs: Option<Node>) -> Self {
		Self {
			op: match rhs {
				Some(node) => { NodeType::VarDef(name, var, Some(Box::new(node))) }
				_ => { NodeType::VarDef(name, var, None)}
			}
		}
	}
	pub fn new_deref(ctype: Type, lhs: Node) -> Self {
		Self {
			op: NodeType::Deref(ctype, Box::new(lhs))
		}
	}
	pub fn new_addr(ctype: Type, lhs: Node) -> Self {
		Self {
			op: NodeType::Addr(ctype, Box::new(lhs))
		}
	}
	pub fn new_sizeof(ctype: Type, val: usize, lhs: Node) -> Self {
		Self {
			op: NodeType::Sizeof(ctype, val, Box::new(lhs))
		}
	}
	pub fn new_eqeq(lhs: Node, rhs: Node) -> Self {
		Self {
			op: NodeType::EqEq(Box::new(lhs), Box::new(rhs))
		}
	}
	pub fn new_neq(lhs: Node, rhs: Node) -> Self {
		Self {
			op: NodeType::Ne(Box::new(lhs), Box::new(rhs))
		}
	}
	pub fn new_dowhile(body: Node, cond: Node) -> Self {
		Self {
			op: NodeType::DoWhile(Box::new(body), Box::new(cond))
		}
	}
	pub fn new_stmtexpr(ctype: Type, body: Node) -> Self {
		Self {
			op: NodeType::StmtExpr(ctype, Box::new(body))
		}
	}
	pub fn new_alignof(expr: Node) -> Self {
		Self {
			op: NodeType::Alignof(Box::new(expr))
		}
	}
	pub fn new_dot(ctype: Type, expr: Node, member: String) -> Self {
		Self {
			op: NodeType::Dot(ctype, Box::new(expr), member)
		}
	}
	pub fn new_not(expr: Node) -> Self {
		Self {
			op: NodeType::Not(Box::new(expr))
		}
	}
	pub fn new_ternary(ctype: Type, cond: Node, then: Node, els: Node) -> Self {
		Self {
			op: NodeType::Ternary(ctype, Box::new(cond), Box::new(then), Box::new(els))
		}
	}
	pub fn new_tuple(ctype: Type, lhs: Node, rhs: Node) -> Self {
		Self {
			op: NodeType::TupleExpr(ctype, Box::new(lhs), Box::new(rhs))
		}
	}
	pub fn new_neg(expr: Node) -> Self {
		Self {
			op: NodeType::Neg(Box::new(expr))
		}
	}
	pub fn new_incdec(ctype: Type, selector: i32, expr: Node) -> Self {
		Self {
			op: NodeType::IncDec(ctype, selector, Box::new(expr))
		}
	}
	pub fn new_decl(ctype: Type, ident: String, args: Vec<Node>) -> Self {
		Self {
			op: NodeType::Decl(ctype, ident, args)
		}
	}
	pub fn new_var(var: Var) -> Self {
		Self {
			op: NodeType::Var(var)
		}
	}
	pub fn new_break() -> Self {
		Self {
			op: NodeType::Break
		}
	}
	pub fn new_null() -> Self {
		Self {
			op: NodeType::NULL
		}
	}
}

#[derive(Debug, Clone, PartialEq)]
pub struct Var {
	pub ctype: Type,
	pub offset: usize,
	pub is_local: bool,
	pub labelname: Option<String>,
	pub strname: Option<String>,
	pub is_extern: bool,
}

impl Var {
	pub fn new(ctype: Type, offset: usize, is_local: bool, labelname: Option<String>, strname: Option<String>) -> Self {
		let is_extern = ctype.is_extern;
		Self {
			ctype,
			offset,
			is_local,
			labelname,
			strname,
			is_extern,
		}
	}
}

#[derive(Debug, Clone)]
pub struct Env {
	tags: HashMap<String, Type>,
	typedefs: HashMap<String, Type>,
	vars: HashMap<String, Var>,
	next: Option<Box<Env>>,
}

impl Env {
	fn new_env(env: Option<Env>) -> Self {
		Self {
			tags: HashMap::new(),
			typedefs: HashMap::new(),
			vars: HashMap::new(),
			next: match env {
				Some(_env) => Some(Box::new(_env)),
				None => None,
			},
		}
	}
	fn env_inc() {
		let env = std::mem::replace(&mut *ENV.lock().unwrap(), Env::new_env(None));
		*ENV.lock().unwrap() = Env::new_env(Some(env));
	}
	fn env_dec() {
		let env = std::mem::replace(&mut *ENV.lock().unwrap(), Env::new_env(None));
		*ENV.lock().unwrap() = *env.next.unwrap();
	}
	fn add_var(ident: String, mut var: Var) -> usize {
		let mut offset = 1000000;
		if var.is_local {
			let stacksize = *STACKSIZE.lock().unwrap();
			*STACKSIZE.lock().unwrap() = roundup(stacksize, var.ctype.align);
			*STACKSIZE.lock().unwrap() += var.ctype.size;
			offset = *STACKSIZE.lock().unwrap();
			var.offset = offset;
		}
		ENV.lock().unwrap().vars.insert(ident, var);
		return offset;
	}
	fn add_typedef(ident: String, ctype: Type) {
		ENV.lock().unwrap().typedefs.insert(ident, ctype);
	}
	fn add_tags(tag: String, ctype: Type) {
		ENV.lock().unwrap().tags.insert(tag, ctype);
	}
}

pub fn roundup(x: usize, align: usize) -> usize {
	return (x + align - 1) & !(align - 1);
}

pub fn decl_specifiers(tokenset: &mut TokenSet) -> Type {
	if tokenset.consume_ty(TokenIdent) {
		tokenset.pos -= 1;
		let name = ident(tokenset);
		return env_find!(name, typedefs, NULL_TY.clone());
	}
	if tokenset.consume_ty(TokenInt){
		return INT_TY.clone();
	}
	if tokenset.consume_ty(TokenChar){
		return CHAR_TY.clone();
	}
	if tokenset.consume_ty(TokenStruct){
		
		let mut members = vec![];
		let mut tag = String::new();
		// tag
		if tokenset.consume_ty(TokenIdent) {
			tokenset.pos -= 1;
			tag = ident(tokenset);
		}
		
		// struct member
		if tokenset.consume_ty(TokenRightCurlyBrace) {
			while !tokenset.consume_ty(TokenLeftCurlyBrace) {
				members.push(declaration(tokenset, false));
			}
		}
		match (members.is_empty(), tag.is_empty()) {
			(true, true) => {
				// error("bat struct definition.");
				// for debug.
				panic!("bat struct definition.");
			}
			(true, false) => {
				return env_find!(tag.clone(), tags, NULL_TY.clone());
			}
			(false, c) => {
				let struct_type = new_struct(tag.clone(), members);
				if !c {
					Env::add_tags(tag, struct_type.clone());
				}
				return struct_type;
			}
		}
	}
	if tokenset.consume_ty(TokenTypeof) {
		tokenset.assert_ty(TokenRightBrac);
		let expr = assign(tokenset);
		tokenset.assert_ty(TokenLeftBrac);
		return get_type(&expr);
	}
	if tokenset.consume_ty(TokenVoid) {
		return VOID_TY.clone();
	}
	return NULL_TY.clone();
}

pub fn new_struct(tag: String, mut members: Vec<Node>) -> Type {
	let mut ty_align = 0;

	let mut off = 0;
	for i in 0..members.len() {
		if let NodeType::VarDef(_, var, ..) = &mut members[i].op {
			let ctype = &mut var.ctype;
			off = roundup(off, ctype.align);
			ctype.offset = off;
			off += ctype.size;
			ty_align = std::cmp::max(ty_align, ctype.align);
		}
	}
	let ty_size = roundup(off, ty_align);

	return Type::new(Ty::STRUCT(tag, members), None, None, ty_size ,ty_align , 0, 0, false);
}

fn assignment_op(tokenset: &mut TokenSet) -> Option<TokenType> {
	if tokenset.consume_ty(TokenAddEq) { return Some(TokenAdd); }
	else if tokenset.consume_ty(TokenSubEq) { return Some(TokenSub); }
	else if tokenset.consume_ty(TokenMulEq) { return Some(TokenStar); }
	else if tokenset.consume_ty(TokenDivEq) { return Some(TokenDiv); }
	else if tokenset.consume_ty(TokenModEq) { return Some(TokenMod); }
	else if tokenset.consume_ty(TokenShlEq) { return Some(TokenShl); }
	else if tokenset.consume_ty(TokenShrEq) { return Some(TokenShr); }
	else if tokenset.consume_ty(TokenAndEq) { return Some(TokenAmpersand); }
	else if tokenset.consume_ty(TokenOrEq) { return Some(TokenOr); }
	else if tokenset.consume_ty(TokenXorEq) { return Some(TokenXor); }
	else if tokenset.consume_ty(TokenEq) { return Some(TokenEq); }
	else { return None; }
}

fn string_literal(tokenset: &mut TokenSet) -> Node {
	// A string literal is converted to a reference to an anonymous
	// global variable of type char array.
	let strname = tokenset.getstring();
	let ctype = CHAR_TY.clone().ary_of(strname.len() + 1);
	tokenset.pos += 1;
	*STRLABEL.lock().unwrap() += 1;
	let labelname = format!(".L.str{}", *STRLABEL.lock().unwrap());
	let var = Var::new(ctype, 0, false, Some(labelname), Some(strname));
	GVARS.lock().unwrap().push(var.clone());
	return Node::new_var(var);
}

fn local_variable(tokenset: &mut TokenSet) -> Node {
	let token = &tokenset.tokens[tokenset.pos-1];
	let name = String::from(&PROGRAMS.lock().unwrap()[token.program_id][token.pos..token.end]);
	let var = env_find!(name.clone(), vars, NULL_VAR.clone());
	if let Ty::NULL = var.ctype.ty {
		panic!("{} is not defined.", name);
	}
	return Node::new_var(var);
}

fn function_call(tokenset: &mut TokenSet) -> Node {
	let token = &tokenset.tokens[tokenset.pos-2];
	let name = String::from(&PROGRAMS.lock().unwrap()[token.program_id][token.pos..token.end]);
	let var = env_find!(name.clone(), vars, NULL_VAR.clone());
	if let Ty::NULL = var.ctype.ty {
		eprintln!("Warning: \"{}\" function is not defined.", name);
	}
	// function call
	let mut args = vec![];
	//// arity = 0;
	if tokenset.consume_ty(TokenLeftBrac){
		return Node::new_call(NULL_TY.clone(), name, args);
	}
	//// arity > 0;
	let arg1 = assign(tokenset);
	args.push(arg1);
	while tokenset.consume_ty(TokenComma) {
		let argv = assign(tokenset);
		args.push(argv);
	}
	tokenset.assert_ty(TokenLeftBrac);
	return Node::new_call(var.ctype, name, args);
}

fn primary(tokenset: &mut TokenSet) -> Node {
	
	if tokenset.consume_ty(TokenRightBrac) {
		if tokenset.consume_ty(TokenRightCurlyBrace) {
			tokenset.pos -= 1;
			let body = Node::new_stmtexpr(NULL_TY.clone(), compound_stmt(tokenset, true));
			tokenset.assert_ty(TokenLeftBrac);
			return body;
		}
		let lhs = expr(tokenset);
		tokenset.assert_ty(TokenLeftBrac);
		return lhs;
	}
	if tokenset.consume_ty(TokenNum) {
		return Node::new_num(tokenset.tokens[tokenset.pos-1].val);
	}
	if tokenset.consume_ty(TokenIdent) {
		// variable
		if !tokenset.consume_ty(TokenRightBrac){
			return local_variable(tokenset);
		}
		return function_call(tokenset);
	}
	if tokenset.consume_ty(TokenString(String::new())) {
		return string_literal(tokenset);
	}
	// error(&format!("parse.rs: primary parse fail. and got {}", tokenset[*pos].input));
	// for debug.
	let token = &tokenset.tokens[tokenset.pos];
	panic!("parse.rs: primary parse fail. and got {}", &PROGRAMS.lock().unwrap()[token.program_id][token.pos..]);
}

fn postfix(tokenset: &mut TokenSet) -> Node {
	
	let mut lhs = primary(tokenset);

	loop {
		if tokenset.consume_ty(TokenInc) {
			lhs = Node::new_incdec(NULL_TY.clone(), 1, lhs);
		}
		if tokenset.consume_ty(TokenDec) {
			lhs = Node::new_incdec(NULL_TY.clone(), 2, lhs);
		}
		// struct member
		if tokenset.consume_ty(TokenDot) {
			let name = ident(tokenset);
			lhs = Node::new_dot(NULL_TY.clone(), lhs, name);
		// struct member arrow
		} else if tokenset.consume_ty(TokenArrow) {
			let name = ident(tokenset);
			let expr = Node::new_deref(INT_TY.clone(), lhs);
			lhs = Node::new_dot(NULL_TY.clone(), expr, name);
		// array
		} else if tokenset.consume_ty(TokenRightmiddleBrace)  {
			let id = assign(tokenset);
			let lhs2 = Node::new_bit(INT_TY.clone(), TokenAdd, lhs, id);
			lhs = Node::new_deref(INT_TY.clone(), lhs2);
			tokenset.assert_ty(TokenLeftmiddleBrace);
		} else {
			return lhs;
		}
	}
}

fn unary(tokenset: &mut TokenSet) -> Node {
	
	if tokenset.consume_ty(TokenInc) {
		let lhs = unary(tokenset);
		let rhs = Node::new_bit(NULL_TY.clone(), TokenAdd, lhs.clone(), Node::new_num(1));
		return Node::new_eq(NULL_TY.clone(), lhs, rhs);
	}
	if tokenset.consume_ty(TokenDec) {
		let lhs = unary(tokenset);
		let rhs = Node::new_bit(NULL_TY.clone(), TokenSub, lhs.clone(), Node::new_num(1));
		return Node::new_eq(NULL_TY.clone(), lhs, rhs);
	}
	if tokenset.consume_ty(TokenSub) {
		return Node::new_neg(unary(tokenset));
	}
	if tokenset.consume_ty(TokenStar) {
		return Node::new_deref(INT_TY.clone(), unary(tokenset));
	}
	if tokenset.consume_ty(TokenAmpersand) {
		return Node::new_addr(INT_TY.clone(), unary(tokenset));
	}
	if tokenset.consume_ty(TokenSizeof) {
		return Node::new_sizeof(INT_TY.clone(), 0, unary(tokenset));
	}
	if tokenset.consume_ty(TokenAlignof) {
		return Node::new_alignof(unary(tokenset));
	}
	if tokenset.consume_ty(TokenNot) {
		return Node::new_not(unary(tokenset));
	}
	if tokenset.consume_ty(TokenTilde) {
		return Node::new_bit(NULL_TY.clone(), TokenTilde, unary(tokenset), Node::new_num(1));
	}
	return postfix(tokenset);
}

fn mul(tokenset: &mut TokenSet) -> Node {
	let mut lhs = unary(tokenset);
	
	loop {
		if tokenset.consume_ty(TokenStar) {
			lhs = Node::new_bit(NULL_TY.clone(), TokenStar, lhs, unary(tokenset));
		} else if tokenset.consume_ty(TokenDiv) {
			lhs = Node::new_bit(NULL_TY.clone(), TokenDiv, lhs, unary(tokenset));
		} else if tokenset.consume_ty(TokenMod) {
			lhs = Node::new_bit(NULL_TY.clone(), TokenMod, lhs, unary(tokenset));
		} else {
			return lhs;
		}
	}

}

fn add(tokenset: &mut TokenSet) -> Node {
	let mut lhs = mul(tokenset);
	
	loop {
		if !tokenset.consume_ty(TokenAdd) && !tokenset.consume_ty(TokenSub) {
			return lhs;
		}
		let ty = tokenset.tokens[tokenset.pos-1].ty.clone();
		let rhs = mul(tokenset);
		lhs = Node::new_bit(NULL_TY.clone(), ty, lhs, rhs);
	}
	
}

fn shift(tokenset: &mut TokenSet) -> Node {
	let mut lhs = add(tokenset);

	loop {
		if tokenset.consume_ty(TokenShl) {
			lhs = Node::new_bit(NULL_TY.clone(), TokenShl, lhs, add(tokenset));
		} else if tokenset.consume_ty(TokenShr) {
			lhs = Node::new_bit(NULL_TY.clone(), TokenShr, lhs, add(tokenset));
		} else {
			return lhs;
		}
	}

}

fn relational(tokenset: &mut TokenSet) -> Node {
	let mut lhs = shift(tokenset);
	
	loop {
		if tokenset.consume_ty(TokenLt) {
			lhs = Node::new_bit(INT_TY.clone(), TokenLt, lhs, shift(tokenset));
		} else if tokenset.consume_ty(TokenRt) {
			lhs = Node::new_bit(INT_TY.clone(), TokenLt, shift(tokenset), lhs);
		} else if tokenset.consume_ty(TokenLe) {
			lhs = Node::new_bit(INT_TY.clone(), TokenLe, lhs, shift(tokenset));
		} else if tokenset.consume_ty(TokenGe) {
			lhs = Node::new_bit(INT_TY.clone(), TokenLe, shift(tokenset), lhs);
		} else {
			return lhs;
		}
	}
}

fn equarity(tokenset: &mut TokenSet) -> Node {
	let mut lhs = relational(tokenset);
	
	loop {
		if tokenset.consume_ty(TokenEqEq) {
			lhs = Node::new_eqeq(lhs, relational(tokenset));
		} else if tokenset.consume_ty(TokenNe) {
			lhs = Node::new_neq(lhs, relational(tokenset));
		} else {
			return lhs;
		}
	}
}

fn bitand(tokenset: &mut TokenSet) -> Node {
	let mut lhs = equarity(tokenset);

	while tokenset.consume_ty(TokenAmpersand) {
		lhs = Node::new_bit(INT_TY.clone(), TokenAmpersand, lhs, equarity(tokenset));
	}
	return lhs;
}

fn bitxor(tokenset: &mut TokenSet) -> Node {
	let mut lhs = bitand(tokenset);

	while tokenset.consume_ty(TokenXor) {
		lhs = Node::new_bit(INT_TY.clone(), TokenXor, lhs, bitand(tokenset));
	}
	return lhs;
}

fn bitor(tokenset: &mut TokenSet) -> Node {
	let mut lhs = bitxor(tokenset);

	while tokenset.consume_ty(TokenOr) {
		lhs = Node::new_bit(INT_TY.clone(), TokenOr, lhs, bitxor(tokenset));
	}
	return lhs;
}

fn logand(tokenset: &mut TokenSet) -> Node {
	let mut lhs = bitor(tokenset);

	while tokenset.consume_ty(TokenLogAnd) {
		lhs = Node::new_and(lhs, bitor(tokenset));
	}
	return lhs;
}

fn logor(tokenset: &mut TokenSet) -> Node {
	let mut lhs = logand(tokenset);

	while tokenset.consume_ty(TokenLogOr) {
		lhs = Node::new_or(lhs, logand(tokenset));
	}
	return lhs;
}

fn conditional(tokenset: &mut TokenSet) -> Node {
	let cond = logor(tokenset);
	if tokenset.consume_ty(TokenQuestion) {
		let then = expr(tokenset);
		tokenset.assert_ty(TokenColon);
		let els = conditional(tokenset);
		return Node::new_ternary(NULL_TY.clone(), cond, then, els);
	}
	return cond;
}

fn assign(tokenset: &mut TokenSet) -> Node {
	let mut lhs = conditional(tokenset);

	if let Some(op) = assignment_op(tokenset) {
		let rhs = assign(tokenset);
		match op {
			TokenEq => {
				lhs = Node::new_eq(NULL_TY.clone(), lhs, rhs);
			}
			_ => {
				let llhs = Node::new_bit(NULL_TY.clone(), op, lhs.clone(), rhs);
				lhs = Node::new_eq(NULL_TY.clone(), lhs, llhs);
			}
		}
	}
	return lhs;
}

fn expr(tokenset: &mut TokenSet) -> Node {
	let lhs = assign(tokenset);
	if tokenset.consume_ty(TokenComma) {
		return Node::new_tuple(NULL_TY.clone(), lhs, expr(tokenset));
	}
	return lhs;
}

fn declarator(tokenset: &mut TokenSet, mut ty: Type) -> Node {
	
	while tokenset.consume_ty(TokenStar) {
		ty = ty.ptr_to();
	}
	
	return direct_decl(tokenset, ty);

}

fn read_array(tokenset: &mut TokenSet, ty: Type) -> Type {
	let mut ary_size = vec![];
	let mut ty = ty.clone();

	while tokenset.consume_ty(TokenRightmiddleBrace) {
		if tokenset.consume_ty(TokenLeftmiddleBrace) {
			ary_size.push(0);
			continue;
		}
		let len = expr(tokenset);
		if let NodeType::Num(val) = &len.op {
			ary_size.push(*val as usize);
			tokenset.assert_ty(TokenLeftmiddleBrace);
			continue;
		}
		// error(&format!("array declaration is invalid at {}.", tokenset[*pos].input));
		// for debug.
		let token = &tokenset.tokens[tokenset.pos];
		panic!("array declaration is invalid at {}.", &PROGRAMS.lock().unwrap()[token.program_id][token.pos..]);
	}

	if ary_size.len() > 0 {
		for i in (0..ary_size.len()).rev() {
			ty = ty.ary_of(ary_size[i]);
		}
	}

	return ty;
}

fn ident(tokenset: &mut TokenSet) -> String {
	// panic!("{}, {}, {}", tokenset[*pos].program_id, tokenset[*pos].pos, tokenset[*pos].val);
	let token = tokenset.tokens[tokenset.pos].clone();
	let name = String::from(&PROGRAMS.lock().unwrap()[token.program_id][token.pos..token.end]);
	if !tokenset.consume_ty(TokenIdent) {
		// error(&format!("should be identifier at {}", &tokenset[*pos].input[*pos..]));
		// for debug.
		panic!("should be identifier at {}", &PROGRAMS.lock().unwrap()[token.program_id][token.pos..]);
	}
	return name;
}

fn decl_init(tokenset: &mut TokenSet, node: &mut Node) {
	if let NodeType::VarDef(.., ref mut init) = node.op {
		if tokenset.consume_ty(TokenEq) {
			let rhs = assign(tokenset);
			std::mem::replace(init, Some(Box::new(rhs)));
		}
	}
	return;
}

fn new_ptr_to_replace_type(ctype: &Type, true_ty: Type) -> Type {
	match ctype.ty {
		Ty::NULL => {
			return true_ty;
		}
		_ => {
			return Type::new(
				ctype.ty.clone(),
				Some(Box::new(new_ptr_to_replace_type(ctype.ptr_to.as_ref().unwrap().as_ref(), true_ty))),
				ctype.ary_to.clone(),
				ctype.size,
				ctype.align,
				ctype.offset,
				ctype.len,
				ctype.is_extern,
			)
		}
	}
}

fn direct_decl(tokenset: &mut TokenSet, ty: Type) -> Node {

	let mut ident_node;
	
	if tokenset.consume_ty(TokenIdent) {
		tokenset.pos -= 1;
		let name = ident(tokenset);
		let mut var = NULL_VAR.clone();
		var.ctype = read_array(tokenset, ty);
		ident_node = Node::new_vardef(name, var, None);
	} else if tokenset.consume_ty(TokenRightBrac) {
		ident_node = declarator(tokenset, NULL_TY.clone());
		tokenset.assert_ty(TokenLeftBrac);
		
		let true_ty = read_array(tokenset, ty);
		let ident_node_true_ty = new_ptr_to_replace_type(&ident_node.nodesctype(None), true_ty);

		if let NodeType::VarDef(name, mut var, init) = ident_node.op {
			var.ctype = ident_node_true_ty;
			ident_node = Node {
				op: NodeType::VarDef(name, var, init)
			};
		} else {
			panic!("direct_decl fun error.");
		}
	} else {
		// for debug
		let token = &tokenset.tokens[tokenset.pos];
		panic!("bad direct declarator at {}", &PROGRAMS.lock().unwrap()[token.program_id][token.pos..]);
		// error(&format!("bad direct declarator at {}", &tokenset[*pos].input[..]));
	}
	decl_init(tokenset, &mut ident_node);
	return ident_node;
}

fn declaration(tokenset: &mut TokenSet, newvar: bool) -> Node {

	// declaration type
	let ty = decl_specifiers(tokenset);

	let ident_node = declarator(tokenset, ty);
	tokenset.assert_ty(TokenSemi);
	// panic!("{:#?}", ident_node);
	
	if newvar {
		if let NodeType::VarDef(name, mut var, init) = ident_node.op {
			// add new var to ENV
			var.offset = Env::add_var(name.clone(), var.clone());
			return Node {
				op: NodeType::VarDef(name, var, init)
			};
		}
	}

	return ident_node;
}

fn expr_stmt(tokenset: &mut TokenSet) -> Node {
	let lhs = expr(tokenset);
	tokenset.consume_ty(TokenSemi);
	return Node::new_expr(lhs);
}

pub fn stmt(tokenset: &mut TokenSet) -> Node {
	
	match tokenset.tokens[tokenset.pos].ty {
		TokenRet => {
			tokenset.pos += 1;
			let lhs = expr(tokenset);
			tokenset.assert_ty(TokenSemi);
			return Node::new_ret(lhs);
		},
		TokenIf => {
			tokenset.pos += 1;
			tokenset.assert_ty(TokenRightBrac);
			let cond = expr(tokenset);
			tokenset.assert_ty(TokenLeftBrac);
			let then = stmt(tokenset);
			if tokenset.consume_ty(TokenElse) {
				let elthen = stmt(tokenset);
				return Node::new_if(cond, then, Some(elthen));
			} else {
				return Node::new_if(cond, then, None);
			}
		}
		TokenFor => {
			tokenset.pos += 1;
			tokenset.assert_ty(TokenRightBrac);
			Env::env_inc();
			let init;
			if tokenset.is_typename() {
				tokenset.pos -= 1;
				init = declaration(tokenset, true);
			} else if tokenset.consume_ty(TokenSemi) {
				init = Node::new_null();
			} else {
				init = expr_stmt(tokenset);
			}
			let mut cond = Node::new_null();
			if !tokenset.consume_ty(TokenSemi) {
				cond = expr(tokenset);
				tokenset.assert_ty(TokenSemi);
			}
			let mut inc = Node::new_null();
			if !tokenset.consume_ty(TokenLeftBrac) {
				inc = stmt(tokenset);
				tokenset.assert_ty(TokenLeftBrac);
			} 
			let body = stmt(tokenset);
			Env::env_dec();
			return Node::new_for(init, cond, inc, body);
		}
		TokenWhile => {
			tokenset.pos += 1;
			let init = Node::new_null();
			let inc = Node::new_null();
			tokenset.assert_ty(TokenRightBrac);
			let cond = expr(tokenset);
			tokenset.assert_ty(TokenLeftBrac);
			let body = stmt(tokenset);
			return Node::new_for(init, cond, inc, body);
		}
		TokenDo => {
			tokenset.pos += 1;
			let body = stmt(tokenset);
			tokenset.assert_ty(TokenWhile);
			tokenset.assert_ty(TokenRightBrac);
			let cond = expr(tokenset);
			tokenset.assert_ty(TokenLeftBrac);
			tokenset.assert_ty(TokenSemi);
			return Node::new_dowhile(body, cond);
		}
		TokenRightCurlyBrace => {
			tokenset.pos += 1;
			Env::env_inc();
			let mut compstmts = vec![];
			while !tokenset.consume_ty(TokenLeftCurlyBrace) {
				compstmts.push(stmt(tokenset));
			}
			Env::env_dec();
			return Node::new_stmt(compstmts);
		}
		TokenInt | TokenChar | TokenStruct | TokenTypeof => {
			return declaration(tokenset, true);
		}
		TokenSemi => {
			tokenset.pos += 1;
			return Node::new_null();
		}
		TokenTypedef => {
			tokenset.pos += 1;
			let lhs = declaration(tokenset, false);
			if let NodeType::VarDef(name, var, None) = lhs.op {
				Env::add_typedef(name, var.ctype);
				return Node::new_null();
			}
			panic!("typedef error.");
		}
		TokenBreak => {
			tokenset.pos += 1;
			return Node::new_break();
		}
		_ => {
			if tokenset.consume_ty(TokenIdent) {
				if tokenset.consume_ty(TokenIdent) {
					tokenset.pos -= 2;
					return declaration(tokenset, true);
				}
				tokenset.pos -= 1;
			}
			return expr_stmt(tokenset);
		}
	}
}

pub fn compound_stmt(tokenset: &mut TokenSet, newenv: bool) -> Node {
	
	let mut compstmts = vec![];
	tokenset.assert_ty(TokenRightCurlyBrace);
	if newenv {
		Env::env_inc();
	}
	loop {
		match tokenset.consume_ty(TokenLeftCurlyBrace) {
			true => { break; },
			false => { 
				let stmt = stmt(tokenset);
				compstmts.push(stmt);
			}
		}
	}
	Env::env_dec();
	return Node::new_stmt(compstmts);
}

pub fn param_declaration(tokenset: &mut TokenSet) -> Node {
	
	// type
	let ty = decl_specifiers(tokenset);
	let node = declarator(tokenset, ty);
	let ctype = node.nodesctype(None);
	if let NodeType::VarDef(name, mut var, init) = node.op {
		if let Ty::ARY = &ctype.ty {
			var.ctype = ctype.ary_to.unwrap().clone().ptr_to();
		}
		var.offset = Env::add_var(name.clone(), var.clone());
		return Node {
			op: NodeType::VarDef(name, var, init)
		};
	} else {
		panic!("{:?} should be NodeType::VarDef", node);
	}
}

pub fn toplevel(tokenset: &mut TokenSet) -> Node {
	
	let mut args = vec![];

	let is_extern = tokenset.consume_ty(TokenExtern);
	let is_typedef = tokenset.consume_ty(TokenTypedef);

	// Ctype
	let mut ctype = decl_specifiers(tokenset);
	ctype.is_extern = is_extern;
	
	while tokenset.consume_ty(TokenStar) {
		ctype = ctype.ptr_to();
	}
	
	// identifier
	let ident = ident(tokenset);
	
	// function
	if tokenset.consume_ty(TokenRightBrac){
		if is_typedef {
			// error(&format!("typedef {} has function definition.", name));
			// for debug.
			panic!("typedef {} has function definition.", ident);
		}
		*STACKSIZE.lock().unwrap() = 0;
		// add new function to Env
		let var = Var::new(ctype.clone(), 0, false, Some(ident.clone()), None);
		Env::add_var(ident.clone(), var);

		Env::env_inc();
		// argument
		if !tokenset.consume_ty(TokenLeftBrac) {
			loop {
				args.push(param_declaration(tokenset));
				if tokenset.consume_ty(TokenLeftBrac){ break; }
				tokenset.assert_ty(TokenComma);
			}
		}
		// function decl
		if tokenset.consume_ty(TokenSemi) {
			return Node::new_null();
		}
		// body
		let body = compound_stmt(tokenset, false);
		return Node::new_func(ctype, ident, args, body, *STACKSIZE.lock().unwrap());
	}

	ctype = read_array(tokenset, ctype);
	tokenset.assert_ty(TokenSemi);
	// typedef
	if is_typedef {
		Env::add_typedef(ident, ctype);
		return Node::new_null();
	}
	// global variable
	let var = Var::new(ctype.clone(), 0, false, Some(ident.clone()), None);
	Env::add_var(ident, var.clone());
	GVARS.lock().unwrap().push(var);
	return Node::new_null();
}

pub fn parse(tokenset: &mut TokenSet, program: &mut Program) {

	*ENV.lock().unwrap() = Env::new_env(None);

	loop {
		match tokenset.consume_ty(TokenEof) {
			true => { break; }
			false => { program.nodes.push(toplevel(tokenset)); }
		}
	}
	program.gvars = std::mem::replace(&mut GVARS.lock().unwrap(), vec![]);
}