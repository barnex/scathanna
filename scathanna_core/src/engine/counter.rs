use std::cell::Cell;

#[derive(Default)]
pub struct Counter(Cell<u64>);

impl Counter {
	pub fn new() -> Self {
		Self::default()
	}

	pub fn inc(&self) {
		self.add(1)
	}

	pub fn add(&self, rhs: u64) {
		self.0.set(self.0.get() + rhs)
	}

	pub fn take(&self) -> u64 {
		self.0.take()
	}
}
