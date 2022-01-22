use std::iter;

// Ear distance in meters (https://en.wikipedia.org/wiki/Sound_localization).
const EAR_DISTANCE: f32 = 0.215;

// Speed of sound in m/s (https://en.wikipedia.org/wiki/Speed_of_sound).
const SOUND_SPEED: f32 = 343.0;

pub trait Signal {}

impl<T> Signal for T where T: Iterator<Item = f32> + Clone {}

use std::f32::consts::PI;

pub fn duplex_v2(src: impl Iterator<Item = f32> + Clone, sampling_rate: f32, azimuth: f32, volume: f32) -> impl Iterator<Item = [f32; 2]> + Clone {
	let mut azimuth = azimuth;
	while azimuth > PI {
		azimuth = azimuth - 2.0 * PI;
	}
	while azimuth < -PI {
		azimuth = azimuth + 2.0 * PI;
	}

	let src = src.map(move |v| v * volume);
	let cutoff = if azimuth.abs() < PI / 2.0 { 0.0 } else { -f32::cos(azimuth) };
	let cutoff = cutoff.sqrt();
	let cutoff_freq = (cutoff * 1000.0) + ((1.0 - cutoff) * 40000.0);
	//println!("azimuth {}, cutoff {}, freq {}", azimuth * 180.0 / PI, cutoff, cutoff_freq);
	let muffled = low_pass(src, sampling_rate, cutoff_freq);
	duplex_v1(muffled, sampling_rate, azimuth)
}

pub fn duplex_v1(src: impl Iterator<Item = f32> + Clone, sampling_rate: f32, azimuth: f32) -> impl Iterator<Item = [f32; 2]> + Clone {
	let muffled = low_pass(src.clone(), sampling_rate, 1200.0);

	let l = (1.0 + f32::sin(azimuth)) * (0.5);
	let left_falloff = src.clone().zip(muffled.clone()).map(move |(orig, low)| l * orig + (1.0 - l) * low);

	let r = (1.0 - f32::sin(azimuth)) * (0.5);
	let right_falloff = src.clone().zip(muffled.clone()).map(move |(orig, low)| r * orig + (1.0 - r) * low);

	let left = delay(left_falloff, iidt_left(sampling_rate, azimuth));
	let right = delay(right_falloff, iidt_right(sampling_rate, azimuth));

	left.zip(right).map(|(l, r)| [l, r])
}

pub fn spatial_falloff(src: impl Iterator<Item = f32> + Clone, sampling_rate: f32, azimuth: f32) -> impl Iterator<Item = [f32; 2]> + Clone {
	let muffled = low_pass(src.clone(), sampling_rate, 1000.0);

	let l = (1.0 + f32::sin(azimuth)) * (0.5);
	let r = (1.0 - f32::sin(azimuth)) * (0.5);

	let left = src.clone().zip(muffled.clone()).map(move |(orig, low)| l * orig + (1.0 - l) * low);
	let right = src.clone().zip(muffled.clone()).map(move |(orig, low)| r * orig + (1.0 - r) * low);

	left.zip(right).map(|(l, r)| [l, r])
}

pub fn spatial_phase(src: impl Iterator<Item = f32> + Clone, sampling_rate: f32, azimuth: f32) -> impl Iterator<Item = [f32; 2]> + Clone {
	let left = delay(src.clone(), iidt_left(sampling_rate, azimuth));
	let right = delay(src, iidt_right(sampling_rate, azimuth));
	left.zip(right).map(|(l, r)| [l, r])
}

pub fn interleave(src: impl Iterator<Item = [f32; 2]> + Clone) -> impl Iterator<Item = f32> + Clone {
	src.flatten()
}

fn iidt_left(sampling_rate: f32, azimuth: f32) -> usize {
	let delay_time = (1.0 - f32::sin(azimuth)) * (EAR_DISTANCE / SOUND_SPEED);
	(delay_time * sampling_rate) as usize
}

fn iidt_right(sampling_rate: f32, azimuth: f32) -> usize {
	iidt_left(sampling_rate, -azimuth)
}

pub fn delay(src: impl Iterator<Item = f32> + Clone, samples: usize) -> impl Iterator<Item = f32> + Clone {
	iter::repeat(0.0).take(samples).chain(src)
}

pub fn low_pass(src: impl Iterator<Item = f32> + Clone, sampling_rate: f32, cutoff_freq: f32) -> impl Iterator<Item = f32> + Clone {
	// https://en.wikipedia.org/wiki/Low-pass_filter
	let dt = 1.0 / sampling_rate;
	let alpha = ((2.0 * PI * dt * cutoff_freq) / (2.0 * PI * dt * cutoff_freq + 1.0)) as f32;
	let mut acc = 0.0;

	src.map(move |sample| {
		acc = (1.0 - alpha) * acc + alpha * sample;
		acc
	})
}
