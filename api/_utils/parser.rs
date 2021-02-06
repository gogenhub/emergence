use crate::_utils::{helpers, lexer};
use helpers::Error;
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
pub struct Function {
	pub iden: Token,
	pub params: Vec<Token>,
	pub out: Token,
	pub body: Vec<Operation>,
}

#[derive(Debug)]
pub enum Def {
	Function(Function),
	Test(Test),
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
pub struct Test {
	pub iden: Token,
	pub params: Vec<Token>,
	pub body: Vec<Breakpoint>,
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
		let value: Option<Vec<String>> = value_pre.map(|x| x.iter().map(|a| a.to_string()).collect());
		let token = self.tokens.next().ok_or(Error::EndOfFile)?;
		match (token.kind == kind, value) {
			(true, None) => Ok(token),
			(true, Some(value)) => {
				if value.contains(&token.value) {
					Ok(token)
				} else {
					Err(Error::UnexpectedToken(token.pos, token.value.len()))
				}
			}
			(false, _) => Err(Error::UnexpectedToken(token.pos, token.value.len())),
		}
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

	fn parse_out(&mut self) -> Result<Token, Error> {
		let token = self.get_token(TokenKind::Name, None)?;
		let _ = self.get_token(TokenKind::Sign, Some(&[";"]))?;

		Ok(token)
	}

	fn parse_operation(&mut self) -> Result<Operation, Error> {
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

	fn parse_operations(&mut self) -> Result<(Vec<Operation>, Token), Error> {
		let mut ops = Vec::new();
		while let Some(token) = self.tokens.next() {
			let exp = match (token.kind, token.value.as_str()) {
				(TokenKind::Keyword, "let") => self.parse_operation()?,
				(TokenKind::Keyword, "out") => break,
				_ => return Err(Error::EndOfFile),
			};

			ops.push(exp);
		}
		let out = self.parse_out()?;

		let _ = self.get_token(TokenKind::Sign, Some(&["}"]))?;
		Ok((ops, out))
	}

	fn parse_func(&mut self) -> Result<Def, Error> {
		let name = self.get_token(TokenKind::Name, None)?;
		let args = self.parse_args()?;
		let _ = self.get_token(TokenKind::Sign, Some(&["{"]))?;

		let (ops, out) = self.parse_operations()?;

		Ok(Def::Function(Function {
			iden: name,
			params: args,
			out,
			body: ops,
		}))
	}

	fn parse_assignment(&mut self) -> Result<TestbenchAssignment, Error> {
		let token = self.get_token(TokenKind::Name, None)?;
		let _ = self.get_token(TokenKind::Sign, Some(&["="]))?;
		let bool_token = self.get_token(TokenKind::Value, None)?;
		let _ = self.get_token(TokenKind::Sign, Some(&[";"]))?;

		let bool_value = bool_token.value.parse::<bool>();
		let bool_value = match bool_value {
			Ok(value) => value,
			Err(_) => Err(Error::UnexpectedToken(bool_token.pos, bool_token.value.len()))?,
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
			if token.kind != TokenKind::Name {
				break;
			}

			let ass = self.parse_assignment()?;
			assignments.push(ass);
		}

		let parsed_time = time_token.value.parse::<u32>();
		let parsed_time = match parsed_time {
			Ok(val) => val,
			Err(_) => Err(Error::UnexpectedToken(time_token.pos, time_token.value.len()))?,
		};

		Ok(Breakpoint {
			symbol: token,
			time: parsed_time,
			assignments: assignments,
		})
	}

	fn parse_breakpoints(&mut self) -> Result<Vec<Breakpoint>, Error> {
		let mut breakpoints = Vec::new();
		while let Some(token) = self.tokens.peek() {
			if token.kind == TokenKind::Sign && token.value == "}" {
				break;
			}

			let exp = self.parse_breakpoint()?;
			breakpoints.push(exp);
		}

		let _ = self.get_token(TokenKind::Sign, Some(&["}"]))?;
		Ok(breakpoints)
	}

	fn parse_test(&mut self) -> Result<Def, Error> {
		let name = self.get_token(TokenKind::Name, None)?;
		let args = self.parse_args()?;
		let _ = self.get_token(TokenKind::Sign, Some(&["{"]))?;
		let breakpoints = self.parse_breakpoints()?;

		Ok(Def::Test(Test {
			iden: name,
			params: args,
			body: breakpoints,
		}))
	}
}

impl<'a> Iterator for ParserIter<'a> {
	type Item = Result<Def, Error>;

	fn next(&mut self) -> Option<Result<Def, Error>> {
		while let Some(_) = self.tokens.peek() {
			let token = self.get_token(TokenKind::Keyword, Some(&["func", "test"]));
			if token.is_err() {
				return Some(Err(token.err().unwrap()));
			}
			let token = token.unwrap();
			return match (token.kind, token.value.as_str()) {
				(TokenKind::Keyword, "func") => Some(self.parse_func()),
				(TokenKind::Keyword, "test") => Some(self.parse_test()),
				_ => None,
			};
		}
		None
	}
}
