use TokenType::*;
use super::parse::{Type, Ty};

lazy_static! {
	pub static ref INT_TY: Type = Type {
		ty: Ty::INT,
		ptr_of: None,
		ary_of: None,
		len: 0,
	};
	pub static ref CHAR_TY: Type = Type {
		ty: Ty::CHAR,
		ptr_of: None,
		ary_of: None,
		len: 0,
	};
}


pub static SIGNALS: [Signal; 20] = [
	Signal::new("&&", TokenLogAnd),
	Signal::new("||", TokenLogOr),
	Signal::new("==", TokenEqEq),
	Signal::new("!=", TokenNe),
	Signal::new("+", TokenAdd),
	Signal::new("-", TokenSub),
	Signal::new("*", TokenStar),
	Signal::new("/", TokenDiv),
	Signal::new(";", TokenSemi),
	Signal::new("=", TokenEq),
	Signal::new("(", TokenRightBrac),
	Signal::new(")", TokenLeftBrac),
	Signal::new(",", TokenComma),
	Signal::new("{", TokenRightCurlyBrace),
	Signal::new("}", TokenLeftCurlyBrace),
	Signal::new("<", TokenLt),
	Signal::new(">", TokenRt),
	Signal::new("[", TokenRightmiddleBrace),
	Signal::new("]", TokenLeftmiddleBrace),
	Signal::new("&", TokenAmpersand),
];

#[derive(Debug, PartialEq, Clone)]
pub enum TokenType {
	TokenNum,
	TokenAdd,
	TokenSub,
	TokenStar,
	TokenDiv,
	TokenRet,
	TokenSemi,
	TokenIdent,
	TokenEq,
	TokenRightBrac,
	TokenLeftBrac,
	TokenIf,
	TokenElse,
	TokenComma,
	TokenRightCurlyBrace,
	TokenLeftCurlyBrace,
	TokenLogAnd,
	TokenLogOr,
	TokenLt,
	TokenRt,
	TokenRightmiddleBrace,
	TokenLeftmiddleBrace,
	TokenAmpersand,
	TokenSizeof,
	TokenFor,
	TokenInt,
	TokenChar,
	TokenDoubleQuo,
	TokenString(String),
	TokenEqEq,
	TokenNe,
	TokenDo,
	TokenWhile,
	TokenExtern,
	TokenNoSignal,
	TokenEof,
}

impl From<String> for TokenType {
	fn from(s: String) -> Self {
		match &s[..] {
			"return" => { TokenRet }
			"if" => { TokenIf }
			"else" => { TokenElse }
			"for" => { TokenFor }
			"int" => { TokenInt }
			"sizeof" => { TokenSizeof }
			"char" => { TokenChar }
			"do" => { TokenDo }
			"while" => { TokenWhile }
			"extern" => { TokenExtern }
			_ => { TokenIdent }
		}
	}
}

#[derive(Debug)]
pub struct Token<'a> {
	pub ty: TokenType,
	pub val: i32,
	pub input: &'a str,
}

