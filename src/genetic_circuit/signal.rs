use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Signal {
	pub name: String,
	pub promoter: String,
	pub rpu_off: f64,
	pub rpu_on: f64,
}

impl Signal {
	pub fn name(&self) -> String {
		self.name.to_string()
	}

	pub fn promoter(&self) -> String {
		self.promoter.to_string()
	}

	pub fn group(&self) -> String {
		self.name.to_string()
	}
}
