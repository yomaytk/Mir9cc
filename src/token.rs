use TokenType::*;

pub static SIGNALS: [Signal; 13] = [
	Signal::new("&&", TokenLogAnd),
	Signal::new("||", TokenLogOr),
	Signal::new("+", TokenAdd),
	Signal::new("-", TokenSub),
	Signal::new("*", TokenMul),
	Signal::new("/", TokenDiv),
	Signal::new(";", TokenSemi),
	Signal::new("=", TokenEq),
	Signal::new("(", TokenRightBrac),
	Signal::new(")", TokenLeftBrac),
	Signal::new(",", TokenComma),
	Signal::new("{", TokenRightCurlyBrace),
	Signal::new("}", TokenLeftCurlyBrace),
];

#[derive(Debug, PartialEq, Clone)]
pub enum TokenType {
	TokenNum,
	TokenAdd,
	TokenSub,
	TokenMul,
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
	TokenNoSignal,
	TokenEof,
}

impl From<String> for TokenType {
	fn from(s: String) -> Self {
		match &s[..] {
			"return" => { TokenRet }
			"if" => { TokenIf }
			"else" => { TokenElse }
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
		assert!(self.consume_ty(ty, pos));
	}
	pub fn consume_ty(&self, ty: TokenType, pos: &mut usize) -> bool {
		if self.ty == ty {
			*pos += 1;
			return true;
		}
		return false;
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

pub fn scan(input: &String) -> Vec<Token> {
	
	let mut tokens: Vec<Token> = vec![];
	let mut pos = 0;
	let mut p = input.chars();

	'outer: while let Some(c) = p.next() {

		// space
		if c.is_whitespace() {
			pos += 1;
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