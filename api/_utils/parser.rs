use crate::_utils::{error, lexer};
use error::Error;
use lexer::{LexerIter, Token, TokenKind};
use std::iter::Peekable;

#[derive(Debug)]
pub struct LogicOp {
	pub var: Token,
	pub pos: usize,
	pub symbol: Token,
	pub args: Vec<Token>,
}

#[derive(Debug)]
pub enum Operation {
	Logic(LogicOp),
}

#[derive(Debug)]
pub struct TestbenchAssignment {
	pub iden: Token,
	pub value: bool,
}

#[derive(Debug)]
pub struct Breakpoint {
	pub symbol: Token,
	pub time: u32,
	pub assignments: Vec<TestbenchAssignment>,
}

#[derive(Debug)]
pub struct Module {
	pub name: Token,
	pub ins: Vec<Token>,
	pub outs: Vec<Token>,
}

#[derive(Debug)]
pub struct Enviroment {
	pub name: Token,
	pub ins: Vec<Token>,
	pub outs: Vec<Token>,
}

#[derive(Debug)]
pub struct Implementation {
	pub name: Token,
	pub body: Vec<Operation>,
}

#[derive(Debug)]
pub struct Test {
	pub module: Token,
	pub name: Token,
	pub body: Vec<Breakpoint>,
}

#[derive(Debug)]
pub enum Def {
	Module(Module),
	Enviroment(Enviroment),
	Implementation(Implementation),
	Test(Test),
}

pub struct ParserIter<'a> {
	tokens: Peekable<LexerIter<'a>>,
}

impl<'a> ParserIter<'a> {
	pub fn new(tokens: LexerIter<'a>) -> Self {
		Self {
			tokens: tokens.peekable(),
		}
	}

	fn get_token(&mut self, kind: TokenKind, value_pre: Option<&[&str]>) -> Result<Token, Error> {
		let value: Option<Vec<String>> =
			value_pre.map(|x| x.iter().map(|a| a.to_string()).collect());
		let token = self.tokens.next().ok_or(Error::EndOfFile)?;
		let res = match (token.kind == kind, value) {
			(true, None) => Ok(token),
			(true, Some(value)) => {
				if value.contains(&token.value) {
					Ok(token)
				} else {
					Err(Error::UnexpectedToken(
						token.value.to_string(),
						token.pos,
						token.value.len(),
					))
				}
			}
			(false, _) => Err(Error::UnexpectedToken(
				token.value.to_string(),
				token.pos,
				token.value.len(),
			)),
		};
		res
	}

	fn parse_args(&mut self) -> Result<Vec<Token>, Error> {
		let _ = self.get_token(TokenKind::Sign, Some(&["("]))?;
		let mut args = Vec::new();
		while let Some(_) = self.tokens.peek() {
			let token = self.get_token(TokenKind::Name, None)?;

			args.push(token);
			let token = self.get_token(TokenKind::Sign, Some(&[",", ")"]))?;

			if token.value == ")" {
				break;
			}
		}
		Ok(args)
	}

	fn parse_operation(&mut self) -> Result<Operation, Error> {
		let _ = self.get_token(TokenKind::Keyword, Some(&["let"]))?;
		let token = self.get_token(TokenKind::Name, None)?;
		let _ = self.get_token(TokenKind::Sign, Some(&["="]))?;
		let token1 = self.get_token(TokenKind::Operation, Some(&["not", "nor"]))?;
		let args = self.parse_args()?;
		let _ = self.get_token(TokenKind::Sign, Some(&[";"]))?;

		let op = Operation::Logic(LogicOp {
			var: token,
			symbol: token1.clone(),
			pos: token1.pos,
			args,
		});

		Ok(op)
	}

	fn parse_operations(&mut self) -> Result<Vec<Operation>, Error> {
		let mut ops = Vec::new();
		while let Some(token) = self.tokens.peek() {
			let exp = match (token.kind, token.value.as_str()) {
				(TokenKind::Keyword, "let") => self.parse_operation()?,
				_ => break,
			};

			ops.push(exp);
		}

		let _ = self.get_token(TokenKind::Sign, Some(&["}"]))?;
		Ok(ops)
	}

	fn parse_impl(&mut self) -> Result<Def, Error> {
		let _ = self.get_token(TokenKind::Keyword, Some(&["impl"]))?;
		let name = self.get_token(TokenKind::Name, None)?;
		let _ = self.get_token(TokenKind::Sign, Some(&["{"]))?;

		let ops = self.parse_operations()?;

		Ok(Def::Implementation(Implementation { name, body: ops }))
	}

