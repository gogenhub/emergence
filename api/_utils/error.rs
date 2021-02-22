use crate::_utils::lexer;
use lexer::Token;
use serde::Serialize;

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
