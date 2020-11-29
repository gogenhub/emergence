use crate::_utils::{helpers, lexer};
use helpers::{eof, err, exp, get_breakpoint_kind, get_op_kind, Error, Loggable};
use lexer::{LexerIter, TokenKind};
use serde::Serialize;
use std::iter::Peekable;

#[derive(Debug, Clone, Serialize)]
pub struct Arg {
	pub name: String,
	pub pos: usize,
}

#[derive(Clone, Debug, PartialEq)]
pub enum OperationKind {
	NOT,
	NOR,
	Unknown,
}

#[derive(Debug)]
pub struct Operation {
	pub var: Arg,
	pub pos: usize,
	pub kind: OperationKind,
	pub args: Vec<Arg>,
}

#[derive(Debug)]
pub struct Function {
	pub name: String,
	pub pos: usize,
	pub params: Vec<Arg>,
	pub out: Arg,
	pub body: Vec<Operation>,
}

#[derive(Debug)]
pub enum Def {
	Function(Function),
	Test(Test),
}

#[derive(Debug)]
pub struct TestbenchAssignment {
	pub name: String,
	pub pos: usize,
	pub value: bool,
}

#[derive(Debug, PartialEq)]
pub enum BreakpointKind {
	At,
	Unknown,
}

#[derive(Debug)]
pub struct Breakpoint {
	pub name: String,
	pub pos: usize,
	pub kind: BreakpointKind,
	pub time: u32,
	pub assignments: Vec<TestbenchAssignment>,
}

#[derive(Debug)]
pub struct Test {
	pub pos: usize,
	pub name: String,
	pub params: Vec<Arg>,
	pub body: Vec<Breakpoint>,
}

pub struct ParserIter<'a> {
	tokens: Peekable<LexerIter<'a>>,
}

impl Def {
	pub fn is_func(&self) -> bool {
		match self {
			Self::Function(_) => true,
			_ => false,
		}
	}

	pub fn func(self) -> Function {
		match self {
			Self::Function(f) => f,
			_ => panic!("yo?"),
		}
	}

	pub fn test(self) -> Test {
		match self {
			Self::Test(t) => t,
			_ => panic!("yo?"),
		}
	}
}

impl Loggable for Arg {
	fn value(&self) -> &str {
		&self.name
	}
	fn pos(&self) -> usize {
		self.pos
	}
}

impl Loggable for Breakpoint {
	fn value(&self) -> &str {
		&self.name
	}
	fn pos(&self) -> usize {
		self.pos
	}
}

impl Loggable for Function {
	fn value(&self) -> &str {
		&self.name
	}
	fn pos(&self) -> usize {
		self.pos
	}
}

impl Loggable for Test {
	fn value(&self) -> &str {
		&self.name
	}
	fn pos(&self) -> usize {
		self.pos
	}
}

impl Loggable for TestbenchAssignment {
	fn value(&self) -> &str {
		&self.name
	}
	fn pos(&self) -> usize {
		self.pos
	}
}

impl<'a> ParserIter<'a> {
	pub fn new(tokens: LexerIter<'a>) -> Self {
		Self { tokens: tokens.peekable() }
	}

	fn parse_args(&mut self) -> Result<Vec<Arg>, Error> {
		let mut args = Vec::new();
		while let Some(token) = self.tokens.next() {
			exp(token.kind == TokenKind::Name, "Expected name, got: ", &token)?;

			args.push(Arg {
				name: token.value,
				pos: token.pos,
			});
			let token = self.tokens.next().ok_or(eof())?;
			exp(
				token.kind == TokenKind::Sign && (token.value != "," || token.value != ")"),
				"Expected token ',' or ')', got: ",
				&token,
			)?;

			if token.value == ")" {
				break;
			}
		}
		Ok(args)
	}

	fn parse_out(&mut self) -> Result<Arg, Error> {
		let token = self.tokens.next().ok_or(eof())?;
		exp(
			token.kind == TokenKind::Keyword && token.value == "out",
			"Expected 'out' keyword, got: ",
			&token,
		)?;

		let name = self.tokens.next().ok_or(eof())?;
		exp(name.kind == TokenKind::Name, "Expected name, got: ", &name)?;

		let eol = self.tokens.next().ok_or(eof())?;
		exp(eol.kind == TokenKind::Sign && eol.value == ";", "Expected sign ';', got: ", &eol)?;

		Ok(Arg {
			name: name.value,
			pos: name.pos,
		})
	}