	fn parse_assignment(&mut self) -> Result<TestbenchAssignment, Error> {
		let token = self.get_token(TokenKind::Name, None)?;
		let _ = self.get_token(TokenKind::Sign, Some(&["="]))?;
		let bool_token = self.get_token(TokenKind::Value, None)?;
		let _ = self.get_token(TokenKind::Sign, Some(&[";"]))?;

		let bool_value = bool_token.value.parse::<bool>();
		let bool_value = match bool_value {
			Ok(value) => value,
			Err(_) => Err(Error::UnexpectedToken(
				bool_token.value.to_string(),
				bool_token.pos,
				bool_token.value.len(),
			))?,
		};

		Ok(TestbenchAssignment {
			iden: token,
			value: bool_value,
		})
	}

	fn parse_breakpoint(&mut self) -> Result<Breakpoint, Error> {
		let token = self.get_token(TokenKind::Sign, Some(&["@"]))?;
		let time_token = self.get_token(TokenKind::Value, None)?;

		let mut assignments = Vec::new();
		while let Some(token) = self.tokens.peek() {
			match token.kind {
				TokenKind::Name => {
					let ass = self.parse_assignment()?;
					assignments.push(ass);
				}
				_ => break,
			}
		}

		let parsed_time = time_token.value.parse::<u32>();
		let parsed_time = match parsed_time {
			Ok(val) => val,
			Err(_) => Err(Error::UnexpectedToken(
				time_token.value.to_string(),
				time_token.pos,
				time_token.value.len(),
			))?,
		};

		Ok(Breakpoint {
			symbol: token,
			time: parsed_time,
			assignments,
		})
	}

	fn parse_breakpoints(&mut self) -> Result<Vec<Breakpoint>, Error> {
		let mut breakpoints = Vec::new();
		while let Some(token) = self.tokens.peek() {
			match (token.kind, token.value.as_str()) {
				(TokenKind::Sign, "@") => {
					let exp = self.parse_breakpoint()?;
					breakpoints.push(exp);
				}
				_ => break,
			}
		}

		Ok(breakpoints)
	}

	fn parse_test(&mut self) -> Result<Def, Error> {
		let _ = self.get_token(TokenKind::Keyword, Some(&["test"]))?;
		let name = self.get_token(TokenKind::Name, None)?;
		let _ = self.get_token(TokenKind::Keyword, Some(&["for"]))?;
		let module = self.get_token(TokenKind::Name, None)?;
		let _ = self.get_token(TokenKind::Sign, Some(&["{"]))?;
		let breakpoints = self.parse_breakpoints()?;
		let _ = self.get_token(TokenKind::Sign, Some(&["}"]))?;

		Ok(Def::Test(Test {
			module,
			name,
			body: breakpoints,
		}))
	}

	fn parse_mod(&mut self) -> Result<Def, Error> {
		let _ = self.get_token(TokenKind::Keyword, Some(&["mod"]))?;
		let name = self.get_token(TokenKind::Name, None)?;
		let _ = self.get_token(TokenKind::Sign, Some(&["{"]))?;
		let _ = self.get_token(TokenKind::Keyword, Some(&["in"]))?;
		let ins = self.parse_args()?;
		self.get_token(TokenKind::Sign, Some(&[";"]))?;
		let _ = self.get_token(TokenKind::Keyword, Some(&["out"]))?;
		let outs = self.parse_args()?;
		self.get_token(TokenKind::Sign, Some(&[";"]))?;

		let _ = self.get_token(TokenKind::Sign, Some(&["}"]))?;

		Ok(Def::Module(Module { name, ins, outs }))
	}

	fn parse_env(&mut self) -> Result<Def, Error> {
		let _ = self.get_token(TokenKind::Keyword, Some(&["env"]))?;
		let name = self.get_token(TokenKind::Name, None)?;
		let _ = self.get_token(TokenKind::Sign, Some(&["{"]))?;
		let _ = self.get_token(TokenKind::Keyword, Some(&["in"]))?;
		let ins = self.parse_args()?;
		self.get_token(TokenKind::Sign, Some(&[";"]))?;
		let _ = self.get_token(TokenKind::Keyword, Some(&["out"]))?;
		let outs = self.parse_args()?;
		self.get_token(TokenKind::Sign, Some(&[";"]))?;

		let _ = self.get_token(TokenKind::Sign, Some(&["}"]))?;

		Ok(Def::Enviroment(Enviroment { name, ins, outs }))
	}
}

impl<'a> Iterator for ParserIter<'a> {
	type Item = Result<Def, Error>;

	fn next(&mut self) -> Option<Result<Def, Error>> {
		while let Some(token) = self.tokens.peek() {
			return match (token.kind, token.value.as_str()) {
				(TokenKind::Keyword, "mod") => Some(self.parse_mod()),
				(TokenKind::Keyword, "env") => Some(self.parse_env()),
				(TokenKind::Keyword, "impl") => Some(self.parse_impl()),
				(TokenKind::Keyword, "test") => Some(self.parse_test()),
				_ => Some(Err(Error::UnexpectedToken(
					token.value.to_string(),
					token.pos,
					token.value.len(),
				))),
			};
		}
		None
	}
}
