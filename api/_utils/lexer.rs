use regex::Regex;
use std::iter::{Enumerate, Peekable};
use std::str::Chars;

#[derive(Debug, PartialEq)]
pub enum TokenKind {
	Sign,
	Operation,
	Symbol,
	Keyword,
	Number,
	Bool,
	Unknown,
}

type Token = (TokenKind, String, usize);

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
			let res = match group.as_str() {
				"\n" | "\t" | " " => None,
				"let" | "func" | "event" | "test" => {
					Some((TokenKind::Keyword, group.to_owned(), pos))
				}
				"(" | ")" | "{" | "}" | "," | ";" | "=" | "->" | "@" => {
					Some((TokenKind::Sign, group.to_owned(), pos))
				}
				"~" | "~|" | "~&" | "~^" | "|" | "&" | "^" => {
					Some((TokenKind::Operation, group.to_owned(), pos))
				}
				"true" | "false" => Some((TokenKind::Bool, group.to_owned(), pos)),
				c if chars.is_match(&c.to_string()) => {
					Some((TokenKind::Symbol, group.to_owned(), pos))
				}
				c if numbers.is_match(&c.to_string()) => {
					Some((TokenKind::Number, group.to_owned(), pos))
				}
				_ => Some((TokenKind::Unknown, group.to_owned(), pos)),
			};
			if res.is_some() {
				return res;
			}
		}
		None
	}
}
