use once_cell::sync::Lazy;
use std::sync::Mutex;
use utils::{assigner, builder, lexer, parser};

use assigner::Assigner;
use builder::LogicCircutBuilder;
use lexer::LexerIter;
use now_lambda::{error::NowError, lambda, IntoResponse, Request, Response};
use parser::ParserIter;
use std::error::Error;

static ASSIGNER: Lazy<Mutex<Assigner>> = Lazy::new(|| Mutex::new(Assigner::new()));

fn handler(req: Request) -> Result<impl IntoResponse, NowError> {
	let emergence = req.body.unwrap().to_string();
	let tokens = LexerIter::new(emergence.chars());
	let ps = ParserIter::new(tokens);
	let mut lc = LogicCircutBuilder::new(ps);
	let (lc, warnings) = lc.build_logic_circut().unwrap();
	let mut ass = ASSIGNER.lock().unwrap();
	if !ass.loaded {
		ass.load();
	}
	let res = ass.assign_gates(lc).unwrap();

	let response = Response::builder()
		.status(StatusCode::OK)
		.header("Content-Type", "text/plain")
		.body("user endpoint")
		.expect("Internal Server Error");

	Ok(response)
}

fn main() -> Result<(), Box<dyn Error>> {
	Ok(lambda!(handler))
}
