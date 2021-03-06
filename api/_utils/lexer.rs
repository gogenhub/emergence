use regex::Regex;
use std::{
	fmt::{Display, Formatter, Result},
	iter::{Enumerate, Peekable},
	str::Chars,
};

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum TokenKind {
	Sign,
	Operation,
	Name,
	Keyword,
	Value,
	Unknown,
}

impl Display for TokenKind {
	fn fmt(&self, f: &mut Formatter) -> Result {
		write!(f, "{:?}", self)
	}
}

#[derive(Debug, Clone)]
pub struct Token {
	pub kind: TokenKind,
	pub value: String,
	pub pos: usize,
}

pub struct LexerIter<'a> {
	chars: Peekable<Enumerate<Chars<'a>>>,
}

impl<'a> LexerIter<'a> {
	pub fn new(text: Chars<'a>) -> Self {
		Self {
			chars: text.enumerate().peekable(),
		}
	}

	fn scan_next(&mut self, pattern: &str) -> String {
		let rg = Regex::new(pattern).unwrap();
		let mut ret = String::new();
		let (_, c) = self.chars.next().unwrap();
		ret.push(c);
		while let Some((_, ch)) = self.chars.peek() {
			if !rg.is_match(&ch.to_string()) {
				return ret;
			}
			let (_, c) = self.chars.next().unwrap();
			ret.push(c);
		}
		ret
	}
}

impl<'a> Iterator for LexerIter<'a> {
	type Item = Token;

	fn next(&mut self) -> Option<Self::Item> {
		let chars = Regex::new("[a-zA-Z]").unwrap();
		let numbers = Regex::new("[0-9]").unwrap();
		while let Some((pos, ch)) = self.chars.peek().cloned() {
			let group = match ch {
				'~' => self.scan_next("[|&^]"),
				'-' => self.scan_next(">"),
				c if chars.is_match(&c.to_string()) => self.scan_next("[a-zA-Z0-9]"),
				c if numbers.is_match(&c.to_string()) => self.scan_next("[0-9]"),
				c => {
					self.chars.next();
					c.to_string()
				}
			};
			if ["\n", "\t", " "].contains(&group.as_str()) {
				continue;
			}
			let res = match group.as_str() {
				"out" | "in" | "let" | "impl" | "test" | "for" | "mod" | "env" => Token {
					kind: TokenKind::Keyword,
					value: group.to_string(),
					pos,
				},
				"(" | ")" | "{" | "}" | "," | ";" | "=" | "@" => Token {
					kind: TokenKind::Sign,
					value: group.to_string(),
					pos,
				},
				"not" | "nor" => Token {
					kind: TokenKind::Operation,
					value: group.to_string(),
					pos,
				},
				"true" | "false" => Token {
					kind: TokenKind::Value,
					value: group.to_string(),
					pos,
				},
				c if chars.is_match(&c.to_string()) => Token {
					kind: TokenKind::Name,
					value: group.to_string(),
					pos,
				},
				c if numbers.is_match(&c.to_string()) => Token {
					kind: TokenKind::Value,
					value: group.to_string(),
					pos,
				},
				_ => Token {
					kind: TokenKind::Unknown,
					value: group.to_string(),
					pos,
				},
			};
			return Some(res);
		}
		None
	}
}
