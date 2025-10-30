pub struct Builder {
	next_id: u32,
}

impl Builder {
	pub fn new() -> Self {
		Self { next_id: 0 }
	}
	/// Returns the next unused id
	pub fn id(&mut self) -> u32 {
		let result = self.next_id;
		self.next_id += 1;
		result
	}
}
