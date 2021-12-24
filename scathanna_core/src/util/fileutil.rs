use super::internal::*;

use std::fs::File;
use std::io::BufReader;
use std::io::BufWriter;
use std::io::Read;
use std::io::Write;
use std::path::Path;

/// BufReader for reading file with more descriptive message on error.
pub fn open(file: &Path) -> Result<impl Read> {
	Ok(BufReader::new(File::open(file).map_err(|err| error(format!("open {:?}: {}", file, err)))?))
}

/// BufWriter for writing file with more descriptive message on error.
pub fn create(file: &Path) -> Result<impl Write> {
	Ok(BufWriter::new(File::create(file).map_err(|err| error(format!("open {:?}: {}", file, err)))?))
}

/// Attempt to prefix `file` with the executable's path.
/// Intended to find asset files relative to the executable. E.g.:
///
///   assets/maps =>  /path/to/binary/assets/maps
///
/// Return file unchanged if this fails (will still work when cd'ed in binary's directory).
/// Return the file unchanged if it is already absolute.
pub fn abs_path(file: &Path) -> PathBuf {
	if file.is_absolute() || file.exists() {
		return file.to_owned();
	}

	match std::env::current_exe() {
		Err(e) => {
			eprintln!("ERROR getting executable path: {}.\nPlease run this command from the directory containing `assets/`", e);
			file.to_owned()
		}
		Ok(exe) => match exe.parent() {
			None => file.to_owned(),
			Some(dir) => dir.join(file),
		},
	}
}