	fn parse_operation(&mut self) -> Result<Operation, Error> {
		let token = self.tokens.next().ok_or(eof())?;
		exp(token.kind == TokenKind::Name, "Expected name, got:", &token)?;

		let equal = self.tokens.next().ok_or(eof())?;
		exp(equal.kind == TokenKind::Sign && equal.value == "=", "Expected sign '=', got:", &equal)?;

		let token1 = self.tokens.next().ok_or(eof())?;
		let token2 = self.tokens.next().ok_or(eof())?;

		let op;
		if token1.kind == TokenKind::Operation && token2.kind == TokenKind::Name {
			let kind = get_op_kind(&token1.value);
			exp(kind != OperationKind::Unknown, "Unknown operation:", &token1)?;
			op = Operation {
				var: Arg {
					pos: token.pos,
					name: token.value,
				},
				pos: token1.pos,
				kind,
				args: vec![Arg {
					name: token2.value,
					pos: token2.pos,
				}],
			};
		} else if token1.kind == TokenKind::Name && token2.kind == TokenKind::Operation {
			let token3 = self.tokens.next().ok_or(eof())?;
			exp(token3.kind == TokenKind::Name, "Expected name, got:", &token3)?;

			let kind = get_op_kind(&token2.value);
			exp(kind != OperationKind::Unknown, "Unknown operation:", &token2)?;

			op = Operation {
				var: Arg {
					pos: token.pos,
					name: token.value,
				},
				pos: token2.pos,
				kind,
				args: vec![
					Arg {
						name: token1.value,
						pos: token1.pos,
					},
					Arg {
						name: token3.value,
						pos: token3.pos,
					},
				],
			};
		} else {
			return Err(exp(false, "Expected operation or a name, got:", &token1).unwrap_err());
		}

		let eol = self.tokens.next().ok_or(eof())?;
		exp(eol.kind == TokenKind::Sign && eol.value == ";", "Expected sign ';', got: ", &eol)?;

		Ok(op)
	}

	fn parse_operations(&mut self) -> Result<(Vec<Operation>, Arg), Error> {
		let mut ops = Vec::new();
		while let Some(token) = self.tokens.peek() {
			if token.kind == TokenKind::Keyword && token.value == "out" {
				break;
			}

			let lett = self.tokens.next().ok_or(eof())?;
			exp(lett.kind == TokenKind::Keyword && lett.value == "let", "Expected 'let' keyword, got: ", &lett)?;

			let exp = self.parse_operation()?;

			ops.push(exp);
		}

		let out = self.parse_out()?;

		let token = self.tokens.next().ok_or(eof())?;
		exp(token.kind == TokenKind::Sign && token.value == "}", "Expected a sign '}}', got: ", &token)?;
		Ok((ops, out))
	}

	fn parse_func(&mut self) -> Result<Def, Error> {
		let name = self.tokens.next().ok_or(eof())?;
		exp(name.kind == TokenKind::Name, "Expected name, got: ", &name)?;

		let arg_token = self.tokens.next().ok_or(eof())?;
		let mut args = Vec::new();
		if arg_token.kind == TokenKind::Sign && arg_token.value == "(" {
			args = self.parse_args()?;
		} else if arg_token.kind == TokenKind::Name {
			args.push(Arg {
				name: arg_token.value.clone(),
				pos: arg_token.pos,
			});
		} else {
			exp(false, "Expected name or a sign '(', got: ", &arg_token)?;
		}

		let cb_token = self.tokens.next().ok_or(eof())?;
		exp(
			cb_token.kind == TokenKind::Sign && cb_token.value == "{",
			"Expected sign '{{', got: ",
			&cb_token,
		)?;

		let (ops, out) = self.parse_operations()?;

		Ok(Def::Function(Function {
			name: name.value,
			pos: name.pos,
			params: args,
			out,
			body: ops,
		}))
	}

