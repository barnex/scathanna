#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum Chan {
	L = 0,
	R = 1,
}

pub struct RingBuffer {
	channels: [Vec<f32>; 2], // left and right channels
	cursor: usize,
}

impl RingBuffer {
	pub fn new(samples: usize) -> Self {
		Self {
			channels: [vec![0.0; samples], vec![0.0; samples]],
			cursor: 0,
		}
	}

	pub fn play_raw_stereo_itl(&mut self, mut src: impl Iterator<Item = f32>) {
		let mut cursor = self.cursor;
		while let Some(mut sample) = src.next() {
			self.channels[0][cursor] += sample;
			sample = src.next().unwrap_or_default();
			self.channels[1][cursor] += sample;
			cursor = self.advance(cursor);
		}
	}

	pub fn play_raw_mono(&mut self, src: impl Iterator<Item = f32>) {
		let mut cursor = self.cursor;
		for sample in src {
			self.channels[0][cursor] += sample;
			self.channels[1][cursor] += sample;
			cursor = self.advance(cursor);
		}
	}

	pub fn next_sample(&mut self) -> (f32, f32) {
		let sample = (self.channels[0][self.cursor], self.channels[1][self.cursor]);
		self.channels[0][self.cursor] = 0.0;
		self.channels[1][self.cursor] = 0.0;
		self.cursor = self.advance(self.cursor);
		sample
	}

	#[must_use]
	fn advance(&self, cursor: usize) -> usize {
		let cursor = cursor + 1;
		if cursor == self.channels[0].len() {
			0
		} else {
			cursor
		}
	}
}
