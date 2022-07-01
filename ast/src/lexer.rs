#![allow(dead_code)]
use std::fmt;

const KEYWORDS: [&str; 5] = [
	"beop", "bop", "boop", "beep", "booop",
];
const TYPES: [&str; 5] = [
	"func", "char", "number", "void", "string",
];
const PUNCTUATION: [char; 8] = [
	'{', '}', ';', ':', '=', '(', ')', '.',
];
const COMPARISON: [&str; 8] = [
	"==", ">=", "<=", ">", "<", "!=", "&&", "||"
];
const OPERATION: [&str; 8] = [
	"+", "-", "*", "/", "<<", ">>", "|", "^"
];
const STRING: [char; 3] = [
	'\'', '"', '`',
];
const NUMBER: [char; 10] = [
	'0', '1', '2', '3', '4', '5', '6', '7', '8', '9',
];

// tokens & pre-processor tokens
enum Tokens {
	Keyword(String),
	Number(f64),
	String(String),
	Datatype(String),
	Operation(String),
	Comparsion(String),
	Punctuator(char),
	Iden(String)
}
impl fmt::Debug for Tokens {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		match self {
			Tokens::Keyword(str) => write!(f, "Keyword	{}", str),
			Tokens::Number(n) => write!(f, "Number	{}", n),
			Tokens::String(str) => write!(f, "String	{}", str),
			Tokens::Datatype(str) => write!(f, "Datatype	{}", str),
			Tokens::Operation(str) => write!(f, "Operation	{}", str),
			Tokens::Comparsion(str) => write!(f, "Comparison	{}", str),
			Tokens::Punctuator(c) => write!(f, "Punctuator	{}", c),
			Tokens::Iden(str) => write!(f, "Iden	{}", str),
		}
	}
}

pub struct ChunkResult(Vec<Tokens>);
impl ChunkResult {
	fn new() -> Self {
		Self(vec![])
	}

	fn add(&mut self, token: Tokens) {
		self.0.push(token)
	}
}
impl fmt::Debug for ChunkResult {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		write!(f, "{:?}", self.0)
	}
}

struct Chunk<'a> {
	bytes: &'a[u8],
	current_feed: Vec<char>,
	idx: usize,
	result: ChunkResult
}

impl<'a> Chunk<'a> {
	fn new(src: &'a String) -> Self {			
		Self { 
			bytes: src.as_bytes(), 
			current_feed: vec![],
			idx: 0, 
			result: ChunkResult::new(),
		}
	}

	// iter functions
	fn has_next(&self) -> bool {
		self.bytes.len() > self.idx
	}
	fn has_prev(&self) -> bool {
		self.idx - 1 > 0
	}
	fn peak_next(&self) -> char {
		self.bytes[self.idx + 1] as char
	}
	fn peak_prev(&self) -> char {
		self.bytes[self.idx - 1] as char
	}
	fn next(&mut self) -> char {
		self.idx += 1;
		self.bytes[self.idx] as char
	}
	fn prev(&mut self) -> char {
		self.idx -= 1;
		self.bytes[self.idx] as char
	}
	fn current(&self) -> char {
		self.bytes[self.idx] as char
	}

	fn consume(&mut self) {
		self.current_feed.push(self.current());
		self.idx += 1;
	}
	fn skip(&mut self) {
		self.idx += 1;
	}
	fn feed(&self) -> String {
		self.current_feed.iter().collect::<String>()
	}
	fn clean(&mut self) -> String {
		let str = self.feed();
		self.current_feed = vec![];

		str
	} 

	fn is_newline(&self) -> bool {
		let char = self.current();
		char == '\n' || char == '\r'
	}

	//
	fn skip_line(&mut self) {
		while self.has_next() {
			self.next();
			if self.is_newline() {
				self.next();
				break;
			}
		}
	}

	fn add_token(&mut self, token: Tokens) {
		self.result.add(token)
	}

	// returns output or error
	fn tokenize(&mut self) -> Result<Vec<String>, String> {
		while self.has_next() {
			let char = self.current();
			if char == '\r' || char == '\n' { // skip all new lines
				self.skip();
				continue;
			}

			let feed = self.feed();
			let trimmed_feed = feed.trim();
			// println!("{} : '{}'", trimmed_feed, self.current());

			// char based

			// skip commented lines
			if char == '/' && self.peak_next() == '/' {
				self.skip_line();
				continue;
			}

			// puctuation
			if PUNCTUATION.contains(&char) {
				if trimmed_feed.len() != 0 {
					self.add_token(Tokens::Iden(trimmed_feed.to_string()));
				}

				self.add_token(Tokens::Punctuator(char));
				self.clean();
				self.skip();
				continue;
			}

			// strings
			if STRING.contains(&char) {
				if trimmed_feed.len() != 0 {
					self.add_token(Tokens::Iden(trimmed_feed.to_string()));
				}
				self.clean();
				self.skip();

				let mut is_escaped = false;
				while self.has_next() {
					let str_char = self.current();
					if str_char == char && !is_escaped {
						self.add_token(Tokens::String(self.feed()));
						self.clean();
						self.skip();
						
						break;
					}
					if str_char == '\\' {
						is_escaped = !is_escaped;
					}

					self.consume();
				}

				continue;
			}

			if NUMBER.contains(&char) {
				if trimmed_feed.len() != 0 {
					self.add_token(Tokens::Iden(trimmed_feed.to_string()));
				}
				self.clean();

				while self.has_next() {
					let c = self.current();
					println!("{}", c);
					if !NUMBER.contains(&c) && c != '.' {
						let number = {
							let a = self.feed();
							let b = a.trim();
							println!("number: {}", b);
							b.parse::<f64>().unwrap()
						};
						self.add_token(Tokens::Number(number));
						self.clean();
						
						break;
					}

					self.consume();
				}

				continue;
			}
			
			// feed based

			// keywords
			if KEYWORDS.contains(&trimmed_feed) && self.current() == ' ' {
				self.add_token(Tokens::Keyword(trimmed_feed.to_owned()));
				self.clean();
				continue;
			}

			// reserved datatypes
			if TYPES.contains(&trimmed_feed) && self.current() == ' ' {
				self.add_token(Tokens::Datatype(trimmed_feed.to_owned()));
				self.clean();
				continue;
			}

			// comparison
			if COMPARISON.contains(&trimmed_feed) {
				self.add_token(Tokens::Comparsion(trimmed_feed.to_owned()));
				self.clean();
				continue;
			}

			// oepration
			if OPERATION.contains(&trimmed_feed) {
				self.add_token(Tokens::Operation(trimmed_feed.to_owned()));
				self.clean();
				continue;
			}

			self.consume();
			// println!("{}", char);
		}

		println!("{:#?}", self.result.0);

		Ok(vec![])
	}
}

pub fn analyze_chunk(src: String) -> Result<ChunkResult, String> {
	let mut chunk = Chunk::new(&src);
	match chunk.tokenize() {
		Ok(_output) => {
			Ok(chunk.result)
		},
		Err(e) => {
			Err(e)
		}
	}
}