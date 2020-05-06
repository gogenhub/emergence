use regex::Regex;
use std::iter::{Enumerate, Peekable};
use std::str::Chars;

#[derive(Debug, PartialEq)]
pub enum TokenKind {
	Sign,
	Operation,
	Symbol,
	Keyword,
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

	fn scan_next(&mut self, first: Token, pattern: &str) -> Option<Token> {
		let rg = Regex::new(pattern).unwrap();
		let mut pk = self.chars.peek();
		if pk.is_none() {
			return None;
		}
		if !rg.is_match(&pk.unwrap().1.to_string()) {
			return Some(first);
		}
		let mut ret = first.1.clone();
		while let Some(p) = self.chars.next() {
			ret.push(p.1);
			pk = self.chars.peek();
			if pk.is_none() {
				break;
			}
			if !rg.is_match(&pk.unwrap().1.to_string()) {
				break;
			}
		}
		let symbol_rg = Regex::new("[a-zA-Z0-9]+").unwrap();
		match ret.as_str() {
			"let" | "fn" | "gene" => Some((TokenKind::Keyword, ret, first.2)),
			"~" | "~|" | "~&" | "~^" => Some((TokenKind::Operation, ret, first.2)),
			"->" => Some((TokenKind::Sign, ret, first.2)),
			c if symbol_rg.is_match(c) => Some((TokenKind::Symbol, ret, first.2)),
			_ => Some((TokenKind::Unknown, ret, first.2)),
		}
	}
}

impl<'a> Iterator for LexerIter<'a> {
	type Item = Token;

	fn next(&mut self) -> Option<Self::Item> {
		while let Some(current) = self.chars.next() {
			let chars = Regex::new("[a-zA-Z]").unwrap();
			let res = match current.1 {
				'\n' | '\t' | ' ' => None,
				'(' | ')' | '{' | '}' | ',' | ';' | '=' | ':' => {
					Some((TokenKind::Sign, current.1.to_string(), current.0))
				}
				'|' | '&' | '^' => Some((TokenKind::Operation, current.1.to_string(), current.0)),
				'~' => self.scan_next(
					(TokenKind::Operation, current.1.to_string(), current.0),
					"[|&^]",
				),
				'-' => self.scan_next((TokenKind::Unknown, current.1.to_string(), current.0), ">"),
				c if chars.is_match(&current.1.to_string()) => {
					self.scan_next((TokenKind::Symbol, c.to_string(), current.0), "[a-zA-Z0-9]")
				}
				_ => Some((TokenKind::Unknown, current.1.to_string(), current.0)),
			};
			if res.is_some() {
				return res;
			}
		}
		None
	}
}
