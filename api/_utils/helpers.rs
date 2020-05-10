use crate::_utils::{assigner, builder, parser};

use assigner::PromoterKind;
use builder::GateKind;
use parser::Arg;
use serde::Serialize;
use std::collections::HashMap;

#[derive(Serialize, Debug)]
pub enum ErrorKind {
	SyntaxError,
	CompileError,
	AssignError,
}

#[derive(Serialize, Debug)]
pub struct Error {
	pub kind: ErrorKind,
	pub message: String,
	pub pos: (usize, usize),
}

#[derive(Serialize, Debug)]
pub enum WarningKind {
	UnusedVar,
}

#[derive(Serialize, Debug)]
pub struct Warning {
	pub kind: WarningKind,
	pub message: String,
	pub pos: (usize, usize),
}

pub fn args_from_to(from: &Vec<Arg>, to: &Vec<Arg>) -> HashMap<String, String> {
	let map: HashMap<String, String> = from
		.iter()
		.zip(to.iter())
		.map(|(x, y)| (x.name.to_owned(), y.name.to_owned()))
		.collect();

	map
}

pub fn ret_from_to(from: &Arg, to: &Arg) -> HashMap<String, String> {
	let mut map = HashMap::new();
	map.insert(from.name.to_owned(), to.name.to_owned());
	map
}

pub fn map_hms(
	from: &HashMap<String, String>,
	to: &HashMap<String, String>,
	prefix: &str,
) -> HashMap<String, String> {
	let new_hm = from
		.iter()
		.map(|(k, v)| {
			(
				k.to_owned(),
				to.get(v).unwrap_or(&format!("{}{}", prefix, v)).to_owned(),
			)
		})
		.collect();
	new_hm
}

pub fn format_args_for_gate(
	from: &Vec<Arg>,
	args_map: &HashMap<String, String>,
	id: &str,
) -> Vec<String> {
	from.iter()
		.map(|x| {
			args_map
				.get(&x.name)
				.unwrap_or(&format!("{}{}", id, x.name))
				.to_owned()
		})
		.collect()
}

pub fn format_ret_for_gate(from: &Arg, rets_map: &HashMap<String, String>, id: &str) -> String {
	rets_map
		.get(&from.name)
		.unwrap_or(&format!("{}{}", id, from.name))
		.to_owned()
}

pub fn get_gate_kind(symbol: &str) -> GateKind {
	match symbol {
		"|" => GateKind::OR,
		"&" => GateKind::AND,
		"~" => GateKind::NOT,
		"~&" => GateKind::NAND,
		"~|" => GateKind::NOR,
		"~^" => GateKind::XOR,
		_ => GateKind::Unknown,
	}
}

pub fn get_promoter_kind(gate_kind: &GateKind) -> PromoterKind {
	match gate_kind {
		GateKind::NOT | GateKind::NOR => PromoterKind::Repressor,
		_ => PromoterKind::Unknown,
	}
}

pub fn compile_err(message: String, pos: (usize, usize)) -> Error {
	Error {
		kind: ErrorKind::CompileError,
		message: message,
		pos: pos,
	}
}

pub fn assign_err(message: String, pos: (usize, usize)) -> Error {
	Error {
		kind: ErrorKind::AssignError,
		message: message,
		pos: pos,
	}
}

pub fn uw(message: String, pos: (usize, usize)) -> Warning {
	Warning {
		kind: WarningKind::UnusedVar,
		message: message,
		pos: pos,
	}
}

pub fn eof() -> Error {
	Error {
		kind: ErrorKind::SyntaxError,
		message: "Unexpected end of file.".to_owned(),
		pos: (0, 0),
	}
}

pub fn syntax_err(message: String, from: usize, len: usize) -> Error {
	Error {
		kind: ErrorKind::SyntaxError,
		message: message,
		pos: (from, len),
	}
}
