mod dna;
mod genetic_circuit;
mod logic_circuit;
mod parser;
mod utils;

use dna::Dna;
use genetic_circuit::GeneticCircuit;
use logic_circuit::builder::LogicCircuitBuilder;
use parser::{lexer::LexerIter, ParserIter};
use serde::Serialize;
use utils::error::Error;

#[derive(Serialize, Debug)]
pub struct CompileResult {
	gc: GeneticCircuit,
	dna: Dna,
}

pub fn compile(emergence: String) -> Result<CompileResult, Error> {
	let lx = LexerIter::new(emergence.chars());
	let prs = ParserIter::new(lx);
	let mut bld = LogicCircuitBuilder::new(prs);
	bld.build_parse_tree()?;
	let lc = bld.build_logic_circut();
	let mut gc = lc.fit_into_biological()?;
	gc.simulate(lc.testbench);
	gc.apply_rules();
	let dna = gc.into_dna();
	Ok(CompileResult { gc, dna })
}
