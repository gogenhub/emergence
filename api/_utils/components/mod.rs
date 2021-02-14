pub mod gene;

pub use gene::Gene;
use serde::Serialize;

#[derive(Serialize, Debug, Clone)]
pub enum Component {
	Gene(Gene),
}
