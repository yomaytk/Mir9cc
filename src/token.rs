use TokenType::*;
use std::collections::HashMap;
use std::sync::Mutex;
use super::lib::*;
use super::preprocess::*;

// Atomic unit in the grammar is called "token".
// For example, `123`, `"abc"` and `while` are tokens.
// The tokenizer splits an input string into tokens.
// Spaces and comments are removed by the tokenizer.

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
	pub static ref PROGRAMS: Mutex<Vec<String>> = Mutex::new(Vec::new());
	pub static ref ESCAPED: Mutex<HashMap<char, char>> = Mutex::new(hash![
		// ('a', "\\a"), ('b', "\\b"), ('f', "\\f"),
		('n', '\n'), ('r', '\r'), // ('v', "\\v"),
		('t', '\t') // ('e', '\033'), ('E', '\033')
	]);
	pub static ref LINE: Mutex<usize> = Mutex::new(1);
}

pub static SIGNALS: &[Signal] = &[
	Signal::new("<<=", TokenShlEq),
	Signal::new(">>=", TokenShrEq),
	Signal::new("&&", TokenLogAnd),
	Signal::new("||", TokenLogOr),
	Signal::new("==", TokenEqual),
	Signal::new("!=", TokenNe),
	Signal::new("->", TokenArrow),
	Signal::new("<=", TokenLe),
	Signal::new(">=", TokenGe),
	Signal::new("<<", TokenShl),
	Signal::new(">>", TokenShr),
	Signal::new("++", TokenInc),
	Signal::new("--", TokenDec),
	Signal::new("+=", TokenAddEq),
	Signal::new("-=", TokenSubEq),
	Signal::new("*=", TokenMulEq),
	Signal::new("/=", TokenDivEq),
	Signal::new("%=", TokenModEq),
	Signal::new("&=", TokenAndEq),
	Signal::new("|=", TokenOrEq),
	Signal::new("^=", TokenXorEq),
	Signal::new("+", TokenAdd),
	Signal::new("-", TokenSub),
	Signal::new("*", TokenStar),
	Signal::new("/", TokenDiv),
	Signal::new(";", TokenSemi),
	Signal::new("=", TokenAssign),
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
	Signal::new(".", TokenDot),
	Signal::new("!", TokenNot),
	Signal::new(":", TokenColon),
	Signal::new("?", TokenQuestion),
	Signal::new("|", TokenOr),
	Signal::new("^", TokenXor),
	Signal::new("%", TokenMod),
	Signal::new("~", TokenTilde),
	Signal::new("#", TokenSharp),
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
	TokenAssign,
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
	TokenEqual,
	TokenNe,
	TokenDo,
	TokenWhile,
	TokenExtern,
	TokenAlignof,
	TokenStruct,
	TokenDot,
	TokenArrow,
	TokenTypedef,
	TokenVoid,
	TokenNot,
	TokenQuestion,
	TokenColon,
	TokenOr,
	TokenXor,
	TokenLe,
	TokenGe,
	TokenShl,
	TokenShr,
	TokenMod,
	TokenInc,
	TokenDec,
	TokenBreak,
	TokenAddEq,
	TokenSubEq,
	TokenMulEq,
	TokenDivEq,
	TokenModEq,
	TokenShlEq,
	TokenShrEq,
	TokenAndEq,
	TokenOrEq,
	TokenXorEq,
	TokenTilde,
	TokenSharp,
	TokenInclude,
	TokenDefine,
	TokenNewLine,
	TokenParam(bool),	// TokenParam(stringize)
	TokenTypeof,
	TokenContinue,
	TokenBool,
	TokenSwitch,
	TokenCase,
	TokenEnum,
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
			"_Alignof" => { TokenAlignof }
			"struct" => { TokenStruct }
			"typedef" => { TokenTypedef }
			"void" => { TokenVoid }
			"break" => { TokenBreak }
			"include" => { TokenInclude }
			"define" => { TokenDefine }
			"typeof" => { TokenTypeof }
			"continue" => { TokenContinue }
			"_Bool" => { TokenBool }
			"switch" => { TokenSwitch }
			"case" => { TokenCase }
			"enum" => { TokenEnum }
			_ => { TokenIdent }
		}
	}
}

#[derive(Debug, Clone)]
pub struct Token {
	pub ty: TokenType,
	pub val: i32,
	pub program_id: usize,
	pub pos: usize,
	pub end: usize,
	pub line: usize,
}

impl Token {
	pub fn new(ty: TokenType, val: i32, program_id: usize, pos: usize, end: usize, line: usize) -> Token {
		Token {
			ty,
			val,
			program_id,
			pos,
			end,
			line
		}
	}
	pub fn getstring(&self) -> String {
		match &self.ty {
			TokenString(sb) => { return sb.clone(); }
			_ => { panic!("{:?}", self); }
		}
	}
}

