use std::process;
use TokenType::*;

#[derive(Debug, PartialEq, Clone)]
pub enum TokenType {
	TokenNum,
	TokenPlus,
	TokenMinus,
	TokenEof,
}

#[derive(Debug)]
pub struct Token {
	pub ty: TokenType,
	pub val: i32,
	pub input: usize,
}

impl Token {
	pub fn new(ty: TokenType, val: i32, input: usize) -> Token {
		Token {
			ty: ty,
			val: val,
			input: input,
		}
	}
}

// return next_number and position
fn next_number(p: &Vec<char>, mut pos: usize) -> (i32, usize) {
	let mut num = String::from("");
	for i in pos..p.len() {
		if p[i].is_digit(10) {
			num.push(p[i]);
			pos += 1;
		} else {
			break;
		}
	}
	(num.parse::<i32>().unwrap(), pos)
}

// return TokenType of given character
fn signal2token (p: char) -> TokenType {
	if p == '+' { TokenPlus }
	else if p == '-' { TokenMinus }
	else { panic!("signal2token error!"); }
}

pub fn tokenize(p: &Vec<char>, tokens: &mut Vec<Token>, mut pos: usize) {
	
	while pos < p.len() {

		if p[pos].is_whitespace() {
			pos += 1;
			continue;
		}
		
		if p[pos] == '+' || p[pos] == '-' {
			let token = Token::new(signal2token(p[pos]), 0, pos);
			tokens.push(token);
			pos += 1;
			continue;
		}
		
		if p[pos].is_digit(10) {
			let next = next_number(p, pos);
			let token = Token::new(TokenNum, next.0, pos);
			pos = next.1;
			tokens.push(token);
			continue;
		}

		eprintln!("cannot tokenize.");
		process::exit(1);
	}

	let token = Token::new(TokenEof, 0, 0);
	tokens.push(token);

}