impl<'a> Token<'a> {
	pub fn new(ty: TokenType, val: i32, input: &'a str) -> Token<'a> {
		Token {
			ty: ty,
			val: val,
			input: input,
		}
	}
	pub fn consume(&self, c: &str, pos: &mut usize) -> bool {
		if self.input[..self.val as usize] == *c {
			*pos += 1;
			return true;
		}
		return false;
	}
	pub fn expect(&self, c: &str, pos: &mut usize) -> bool {
		if self.consume(c, pos) {
			return true;
		}
		panic!("expect fun error: {} is expected, but got {}", c, &self.input[..self.val as usize]);
	}
	pub fn assert_ty(&self, ty: TokenType, pos: &mut usize) {
		if !self.consume_ty(ty, pos) {
			panic!("assertion failed at: {}", &self.input[..self.val as usize])
		}
	}
	pub fn consume_ty(&self, ty: TokenType, pos: &mut usize) -> bool {
		if self.ty == ty { 
			*pos += 1; 
			return true; 
		}
		else { 
			match ty {
				TokenString(_) => { return true;}
				_ => { return false; }
			}
		}
	}
	pub fn is_typename(&self, pos: &mut usize) -> bool {
		if self.ty == TokenInt {
			*pos += 1;
			return true;
		}
		return false;
	}
	pub fn decl_type(&self, pos: &mut usize) -> Type {
		*pos += 1;
		if self.ty == TokenInt { return INT_TY.clone(); }
		else if self.ty == TokenChar { return CHAR_TY.clone(); }
		else { panic!("decralation type is invalid."); }
	}
	pub fn getstring(&self) -> String {
		match &self.ty {
			TokenString(sb) => { return sb.clone(); }
			_ => { panic!("getstring error."); }
		}
	}
}

pub struct Signal {
	pub name: &'static str,
	pub ty: TokenType
}

impl Signal {
	const fn new(name: &'static str, ty: TokenType) -> Self {
		Self {
			name,
			ty,
		}
	}
}

// return next number
fn strtol(p: &mut core::str::Chars, pos: &mut usize, c: char) -> i32 {

	let mut pp = p.clone();
	let mut num_str = String::from("");
	num_str.push(c);

	while let Some(c) = pp.next() {
		if c.is_ascii_digit() {
			num_str.push(c);
			p.next();
			*pos += 1;
			continue;
		}
		break;
	}
	num_str.parse::<i32>().unwrap()
}

fn read_string<'a> (p: &mut core::str::Chars, pos: &mut usize, input: &'a String) -> Token<'a> {

	let mut len = 0;
	let start = *pos;
	let mut sb = String::new();

	while let Some(c) = p.next() {
		if c == '"'	{
			*pos += 1;
			break;
		}
		if c != '\\' {
			sb.push(c);
			*pos += 1;
			len += 1;
			continue;
		}
		let c2 = p.next().unwrap();
		match c2 {
			'a' => { sb.push_str("\\a"); }
			'b' => { sb.push_str("\\b"); }
			'f' => { sb.push_str("\\f"); }
			'n' => { sb.push_str("\\n"); }
			'r' => { sb.push_str("\\r"); }
			'v' => { sb.push_str("\\v"); }
			't' => { sb.push_str("\\t"); }
			'\0' => { panic!("PREMATURE of input."); }
			_ => { sb.push(c); }
		}
		*pos += 2;
		len += 2;
	}
	return Token::new(TokenString(sb), len, &input[start..]);
}

pub fn tokenize(input: &String) -> Vec<Token> {
	
	let mut tokens: Vec<Token> = vec![];
	let mut pos = 0;
	let mut p = input.chars();

	'outer: while let Some(c) = p.next() {

		// space
		if c.is_whitespace() {
			pos += 1;
			continue;
		}

		// string literal
		if c == '"' {
			pos += 1;
			tokens.push(read_string(&mut p, &mut pos, &input));
			continue;
		}
		
		// signal
		for signal in &SIGNALS {
			let len = signal.name.len();
			if input.len() >= pos+len && *signal.name == input[pos..pos+len] {
				let token = Token::new(signal.ty.clone(), len as i32, &input[pos..]);
				tokens.push(token);
				pos += len;
				for _ in 0..len-1 { p.next(); }
				continue 'outer;
			}
		}

		// ident
		if c.is_alphabetic() || c == '_' {
			let mut ident = String::new();
			ident.push(c);
			let mut len = 1;
			let mut pp = p.clone();
			let possub = pos;
			loop {
				if let Some(cc) = pp.next() {
					if !cc.is_alphabetic() && !cc.is_ascii_digit() && cc != '_'{
						break;
					}
					p.next();
					ident.push(cc);
					len += 1;
					pos += 1;
				}
			}
			let token = Token::new(TokenType::from(ident), len, &input[possub..]);
			tokens.push(token);
			pos += 1;
			continue;
		}
		
		// number
		if c.is_digit(10) {
			let possub = pos;
			let num = strtol(&mut p, &mut pos, c);
			let token = Token::new(TokenNum, num, &input[possub..]);
			tokens.push(token);
			pos += 1;
			continue;
		}
		panic!("cannot scan at {}", &input[pos..]);
	}

	// guard
	let token = Token::new(TokenEof, 0, &input[pos..]);
	tokens.push(token);
	
	tokens
}