pub struct TokenSet {
	pub tokens: Vec<Token>,
	pub pos: usize
}

impl TokenSet {
	pub fn new(tokens: Vec<Token>) -> Self {
		Self {
			tokens,
			pos: 0
		}
	}
	pub fn assert_ty(&mut self, ty: TokenType) {
		let pos = self.pos;
		if !self.consume_ty(ty) {
			// error(&format!("assertion failed at: {}", &self.input[..self.val as usize]));
			// for debug.
			panic!("assertion failed at: {}..", &PROGRAMS.lock().unwrap()[self.tokens[pos].program_id][pos..pos+self.tokens[pos].val as usize]);
		}
	}
	pub fn consume_ty(&mut self, ty: TokenType) -> bool {
		let token = &self.tokens[self.pos];
		match (&token.ty, &ty) {
			(TokenString(_), TokenString(_)) => {
				return true;
			}
			_ => {
				if token.ty == ty {
					self.pos += 1;
					return true;
				} else {
					return false;
				}
			}
		}
	}
	pub fn is_typename(&mut self) -> bool {
		let token = &self.tokens[self.pos];
		match token.ty {
			TokenInt | TokenChar | TokenVoid 
			| TokenStruct | TokenTypeof => {
				self.pos += 1;
				return true;
			}
			_ => {
				return false;
			}
		}
	}
	pub fn ident(&mut self) -> String {
		let token = self.tokens[self.pos].clone();
		let name = String::from(&PROGRAMS.lock().unwrap()[token.program_id][token.pos..token.end]);
		if !self.consume_ty(TokenIdent) {
			// error(&format!("should be identifier at {}", &tokenset[*pos].input[*pos..]));
			// for debug.
			panic!("should be identifier at {}", &PROGRAMS.lock().unwrap()[token.program_id][token.pos..]);
		}
		return name;
	}
	pub fn getstring(&self) -> String {
		let token = &self.tokens[self.pos];
		match &token.ty {
			TokenString(sb) => { return sb.clone(); }
			_ => { panic!("{:?}", token); }
		}
	}
	pub fn getval(&self) -> i32 {
		return self.tokens[self.pos].val;
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

pub fn read_file(filename: &str) -> Result<String, Box<dyn std::error::Error>>{
	let content = std::fs::read_to_string(filename)?;
	return Ok(content);
}

fn read_string (p: &mut core::str::Chars, program_id: usize, pos: &mut usize) -> Token {

	let start = *pos;
	let mut sb = String::new();

	loop {
		let c = c_char(p, pos) as char;
		if c == '"'	{
			break;
		}
		sb.push(c);
	}
	return Token::new(TokenString(sb), 0, program_id, start, *pos, *LINE.lock().unwrap());
}

fn next_char(p: &mut core::str::Chars, pos: &mut usize) -> char {
	if let Some(c) = p.next() {
		*pos += 1;
		return c;
	} else {
		panic!("next char error.");
	}
}

fn c_char(p: &mut core::str::Chars, pos: &mut usize) -> u8 {
	let mut c = next_char(p, pos);
	if c != '\\' {
		// normal char literal ex. 'a', 'b' ...
		return c as u8; 
	}
	c = next_char(p, pos);
	if let Some(c_) = ESCAPED.lock().unwrap().get(&c) {
		// escaped char literal
		return *c_ as u8;
	}
	if c == 'x' {
		let mut val = 0;
		for _ in 0..2 {
			c = next_char(p, pos);
			val = val * 16 + hex(c);
		}
		return val;
	}
	if isoctal(c) {
		// octal-escaped-sequence in a char literal
		let mut val = c as u8 - '0' as u8;
		let mut pp = p.clone();
		for _ in 0..2 {
			c = next_char(&mut pp, pos);
			if isoctal(c) {
				val = 8 * val + c as u8 - '0' as u8;
				p.next();
			} else {
				*pos -= 1;
				return val;
			}
		}
		return val;
	}
	panic!("invalid char.");
}

fn isoctal(c: char) -> bool {
	match c {
		'0'..='7' => true,
		_ => false,
	}
}

fn hex(c: char) -> u8 {
	if let '0'..='9' = c {
		return c as u8 - '0' as u8;
	} else if let 'a'..='f' = c {
		return c as u8 - 'a' as u8 + 10;
	} else if let 'A'..='F' = c {
		return c as u8 - 'A' as u8 + 10;
	}
	panic!("{} c is not hex char.");
}

fn isxdigit(c: char) -> bool {
	match c {
		'0'..='9' | 'a'..='f' | 'A'..='F' => true,
		_ => false
	}
}

fn read_char (p: &mut core::str::Chars, program_id: usize, pos: &mut usize) -> Token {
	let start = *pos;
	let val = c_char(p, pos) as i32;
	assert!(p.next().unwrap() == '\'');
	*pos += 1;
	return Token::new(TokenNum, val, program_id, start, *pos, *LINE.lock().unwrap());
}

fn line_comment(p: &mut core::str::Chars, pos: &mut usize) {
	let start = *pos;
	*pos += 2;
	let mut pp = p.clone();
	pp.next();
	while let Some(c) = pp.next() {
		*pos += 1;
		if c == '\n' {
			break;
		}
	}
	for _ in 0..(*pos - start)-1 {
		p.next();
	}
	return;
}

fn block_comment(p: &mut core::str::Chars, program_id: usize, pos: &mut usize) {
	let start = *pos;
	*pos += 2;
	let mut pp = p.clone();
	pp.next();
	loop {
		if let Some(c) = pp.next() {
			*pos += 1;
			if c == '*' && &PROGRAMS.lock().unwrap()[program_id][*pos..*pos+1] == "/" {
				*pos += 1;
				break;
			}
		} else {
			error(get_path(program_id), *LINE.lock().unwrap(), "premature end of input.");
		}
	}
	for _ in 0..(*pos - start)-1 {
		p.next();
	}
	return;
}

fn signal(p: &mut core::str::Chars, program_id: usize, pos: &mut usize, input: &str) -> Option<Token> {
	for signal in &SIGNALS[..] {
		let len = signal.name.len();
		if input.len() >= *pos+len && *signal.name == input[*pos..*pos+len] {
			let token = Token::new(signal.ty.clone(), len as i32, program_id, *pos, *pos+len, *LINE.lock().unwrap());
			*pos += len;
			for _ in 0..len-1 { p.next(); }
			return Some(token);
		}
	}
	return None;
}

fn ident(p: &mut core::str::Chars, program_id: usize, pos: &mut usize, c: char) -> Token {
	let mut ident = String::new();
	ident.push(c);
	let mut len = 1;
	let mut pp = p.clone();
	let possub = *pos;
	loop {
		if let Some(cc) = pp.next() {
			if !cc.is_alphabetic() && !cc.is_ascii_digit() && cc != '_'{
				break;
			}
			p.next();
			ident.push(cc);
			len += 1;
			*pos += 1;
		}
	}
	*pos += 1;
	let token = Token::new(TokenType::from(ident), len, program_id, possub, *pos, *LINE.lock().unwrap());
	return token;
}

fn number(p: &mut core::str::Chars, program_id: usize, pos: &mut usize, input: &str, c: char) -> Token {

	if c == '0' && (&input[*pos+1..*pos+2] == "X" || &input[*pos+1..*pos+2] == "x") {
		*pos += 2;
		p.next();
		return hexadecimal(p, program_id, pos, input);
	}
	
	if c == '0' {
		*pos += 1;
		return octal(p, program_id, pos);
	}

	*pos += 1;
	return decimal(p, program_id, pos, c);
}

fn hexadecimal(p: &mut core::str::Chars, program_id: usize, pos: &mut usize, input: &str) -> Token{

	let mut pp = p.clone();
	let mut ishex = false;
	let mut num = 0;
	let possub = *pos;

	while let Some(c) = pp.next() {
		if isxdigit(c) {
			next_char(p, pos);
			num = num * 16 + hex(c) as i32;
			ishex = true;
		} else {
			if ishex {
				break;
			} else {
				error(get_path(program_id), *LINE.lock().unwrap(), &format!("bad hexadecimal number at {}..", &input[*pos..*pos+5]));
			}
		}
	}

	return Token::new(TokenNum, num, program_id, possub-2, *pos, *LINE.lock().unwrap());
}

fn decimal(p: &mut core::str::Chars, program_id: usize, pos: &mut usize, c: char) -> Token{

	let mut pp = p.clone();
	let possub = *pos;
	let mut num = c as i32 - '0' as i32;

	while let Some(c) = pp.next() {
		if let '0' ..= '9' = c {
			num = num * 10 + c as i32 - '0' as i32;
			p.next();
			*pos += 1;
			continue;
		}
		break;
	}

	return Token::new(TokenNum, num, program_id, possub-1, *pos, *LINE.lock().unwrap());
}

fn octal(p: &mut core::str::Chars, program_id: usize, pos: &mut usize) -> Token{

	let mut pp = p.clone();
	let possub = *pos;
	let mut num = 0;

	while let Some(c) = pp.next() {
		if let '0' ..= '9' = c {
			num = num * 8 + c as i32 - '0' as i32;
			p.next();
			*pos += 1;
			continue;
		}
		break;
	}

	return Token::new(TokenNum, num, program_id, possub-1, *pos, *LINE.lock().unwrap());
}

pub fn remove_backslash_or_crlf_newline(input: &mut String) {
	let mut i = 0;
	loop {
		match (input.get(i..i+1), input.get(i+1..i+2)) {
			(Some("\\"), Some("\n")) => {
				input.remove(i);
				input.remove(i);
				continue;
			}
			(Some("\r"), Some("\\")) => {
				input.remove(i);
				continue;
			}
			(Some(_), _) => {
				i += 1;
			}
			(None, _) => {
				break;
			}
		}
	}
}

fn strip_newline_tokens(tokens: Vec<Token>) -> Vec<Token> {
	let mut v = Vec::new();
	for ref mut token in tokens {
		let token = std::mem::replace(token, NONE_TOKEN.clone());
		if let TokenNewLine = token.ty {
			continue;
		}
		v.push(token);
	}
	return v;
}

// Returns true if Token t followed a space or a comment
// in an original source file.
fn need_space(token: &Token) -> bool {
	let start = token.pos as i32 - 1;
	let program_id = token.program_id;
	if start >= 0 && &PROGRAMS.lock().unwrap()[program_id][start as usize..start as usize + 1] == " " {
		return true;
	} else {
		return false;
	}
}

pub fn stringize(tokens: &Vec<Token>) -> Token {
	let mut sb = String::new();
	let start = tokens[0].pos;
	let program_id = tokens[0].program_id;
	let line = tokens[0].line;
	let mut end = start;
	for i in 0..tokens.len() {
		let token = &tokens[i];
		if token.ty == TokenNewLine {
			continue;
		}
		if i > 0 && need_space(token) {
			sb.push(' ');
			end += 1;
		}
		sb.push_str(&String::from(&PROGRAMS.lock().unwrap()[program_id][token.pos..token.end]));
		end += token.end-token.pos;
	}
	return Token::new(TokenString(sb), 0, program_id, start, end, line);
}

pub fn scan(program_id: usize, add_eof: bool) -> Vec<Token> {
	
	let mut tokens: Vec<Token> = vec![];
	let mut pos = 0;
	let input = PROGRAMS.lock().unwrap()[program_id].clone();
	let mut p = input.chars();
	
	while let Some(c) = p.next() {

		// \n
		if c == '\n' {
			tokens.push(Token::new(TokenNewLine, 0, program_id, pos, pos+1, *LINE.lock().unwrap()));
			pos += 1;
			*LINE.lock().unwrap() += 1;
			continue;
		}

		// space
		if c.is_whitespace() {
			pos += 1;
			continue;
		}

		// Line Comment
		if c == '/' && &input[pos+1..pos+2] == "/" {
			line_comment(&mut p, &mut pos);
			continue;
		}
		
		// Block Comment
		if c == '/' && &input[pos+1..pos+2] == "*" {
			block_comment(&mut p, program_id, &mut pos);
			continue;
		}

		// char literal
		if c == '\'' {
			pos += 1;
			tokens.push(read_char(&mut p, program_id, &mut pos));
			continue;
		}

		// string literal
		if c == '"' {
			pos += 1;
			let mut string_token = read_string(&mut p, program_id, &mut pos);
			if !tokens.is_empty() {
				if let (TokenString(s1), TokenString(s2)) = (&tokens.last().unwrap().ty, &string_token.ty) {
					let s = format!("{}{}", s1, s2);
					tokens.pop();
					string_token.ty = TokenString(s);
					tokens.push(string_token);
					continue;
				}
			}
			tokens.push(string_token);
			continue;
		}
		
		// signal
		if let Some(token) = signal(&mut p, program_id, &mut pos, &input) {
			tokens.push(token);
			continue;
		}

		// ident
		if c.is_alphabetic() || c == '_' {
			tokens.push(ident(&mut p, program_id, &mut pos, c));
			continue;
		}
		
		// number
		if c.is_digit(10) {
			tokens.push(number(&mut p, program_id, &mut pos, &input, c));
			continue;
		}

		error(get_path(program_id), *LINE.lock().unwrap(), &format!("cannot scan at {}", &input[pos..]));
	}

	// guard
	if add_eof {
		let token = Token::new(TokenEof, 0, program_id, pos, pos, *LINE.lock().unwrap());
		tokens.push(token);
	}
	
	return tokens;
}

pub fn tokenize(program_id: usize, add_eof: bool) -> Vec<Token> {
	*LINE.lock().unwrap() = 1;
	let tokens = scan(program_id, add_eof);
	let tokens = preprocess(tokens);
	let tokens = strip_newline_tokens(tokens);
	return tokens;
}