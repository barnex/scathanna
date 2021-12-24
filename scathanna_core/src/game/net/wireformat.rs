use super::internal::*;

pub fn serialize<T>(msg: &T) -> Bytes
where
	T: Serialize + Send + 'static,
{
	Bytes::new(bincode::serialize(msg).unwrap())
}

pub fn serialize_into<W, T>(w: &mut W, msg: &T) -> std::result::Result<(), Box<bincode::ErrorKind>>
where
	T: Serialize + Send + 'static,
	W: Write,
{
	bincode::serialize_into(w, msg)
}

pub fn deserialize_from<R, T>(r: &mut R) -> std::result::Result<T, Box<bincode::ErrorKind>>
where
	T: DeserializeOwned + Send + 'static,
	R: Read,
{
	bincode::deserialize_from(r)
}
