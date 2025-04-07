use super::CompileResult;

const BYTECODE_VERSION: u32 = 5;

pub fn serialize(cr: CompileResult) -> Result<Vec<u8>, bincode::error::EncodeError> {
    let mut buffer = BYTECODE_VERSION.to_le_bytes().to_vec();
    bincode::serde::encode_into_std_write(cr, &mut buffer, bincode::config::standard())?;
    Ok(buffer)
}

#[derive(Debug)]
pub enum DeserializeError {
    Bincode(bincode::error::DecodeError),
    InvalidVersion,
}

pub fn deserialize(buf: &[u8]) -> Result<CompileResult, DeserializeError> {
    let bytes = buf.try_into().map_err(|_| DeserializeError::InvalidVersion)?;
    let version = u32::from_le_bytes(bytes);

    if version != BYTECODE_VERSION {
        return Err(DeserializeError::InvalidVersion);
    }

    let (cr, _) =
        bincode::serde::decode_from_slice(&buf[4..], bincode::config::standard()).map_err(DeserializeError::Bincode)?;

    Ok(cr)
}