	fn parse_assignment(&mut self) -> Result<TestbenchAssignment, Error> {
		let token = self.tokens.next().ok_or(eof())?;
		exp(token.kind == TokenKind::Name, "Expected name , got: ", &token)?;
		let equal = self.tokens.next().ok_or(eof())?;
		exp(equal.kind == TokenKind::Sign && equal.value == "=", "Expected sign '=', got: ", &equal)?;

		let bool_token = self.tokens.next().ok_or(eof())?;
		exp(bool_token.kind == TokenKind::Value, "Expected value, got: ", &bool_token)?;

		let eol = self.tokens.next().ok_or(eof())?;
		exp(eol.kind == TokenKind::Sign && eol.value == ";", "Expected sign ';', got: ", &eol)?;

		let bool_value = bool_token.value.parse::<bool>();
		exp(bool_value.is_ok(), "Failed to parse bool, got: ", &bool_token)?;
		let bool_value = bool_value.unwrap();

		Ok(TestbenchAssignment {
			name: token.value,
			pos: token.pos,
			value: bool_value,
		})
	}

	fn parse_breakpoint(&mut self) -> Result<Breakpoint, Error> {
		let token = self.tokens.next().ok_or(eof())?;
		exp(token.kind == TokenKind::Sign, "Expected sign, got: ", &token)?;

		let time = self.tokens.next().ok_or(eof())?;
		exp(time.kind == TokenKind::Value, "Expected value, got: ", &time)?;

		let mut assignments = Vec::new();
		while let Some(token) = self.tokens.peek() {
			if token.kind != TokenKind::Name {
				break;
			}

			let ass = self.parse_assignment()?;
			assignments.push(ass);
		}

		let kind = get_breakpoint_kind(&token.value);
		exp(kind != BreakpointKind::Unknown, "Unknown breakpoint kind: ", &token)?;
		let parsed_time = time.value.parse::<u32>();
		exp(parsed_time.is_ok(), "Failed to parse integer, got: ", &time)?;
		let parsed_time = parsed_time.unwrap();

		Ok(Breakpoint {
			name: token.value,
			pos: token.pos,
			kind: kind,
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

		let token = self.tokens.next().ok_or(eof())?;
		exp(token.kind == TokenKind::Sign && token.value == "}", "Expected a sign '}}', got: ", &token)?;
		Ok(breakpoints)
	}

	fn parse_test(&mut self) -> Result<Def, Error> {
		let name = self.tokens.next().ok_or(eof())?;
		exp(name.kind == TokenKind::Name, "Expected name, got: ", &name)?;

		let arg_token = self.tokens.next().ok_or(eof())?;
		let mut args = Vec::new();
		if arg_token.kind == TokenKind::Sign && arg_token.value == "(" {
			args = self.parse_args()?;
		} else if arg_token.kind == TokenKind::Name {
			args.push(Arg {
				name: arg_token.value,
				pos: arg_token.pos,
			});
		} else {
			exp(false, "Expected name or a sign '(', got: ", &arg_token)?;
		}

		let breakpoints;
		let cb_token = self.tokens.next().ok_or(eof())?;
		if cb_token.kind == TokenKind::Sign && cb_token.value == "{" {
			breakpoints = self.parse_breakpoints()?;
		} else {
			return Err(exp(false, "Expected sign '{{', got: ", &cb_token).unwrap_err());
		}

		Ok(Def::Test(Test {
			name: name.value,
			pos: name.pos,
			params: args,
			body: breakpoints,
		}))
	}
}

impl<'a> Iterator for ParserIter<'a> {
	type Item = Result<Def, Error>;

	fn next(&mut self) -> Option<Result<Def, Error>> {
		while let Some(token) = self.tokens.next() {
			if token.kind != TokenKind::Keyword || (token.value != "func" && token.value != "test") {
				return Some(Err(err(
					format!("Expected keyword 'func' or 'test', found: '{}'.", token.value),
					(token.pos, token.value.len()),
				)));
			}
			if token.value == "func" {
				return Some(self.parse_func());
			} else {
				return Some(self.parse_test());
			};
		}
		None
	}
}
