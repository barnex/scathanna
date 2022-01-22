use super::internal::*;
use super::*;

use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{Device, StreamConfig};

use std::sync::Arc;
use std::sync::Mutex;
use std::thread;
use std::time::Duration;

use spatial_filter::*;

pub struct Mixer {
	ringbuf: Arc<Mutex<RingBuffer>>,
}

const SAMPLE_RATE: usize = 44100;

impl Mixer {
	pub fn new(size: Duration) -> Result<Self> {
		let samples = (SAMPLE_RATE / 1000) * size.as_millis() as usize;
		let ringbuf = Arc::new(Mutex::new(RingBuffer::new(samples)));

		{
			let ringbuf = ringbuf.clone();
			thread::spawn(|| run(ringbuf).expect("error"));
		}

		Ok(Self { ringbuf })
	}

	const SAMPLING_RATE: f32 = 44100.0;

	pub fn play_raw_stereo_itl(&self, src: impl Iterator<Item = f32>) {
		let mut buf = self.ringbuf.lock().unwrap();
		buf.play_raw_stereo_itl(src)
	}

	pub fn play_raw_mono(&self, src: impl Iterator<Item = f32>) {
		let mut buf = self.ringbuf.lock().unwrap();
		buf.play_raw_mono(src)
	}

	pub fn play_spatial(&self, azimuth: f32, volume: f32, src: impl Iterator<Item = f32> + Clone) {
		self.play_raw_stereo_itl(interleave(duplex_v2(src, Self::SAMPLING_RATE, azimuth, volume)));
	}
}

fn run(buffer: Arc<Mutex<RingBuffer>>) -> Result<()> {
	let device = cpal::default_host().default_output_device().ok_or(anyhow!("No default audio device"))?;

	let config = device.default_output_config()?;
	println!("Output device: {}", device.name()?);
	println!("Default output config: {:?}", config);

	match config.sample_format() {
		cpal::SampleFormat::F32 => run_generic::<f32>(&device, &config.into(), buffer),
		cpal::SampleFormat::I16 => run_generic::<i16>(&device, &config.into(), buffer),
		cpal::SampleFormat::U16 => run_generic::<u16>(&device, &config.into(), buffer),
	}
}

fn run_generic<T: cpal::Sample>(device: &Device, config: &StreamConfig, buffer: Arc<Mutex<RingBuffer>>) -> Result<()> {
	let stream = device.build_output_stream(
		config,
		move |data: &mut [T], _: &cpal::OutputCallbackInfo| {
			let mut buffer = buffer.lock().unwrap();
			copy_stero_itl(data, &mut || buffer.next_sample())
		},
		move |err| eprintln!("an error occurred on stream: {}", err),
	)?;
	stream.play()?;

	loop {
		thread::park()
	}
}

fn copy_stero_itl<T: cpal::Sample>(dst: &mut [T], next_sample: &mut dyn FnMut() -> (f32, f32)) {
	for i in (0..dst.len()).step_by(2) {
		let sample = next_sample();
		dst[i] = cpal::Sample::from::<f32>(&sample.0);
		dst[i + 1] = cpal::Sample::from::<f32>(&sample.1);
	}
}
