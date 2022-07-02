/*

hello_world ast:
chunk [
	Import { name = "std" },
	Function { name = "boppers", parameters = {}, body = {
		Assignment { name = "beop", value = String("hello") },
		Call { 
			chunk = GetValue { name = "std", uproots = ["print"] },
			arguments = [ GetValue("beop") ]
		} 
	}
]

*/

#![allow(dead_code)]

use std::cell::{RefCell, Cell};
use super::lexer::{ChunkResult, Tokens};

const IMPORT_KEYWORD: &str = "beop";
const DECLARE_KEYWORD: &str = "bop";

fn parse_iden(iden: &Tokens) -> Result<&String, bool> {
	match iden {
		Tokens::Iden(id) => Ok(id),
		_ => Err(false)
	}
}

fn parse_datatype(dtype: &Tokens) -> Option<AstDatatypes> {
	match dtype {
		Tokens::Datatype(dt) => {
			match dt.as_str() {
				"string" => Some(AstDatatypes::String),
				"number" => Some(AstDatatypes::Number),
				"bool" => Some(AstDatatypes::Bool),
				"void" => Some(AstDatatypes::Void),
				"func" => Some(AstDatatypes::Func),
				_ => panic!("unknown datatype")
			}
		},
		_ => None
	}
}

fn punctuator_check(token: &Tokens, c: char) -> bool {
	match token {
		Tokens::Punctuator(punc) => punc == &c,
		_ => false
	}
}

#[derive(Debug)]
pub enum AstDatatypes {
	String,
	Number,
	Bool,
	Void,
	Func
}

#[derive(Debug)]
pub enum AstValue {
	String(String),
	Number(f64),
	Bool(bool),
	Void,
	Func(Box<AstChunk>)
}

#[derive(Debug)]
pub enum AstInstruction {
	Import{ name: String },
	Function{ name: String, parameters: Vec<Tokens>, body: AstChunk },
	Assignment{ name: String, datatype: AstDatatypes, value: Vec<Tokens> },
	Call{ chunk: Box<Self>, arguments: Vec<Tokens> },
	GetValue { name: String, uproots: Vec<String> }
}

#[derive(Debug)]
pub struct AstChunk { 
	pub tree: RefCell<Vec<AstInstruction>>,
	lexed: ChunkResult,
	idx: Cell<usize>
}

impl AstChunk {
	pub fn new(lexed: ChunkResult) -> Self {
		Self { tree: RefCell::new(vec![]), lexed, idx: Cell::new(0) }
	}

	pub fn from_tokens(tokens: Vec<&Tokens>) -> Self {
		Self { 
			tree: RefCell::new(vec![]), 
			lexed: ChunkResult(tokens.into_iter().map(|v| v.clone()).collect::<Vec<Tokens>>()), 
			idx: Cell::new(0) 
		}
	}

	// iterator functions
	fn next(&self) -> Option<&Tokens> {
		let val = self.idx.get();
		self.idx.set(val + 1);
		self.lexed.0.get(val)
	}
	//

	pub fn add(&self, instr: AstInstruction) {
		self.tree.borrow_mut().push(instr)
	}

	pub fn check_puncuator(&self, puncuator: char) -> Result<bool, &Tokens> {
		let punc = self.next().expect("no punctuator");
		match punc {
			Tokens::Punctuator(c) => if c == &puncuator { Ok(true) } else { Err(punc) },
			_ => Err(punc)
		}
	}

	pub fn check_puncuators(&self, puncuators: &[char]) -> Result<char, &Tokens> {
		let punc = self.next().expect("no punctuator");
		match punc {
			Tokens::Punctuator(c) => {
				if puncuators.contains(c) {
					Ok(*c)
				} else {
					Err(punc)
				}
			},
			_ => Err(punc)
		}
	}

	pub fn till_char(&self, c: char) -> Vec<&Tokens> {
		let mut feed = vec![];

		loop {
			let instr = self.check_puncuator(c);
			if instr.is_ok() {
				break;
			}

			feed.push(instr.err().unwrap());
		}

		feed
	}

