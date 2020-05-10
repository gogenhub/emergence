use crate::_utils::{helpers, lexer};
use helpers::{eof, syntax_err, Error};
use lexer::{LexerIter, TokenKind};
use std::iter::Peekable;

#[derive(Debug, Clone)]
pub struct Arg {
	pub name: String,
	pub pos: usize,
}

#[derive(Debug, PartialEq)]
pub enum OperationKind {
	Call,
	Operation,
}

#[derive(Debug)]
pub struct Operation {
	pub pos: usize,
	pub kind: OperationKind,
	pub name: String,
	pub args: Vec<Arg>,
}

#[derive(Debug, PartialEq)]
pub enum ExpressionKind {
	Assign,
	Return,
}

#[derive(Debug)]
pub struct Expression {
	pub kind: ExpressionKind,
	pub var: Arg,
	pub op: Operation,
}

#[derive(Debug, PartialEq)]
pub enum DefKind {
	Gene,
	Function,
}

#[derive(Debug)]
pub struct Def {
	pub kind: DefKind,
	pub name: String,
	pub pos: usize,
	pub params: Vec<Arg>,
	pub ret: Arg,
	pub body: Vec<Expression>,
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

	fn parse_args(&mut self) -> Result<Vec<Arg>, Error> {
		let mut args = Vec::new();
		while let Some(token) = self.tokens.next() {
			let (name_type, name, name_pos) = token;
			let name_len = name.len();
			if name_type != TokenKind::Symbol {
				return Err(syntax_err(
					format!("Expected symbol, got: '{}'.", name),
					name_pos,
					name_len,
				));
			}
			args.push(Arg {
				name: name,
				pos: name_pos,
			});
			let (sep_type, sep, sep_pos) = self.tokens.next().ok_or(eof())?;
			if sep_type != TokenKind::Sign {
				return Err(syntax_err(
					format!("Expected token ',' or ')', got: '{}'.", sep),
					sep_pos,
					sep.len(),
				));
			}
			if sep != "," && sep != ")" {
				return Err(syntax_err(
					format!("Expected token ',' or ')', got: '{}'.", sep),
					sep_pos,
					sep.len(),
				));
			}

			if sep == ")" {
				break;
			}
		}
		Ok(args)
	}

	fn parse_operation(&mut self) -> Result<Operation, Error> {
		let (first_token_type, first_token, first_token_pos) = self.tokens.next().ok_or(eof())?;
		if (first_token_type != TokenKind::Symbol) && (first_token_type != TokenKind::Operation) {
			return Err(syntax_err(
				format!("Expected symbol or operation, got: '{}'.", first_token),
				first_token_pos,
				first_token.len(),
			));
		}
		let (second_token_type, second_token, second_token_pos) = self.tokens.next().ok_or(eof())?;
		if first_token_type == TokenKind::Operation && second_token_type != TokenKind::Symbol {
			return Err(syntax_err(
				format!("Expected symbol, got: '{}'.", second_token),
				second_token_pos,
				second_token.len(),
			));
		}

		let res = match (first_token_type, second_token_type) {
			(TokenKind::Operation, TokenKind::Symbol) => Ok(Operation {
				pos: first_token_pos,
				kind: OperationKind::Operation,
				name: first_token,
				args: vec![Arg {
					name: second_token.clone(),
					pos: second_token_pos,
				}],
			}),
			(TokenKind::Symbol, TokenKind::Operation) => {
				let (third_token_type, third_token, third_token_pos) =
					self.tokens.next().ok_or(eof())?;
				if third_token_type != TokenKind::Symbol {
					return Err(syntax_err(
						format!("Expected symbol, got: '{}'.", third_token),
						third_token_pos,
						third_token.len(),
					));
				}

				let args = vec![
					Arg {
						name: first_token,
						pos: first_token_pos,
					},
					Arg {
						name: third_token,
						pos: third_token_pos,
					},
				];
				Ok(Operation {
					pos: second_token_pos,
					kind: OperationKind::Operation,
					name: second_token.clone(),
					args: args,
				})
			}
			(TokenKind::Symbol, TokenKind::Sign) => {
				if second_token != "(" {
					return Err(syntax_err(
						format!("Expected sign '(', got: '{}'.", second_token),
						second_token_pos,
						second_token.len(),
					));
				}

				let args = self.parse_args()?;
				Ok(Operation {
					pos: first_token_pos,
					kind: OperationKind::Call,
					name: first_token,
					args: args,
				})
			}
			_ => {
				return Err(syntax_err(
					format!("Expected symbol or operation, got: '{}'.", second_token),
					second_token_pos,
					second_token.len(),
				))
			}
		};
		let (eol_token_type, eol_token, eol_pos) = self.tokens.next().ok_or(eof())?;
		if eol_token_type != TokenKind::Sign || eol_token != ";" {
			return Err(syntax_err(
				format!("Expected sign ';', got: '{}'.", eol_token),
				eol_pos,
				eol_token.len(),
			));
		}
		res
	}

