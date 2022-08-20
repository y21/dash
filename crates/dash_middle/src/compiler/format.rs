use super::CompileResult;

const BYTECODE_VERSION: u32 = 2;

pub fn serialize(cr: CompileResult) -> bincode::Result<Vec<u8>> {
    let mut buffer = BYTECODE_VERSION.to_le_bytes().to_vec();
    bincode::serialize_into(&mut buffer, &cr)?;
    Ok(buffer)
}

pub enum DeserializeError {
    Bincode(bincode::Error),
    InvalidVersion,
}

pub fn deserialize(buf: &[u8]) -> Result<CompileResult, DeserializeError> {
    let bytes = buf.try_into().map_err(|_| DeserializeError::InvalidVersion)?;
    let version = u32::from_le_bytes(bytes);

    if version != BYTECODE_VERSION {
        return Err(DeserializeError::InvalidVersion);
    }

    bincode::deserialize(&buf[4..]).map_err(DeserializeError::Bincode)
}
