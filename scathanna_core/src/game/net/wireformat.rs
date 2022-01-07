use super::internal::*;

const MAGIC: u64 = 0xff53434154480002;

pub fn serialize_into<W, T>(mut w: W, msg: &T) -> Result<()>
where
	T: Serialize + Send + 'static,
	W: Write,
{
	bincode::serialize_into(&mut w, &MAGIC)?;
	bincode::serialize_into(&mut w, msg)?;
	Ok(())
}

pub fn deserialize_from<R, T>(mut r: R) -> Result<T>
where
	T: DeserializeOwned + Send + 'static,
	R: Read,
{
	let magic: u64 = bincode::deserialize_from(&mut r)?;
	if magic != MAGIC {
		return err(format!("server-client version mismatch: want {:x}, got {:x}", MAGIC, magic));
	}
	Ok(bincode::deserialize_from(&mut r)?)
}