	pub fn till_seperator(&self) -> Vec<&Tokens> {
		self.till_char(';')
	}

	pub fn get_matching_body(&self, begin_match: char, end_match: char) -> Vec<&Tokens> {
		let mut body = vec![];
		let mut matching = 0;

		loop {
			let instr = self.check_puncuators(&[begin_match, end_match]);
			if instr.is_ok() {
				matching += if instr.unwrap() == begin_match { 1 } else { -1 };
				if matching <= 0 {
					break;
				}
			}

			body.push(instr.err().unwrap());
		}

		body
	}

	pub fn handle_keyword(&self, key: &String) {
		match key.as_str() {
			IMPORT_KEYWORD => {
				let iden = parse_iden(self.next().expect("expected iden")).expect("expected iden");
				self.add(AstInstruction::Import { name: iden.to_string() })
			},
			DECLARE_KEYWORD => {
				let dtype = self.next().expect("unfinished declaration, no datatype");
				let datatype = parse_datatype(dtype);

				if let Some(datatype) = datatype {
					match datatype {
						AstDatatypes::Func => {
							let name_token = self.next().expect("unfinished declaration, no name");
							let name = parse_iden(name_token).expect("invalid name");
							self.check_puncuator('(').expect("no param start");
							let params = self.till_char(')').into_iter()
								.filter(|token| !punctuator_check(token, ','))
								.map(|val| val.clone())
								.collect::<Vec<Tokens>>();
							self.check_puncuator('{').expect("no body start");
							let body = self.get_matching_body('{', '}');
							let chunk = Self::from_tokens(body);
							chunk.create();
							self.add(AstInstruction::Function { 
								name: name.to_string(), 
								parameters: params, 
								body: chunk 
							})
						},
						_ => {
							let name_token = self.next().expect("unfinished declaration, no name");
							self.check_puncuator(':').expect("no punctuator");
							let value = self.till_seperator();

							let name = parse_iden(name_token).expect("invalid name");

							self.add(AstInstruction::Assignment { 
								name: name.to_string(),
								datatype,
								value: 
									value.into_iter().map(|val| val.clone())
									.collect::<Vec<Tokens>>()
							})
						}
					}
				} else {
					panic!("no type specified");
				}
			},
			_ => println!("unimplemented keyword: {}", key)
		}
	}

	pub fn handle_iden(&self, iden: &String) {
		let mut uproots = vec![];
		loop {
			let token_res = self.check_puncuators(&['=', '(', ';']);
			if token_res.is_ok() {
				let punc = token_res.ok().unwrap();
				if punc == '=' {
					println!("assignment!");
				} else if punc == '(' {
					// get args & remove , seperators, and take ownership via cloning
					let args = self.till_char(')').into_iter()
						.filter(|token| !punctuator_check(token, ','))
						.map(|val| val.clone())
						.collect::<Vec<Tokens>>();

					// println!("call! {:?} {:?} args: {:?}", iden, uproots, args);
					self.add(AstInstruction::Call { 
						chunk: Box::new(AstInstruction::GetValue { name: iden.to_string(), uproots: uproots }), 
						arguments: args 
					})
				}

				break;
			}
			let token = token_res.err().unwrap();
			if !punctuator_check(token, '.') {
				// uproots
				match token {
					Tokens::Iden(uproot) => uproots.push(uproot.to_string()),
					_ => panic!("invalid uproot: {:?}, iden {:?}, previous: {:?}", token, iden, uproots)
				}
			}
		}
	}
	
	pub fn create(&self) {
		while let Some(instr) = self.next() {
			match instr {
				Tokens::Keyword(key) => self.handle_keyword(key),
				Tokens::Iden(iden) => self.handle_iden(iden),
				_ => {
					// println!("skipping unknown token");
				}
			}
		}

		// println!("{:#?}", self.tree.borrow());
	}
}

pub fn create_ast(lexed: ChunkResult) -> AstChunk {
	let ast = AstChunk::new(lexed);
	ast.create();
	ast
}