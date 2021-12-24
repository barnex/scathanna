use super::internal::*;
use std::ops::Range;

pub fn integrate<F>(num_samples: Range<u32>, max_err: f64, f: &mut F) -> Color
where
	F: FnMut() -> Color,
{
	let mut acc = Color::BLACK;
	let mut stats = Stats::default();

	for _ in 0..num_samples.start {
		let color = f();
		acc += color;
		stats.add(srgb_intensity(color))
	}

	let max_err2 = sq(max_err);
	for _ in num_samples.start..num_samples.end {
		let color = f();
		acc += color;
		stats.add(srgb_intensity(color));
		if stats.sum != 0.0 && stats.error_squared() < max_err2 {
			break;
		}
	}

	acc / (stats.n() as f32)
}

fn srgb_intensity(c: Color) -> f64 {
	f32::sqrt((c.r() + c.g() + c.b()) / 3.0) as f64
}

#[derive(Default)]
struct Stats {
	n: u32,
	sum: f64,
	sumsq: f64,
}

impl Stats {
	pub fn add(&mut self, v: f64) {
		self.n += 1;
		self.sum += v;
		self.sumsq += v * v;
	}

	pub fn n(&self) -> u32 {
		self.n
	}

	#[cfg(test)]
	pub fn avg(&self) -> f64 {
		self.sum / (self.n as f64)
	}

	pub fn error_squared(&self) -> f64 {
		self.var() / (self.n as f64)
	}

	//#[cfg(test)]
	//pub fn std_error(&self) -> f64 {
	//	f64::sqrt(self.error_squared())
	//}

	#[cfg(test)]
	pub fn stddev(&self) -> f64 {
		f64::sqrt(self.var())
	}

	pub fn var(&self) -> f64 {
		let n = self.n as f64;
		(self.sumsq / n) - sq(self.sum / n)
	}
}

fn sq(v: f64) -> f64 {
	v * v
}

#[cfg(test)]
mod test {

	use super::*;

	// integrating a constant should return the constant.
	#[test]
	fn integrate_const() {
		fn sample() -> Color {
			Color::new(0.1, 0.2, 0.3)
		}

		assert_eq!(integrate(1..1, 0.0, &mut sample), Color::new(0.1, 0.2, 0.3));
	}

	#[test]
	fn integrate_noise() {
		let mut rng = rand::thread_rng();
		let mut sample = || Color::new(rng.gen(), rng.gen(), rng.gen());

		let have: vec3 = integrate(1000..1000, 0.0, &mut sample).into();
		let want: vec3 = Color::new(0.5, 0.5, 0.5).into();

		let diff = (have - want).len();
		if diff > 1.0 / f32::sqrt(1000.0) {
			panic!("have: {}, want: approx {}, diff: {} too large", have, want, diff)
		}
	}

	#[test]
	fn noise_early_return() {
		let mut rng = rand::thread_rng();
		let mut sample = || Color::new(rng.gen(), rng.gen(), rng.gen());

		let have: vec3 = integrate(30..1000_000_000, 0.01, &mut sample).into();
		let want: vec3 = Color::new(0.5, 0.5, 0.5).into();

		let diff = (have - want).len();
		if diff > 1.0 / 0.01 {
			panic!("have: {}, want: approx {}, diff: {} too large", have, want, diff)
		}
	}

	// https://en.wikipedia.org/wiki/Standard_deviation
	#[test]
	fn stats() {
		let mut stats = Stats::default();
		for v in [2, 4, 4, 4, 5, 5, 7, 9] {
			stats.add(v as f64)
		}
		assert_eq!(stats.avg(), 5.0);
		assert_eq!(stats.var(), 4.0);
		assert_eq!(stats.stddev(), 2.0);
	}
}
