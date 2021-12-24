/// Halton(b, i) returns the i'th element of the Halton series with base b.
/// i starts from 0.
/// The base b should be >= 2.
/// See https://en.wikipedia.org/wiki/Halton_sequence
pub fn halton(b: u32, i: u32) -> f32 {
	let mut i = i + 1; // actual series starts from 1
	let bf = b as f32;
	let mut f = 1.0;
	let mut r = 0.0;

	while i > 0 {
		f = f / bf;
		r = r + f * ((i % b) as f32);
		i = (f32::floor(i as f32 / bf)) as u32;
	}
	r
}

pub fn halton23(i: u32) -> (f32, f32) {
	(halton(2, i), halton(3, i))
}

pub fn halton23_scrambled(i: u32, rand: (f32, f32)) -> (f32, f32) {
	let (u, v) = halton23(i);
	((u + rand.0) % 1.0, (v + rand.1) % 1.0)
}

#[test]
pub fn test_halton() {
	assert_eq!(halton(2, 0), 0.5);
	assert_eq!(halton(2, 1), 0.25);
	assert_eq!(halton(2, 2), 0.75);
	assert_eq!(halton(2, 3), 0.125);
	assert_eq!(halton(2, 4), 0.625);
	assert_eq!(halton(2, 5), 0.375);
	assert_eq!(halton(2, 6), 0.875);
	assert_eq!(halton(2, 7), 0.0625);
	assert_eq!(halton(2, 8), 0.5625);
	assert_eq!(halton(2, 9), 0.3125);
}
