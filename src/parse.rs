use std::process;
use super::token::*;
use super::token::TokenType::*;

#[allow(dead_code)]
#[derive(Debug)]
pub enum NodeType {
	Num,
	BinaryTree(TokenType, Option<Box<Node>>, Option<Box<Node>>),
	Ret(Box<Node>),
	Expr(Box<Node>),
	CompStmt(Vec<Node>),
}

#[allow(dead_code)]
impl NodeType {
	fn bit_new(tk_ty: TokenType) -> Self {
		NodeType::BinaryTree(tk_ty, None, None)
	}

	fn bit_init(tk_ty: TokenType, lhs: Node, rhs: Node) -> Self {
		NodeType::BinaryTree(tk_ty, Some(Box::new(lhs)), Some(Box::new(rhs)))
	}

	fn ret_init(lhs: Node) -> Self {
		NodeType::Ret(Box::new(lhs))
	}

	fn expr_init(lhs: Node) -> Self {
		NodeType::Expr(Box::new(lhs))
	}

	fn stmt_init(compstmts: Vec<Node>) -> Self {
		NodeType::CompStmt(compstmts)
	}
}

#[derive(Debug)]
pub struct Node {
	pub val: i32,
	pub ty: NodeType,
}

#[allow(dead_code)]
impl Node {
	pub fn new(tk_ty: TokenType) -> Self {
		Self {
			val: -1,
			ty: NodeType::bit_new(tk_ty),
		}
	}

	pub fn bit_init(tk_ty: TokenType, lhs: Node, rhs: Node) -> Self {
		Self {
			val: -1,
			ty: NodeType::bit_init(tk_ty, lhs, rhs),
		}
	}
	
	pub fn new_node_num(val: i32) -> Self {
		Self {
			val: val,
			ty: NodeType::Num,
		}
	}

	pub fn new_ret(lhs: Node) -> Self {
		Self {
			val: -1,
			ty: NodeType::ret_init(lhs)
		}
	}

	pub fn new_expr(lhs: Node) -> Self {
		Self {
			val: -1,
			ty: NodeType::expr_init(lhs)
		}
	}

	pub fn new_stmt(compstmts: Vec<Node>) -> Self {
		Self {
			val: -1,
			ty: NodeType::stmt_init(compstmts)
		}
	}
}

fn number(tokens: &Vec<Token>, pos: usize) -> (Node, usize) {
	if tokens[pos].ty == TokenNum {
		return (Node::new_node_num(tokens[pos].val), pos+1);
	}
	eprintln!("parse.rs: number expected, but got {}", tokens[pos].input);
	process::exit(1);
}

fn mul(tokens: &Vec<Token>, pos: usize) -> (Node, usize) {
	let (mut lhs, mut pos) = number(tokens, pos);
	
	loop {
		if tokens[pos].ty != TokenMul && tokens[pos].ty != TokenDiv {
			return (lhs, pos);
		}
		let (rhs, new_pos) = number(tokens, pos+1);
		lhs = Node::bit_init(tokens[pos].ty.clone(), lhs, rhs);
		pos = new_pos;
	}

}

pub fn expr(tokens: &Vec<Token>, pos: usize) -> (Node, usize) {
	let (mut lhs, mut pos) = mul(tokens, pos);

	loop {
		if tokens[pos].ty != TokenPlus && tokens[pos].ty != TokenMinus {
			return (lhs, pos);
		}
		let (rhs, new_pos) = mul(tokens, pos+1);
		lhs = Node::bit_init(tokens[pos].ty.clone(), lhs, rhs);
		pos = new_pos;
	}
	
}

pub fn stmt(tokens: &Vec<Token>, pos: usize) -> Node {
	
	let mut pos = pos;
	let mut compstmts = vec![];

	loop {
		if tokens[pos].ty == TokenEof {
			break;
		}
		match tokens[pos].ty.clone() {
			TokenRet => {
				let (lhs, new_pos) = expr(tokens, pos+1);
				compstmts.push(Node::new_ret(lhs));
				pos = new_pos;
			},
			_ => {
				let (lhs, new_pos) = expr(tokens, pos);
				compstmts.push(Node::new_expr(lhs));
				pos = new_pos;
			}
		}
		assert_eq!(TokenSemi, tokens[pos].ty.clone());
		pos += 1;
	}

	let compstmt = Node::new_stmt(compstmts);
	compstmt
}

pub fn parse(tokens: &Vec<Token>, pos: usize) -> Node {
	
	let compstmt = stmt(tokens, pos);

	compstmt
}