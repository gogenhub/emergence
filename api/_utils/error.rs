use crate::_utils::lexer;
use lexer::Token;
use serde::Serialize;

#[derive(Serialize, Debug)]
#[serde(tag = "kind", content = "data")]
pub enum Error {
	UnexpectedToken(String, usize, usize),
	AlreadyExists(String, usize, usize),
	NotFound(String, usize, usize),
	NotUsed(String, usize, usize),
	NotEnoughGenes,
	InvalidNumberOfArgs(String, usize, usize),
	EndOfFile,
}

impl Error {
	pub fn already_exists(condition: bool, token: &Token) -> Result<(), Self> {
		if condition {
			return Err(Self::AlreadyExists(
				token.value.to_string(),
				token.pos,
				token.value.len(),
			));
		}
		Ok(())
	}

	pub fn invalid_number_of_args(condition: bool, token: &Token) -> Result<(), Self> {
		if condition {
			return Err(Self::InvalidNumberOfArgs(
				token.value.to_string(),
				token.pos,
				token.value.len(),
			));
		}
		Ok(())
	}

	pub fn not_found(condition: bool, token: &Token) -> Result<(), Self> {
		if condition {
			return Err(Self::NotFound(
				token.value.to_string(),
				token.pos,
				token.value.len(),
			));
		}
		Ok(())
	}

	pub fn not_used(condition: bool, token: &Token) -> Result<(), Self> {
		if condition {
			return Err(Self::NotUsed(
				token.value.to_string(),
				token.pos,
				token.value.len(),
			));
		}
		Ok(())
	}
}
