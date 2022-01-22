use super::internal::*;
use lewton::inside_ogg::OggStreamReader;

pub fn decode_44khz_mono_f32(file: impl AsRef<Path>) -> Result<Vec<f32>> {
	decode_44khz_mono(file.as_ref()).map(|clip| convert_to_f32(&clip))
}

pub fn decode_44khz_mono(file: impl AsRef<Path>) -> Result<Vec<i16>> {
	let file = file.as_ref();
	let mut ogg = OggStreamReader::new(BufReader::new(File::open(file).map_err(|err| anyhow!("open {}: {}", file.to_string_lossy(), err))?))?;
	if ogg.ident_hdr.audio_sample_rate != 44100 {
		//return Err(anyhow!("sampling rate 44.1kHz expected, got {} Hz", ogg.ident_hdr.audio_sample_rate));
		eprintln!("ERROR: `{}`: sampling rate 44.1kHz expected, got {} Hz", file.to_string_lossy(), ogg.ident_hdr.audio_sample_rate);
	}

	if ogg.ident_hdr.audio_channels != 1 {
		return Err(anyhow!("mono audio expected, got {} channels", ogg.ident_hdr.audio_channels));
	}

	// Note: read interleaved assumes mono stream
	let mut decoded = Vec::new();
	while let Some(mut pck_samples) = ogg.read_dec_packet_itl()? {
		decoded.append(&mut pck_samples)
	}

	Ok(decoded)
}

pub fn convert_to_f32(clip: &[i16]) -> Vec<f32> {
	clip.iter().map(|&sample| (sample as f32) / (i16::MAX as f32)).collect()
}
