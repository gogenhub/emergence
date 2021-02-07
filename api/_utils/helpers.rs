use crate::_utils::{data, devices, lexer};
use chrono::Utc;
use data::PartKind;
use devices::GateKind;
use lexer::Token;
use serde::Serialize;
use std::collections::HashMap;
use std::ops::{Add, Div, Mul, Sub};

#[derive(Serialize, Debug)]
#[serde(tag = "kind", content = "pos")]
pub enum Error {
	UnexpectedToken(usize, usize),
	AlreadyExists(usize, usize),
	NotFound(usize, usize),
	NotUsed(usize, usize),
	NotEnoughGates,
	InvalidNumberOfArgs(usize, usize),
	EndOfFile,
}

impl Error {
	pub fn already_exists(condition: bool, token: &Token) -> Result<(), Self> {
		if condition {
			return Err(Self::AlreadyExists(token.pos, token.value.len()));
		}
		Ok(())
	}

	pub fn invalid_number_of_args(condition: bool, token: &Token) -> Result<(), Self> {
		if condition {
			return Err(Self::InvalidNumberOfArgs(token.pos, token.value.len()));
		}
		Ok(())
	}

	pub fn not_found(condition: bool, token: &Token) -> Result<(), Self> {
		if condition {
			return Err(Self::NotFound(token.pos, token.value.len()));
		}
		Ok(())
	}

	pub fn not_used(condition: bool, token: &Token) -> Result<(), Self> {
		if condition {
			return Err(Self::NotUsed(token.pos, token.value.len()));
		}
		Ok(())
	}
}

pub fn args_from_to(from: &Vec<Token>, to: &Vec<Token>) -> HashMap<String, String> {
	let map: HashMap<String, String> = from
		.iter()
		.zip(to.iter())
		.map(|(x, y)| (x.value.to_owned(), y.value.to_owned()))
		.collect();

	map
}

pub fn get_gate_kind(token: &Token) -> Result<GateKind, Error> {
	match token.value.as_str() {
		"not" => Ok(GateKind::Not),
		"nor" => Ok(GateKind::Nor),
		_ => Err(Error::UnexpectedToken(token.pos, token.value.len())),
	}
}

pub fn map<A>(num: A, in_min: A, in_max: A, out_min: A, out_max: A) -> A
where
	A: Add<Output = A> + Mul<Output = A> + Sub<Output = A> + Div<Output = A> + Copy,
{
	(num - in_min) * (out_max - out_min) / (in_max - in_min) + out_min
}

pub fn make_plasmid_dna(seq: &str) -> String {
	return "ORIGIN\n".to_owned()
		+ &seq
			.as_bytes()
			.chunks(60)
			.enumerate()
			.map(|(i, chunk)| {
				let ch: Vec<String> = chunk
					.chunks(10)
					.map(|x| {
						let parsed: String = std::str::from_utf8(x).unwrap().to_owned();
						parsed
					})
					.collect();
				let index_fmt = format!("{:>9}", (i * 60) + 1);
				format!("{} {}", index_fmt, ch.join(" "))
			})
			.collect::<Vec<String>>()
			.join("\n");
}

pub fn make_plasmid_title(name: &str, len: usize) -> String {
	format!(
		"LOCUS      {}      {} bp ds-DNA      circular      {}\nFEATURES             Location/Qualifiers\n",
		name,
		len,
		Utc::today().format("%e-%b-%Y")
	)
}

pub fn make_plasmid_part(kind: &PartKind, start: usize, end: usize, label: &str, color: &str) -> String {
	return format!("     {:<16}{}..{}\n", format!("{:?}", kind), start + 1, end)
		+ &format!("                     /label={}\n", label)
		+ &format!("                     /ApEinfo_fwdcolor={}\n", color);
}