	fn parse_expression(&mut self) -> Result<Expression, Error> {
		let (token_type, token, token_pos) = self.tokens.next().ok_or(eof())?;
		let token_len = token.len();
		match (token_type, token) {
			(TokenKind::Keyword, c) if c == "let" => {
				let (name_token_type, name, name_pos) = self.tokens.next().ok_or(eof())?;
				if name_token_type != TokenKind::Symbol {
					return Err(syntax_err(
						format!("Expected symbol, got: '{}'.", name),
						name_pos,
						name.len(),
					));
				}
				let (equal_token_type, equal, equal_pos) = self.tokens.next().ok_or(eof())?;
				if equal_token_type != TokenKind::Sign || equal != "=" {
					return Err(syntax_err(
						format!("Expected sign '=' got: '{}'.", equal),
						equal_pos,
						equal.len(),
					));
				}
				let op = self.parse_operation()?;
				Ok(Expression {
					kind: ExpressionKind::Assign,
					var: Arg {
						name: name,
						pos: name_pos,
					},
					op: op,
				})
			}
			(TokenKind::Symbol, name) => {
				let (equal_token_type, equal, equal_pos) = self.tokens.next().ok_or(eof())?;
				if equal_token_type != TokenKind::Sign || equal != "=" {
					return Err(syntax_err(
						format!("Expected sign '=', got: '{}'.", equal),
						equal_pos,
						equal.len(),
					));
				}
				let op = self.parse_operation()?;
				Ok(Expression {
					kind: ExpressionKind::Return,
					var: Arg {
						name: name,
						pos: token_pos,
					},
					op: op,
				})
			}
			(_, c) => {
				return Err(syntax_err(
					format!("Expected symbol or keyword 'let', got: '{}'.", c),
					token_pos,
					token_len,
				));
			}
		}
	}

	fn parse_expressions(&mut self) -> Result<Vec<Expression>, Error> {
		let mut expressions = Vec::new();
		while let Some((kind, token, _)) = self.tokens.peek() {
			if *kind == TokenKind::Sign && token == "}" {
				break;
			}

			let exp = self.parse_expression()?;
			expressions.push(exp);
		}

		let (kind, token, pos) = self.tokens.next().ok_or(eof())?;
		if kind != TokenKind::Sign && token != "}" {
			return Err(syntax_err(
				format!("Expected a sign '}}', got: '{}'.", token),
				pos,
				token.len(),
			));
		}
		Ok(expressions)
	}

	fn parse_def(&mut self, def_kind: DefKind) -> Result<(String, Def), Error> {
		let (name_token_type, name, name_pos) = self.tokens.next().ok_or(eof())?;
		if name_token_type != TokenKind::Symbol {
			return Err(syntax_err(
				format!("Expected symbol, got: '{}'.", name),
				name_pos,
				name.len(),
			));
		}

		let (arg_token_type, arg_token, arg_token_pos) = self.tokens.next().ok_or(eof())?;
		let mut args = Vec::new();
		if arg_token_type == TokenKind::Sign && arg_token == "(" {
			args = self.parse_args()?;
		} else if arg_token_type == TokenKind::Symbol {
			args.push(Arg {
				name: arg_token.clone(),
				pos: arg_token_pos,
			});
		} else {
			return Err(syntax_err(
				format!("Expected symbol or list of symbols, got: '{}'.", arg_token),
				arg_token_pos,
				arg_token.len(),
			));
		}

		let (arrow_token_type, arrow_token, arrow_token_pos) = self.tokens.next().ok_or(eof())?;
		if arrow_token_type != TokenKind::Sign || arrow_token != "->" {
			return Err(syntax_err(
				format!("Expected sign '->', got: '{}'.", arrow_token),
				arrow_token_pos,
				arrow_token.len(),
			));
		}

		let (retr_token_type, retr_token, retr_token_pos) = self.tokens.next().ok_or(eof())?;
		let retr_token_len = retr_token.len();
		if retr_token_type != TokenKind::Symbol {
			return Err(syntax_err(
				format!("Expected symbol, got: '{}'.", retr_token),
				retr_token_pos,
				retr_token_len,
			));
		}

		let expressions;
		let (curly_braces_token_type, curly_braces_token, curly_braces_token_pos) =
			self.tokens.next().ok_or(eof())?;
		if curly_braces_token_type == TokenKind::Sign && curly_braces_token == "{" {
			expressions = self.parse_expressions()?;
		} else {
			return Err(syntax_err(
				format!("Expected sign '{{', got: '{}'.", curly_braces_token),
				curly_braces_token_pos,
				curly_braces_token.len(),
			));
		}

		Ok((
			name.to_owned(),
			Def {
				name: name,
				kind: def_kind,
				pos: name_pos,
				params: args,
				ret: Arg {
					name: retr_token,
					pos: retr_token_pos,
				},
				body: expressions,
			},
		))
	}
}

impl<'a> Iterator for ParserIter<'a> {
	type Item = Result<(String, Def), Error>;

	fn next(&mut self) -> Option<Result<(String, Def), Error>> {
		while let Some((token_kind, token, pos)) = self.tokens.next() {
			if token_kind != TokenKind::Keyword || (token != "fn" && token != "gene") {
				return Some(Err(syntax_err(
					format!("Expected token 'fn' or 'gene', found: '{}'.", token),
					pos,
					token.len(),
				)));
			}
			let kind = if token == "fn" {
				DefKind::Function
			} else {
				DefKind::Gene
			};
			return Some(self.parse_def(kind));
		}
		None
	}
}
