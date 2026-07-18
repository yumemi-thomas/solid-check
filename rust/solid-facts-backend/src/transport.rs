//! Framing for the retained TypeFacts sidecar transport.

use std::{
    io::{BufWriter, Write},
    process::{ChildStdin, ChildStdout},
    sync::{Arc, Mutex},
};

use crate::BackendError;

pub(crate) fn write_frame(
    writer: &Arc<Mutex<BufWriter<ChildStdin>>>,
    value: &solid_ts_facts::v3::Request,
) -> Result<usize, BackendError> {
    let payload = solid_ts_facts::encode_sidecar_request(value)?;
    let length = u32::try_from(payload.len())
        .map_err(|_| BackendError::Process("TypeFacts request exceeds u32 framing".into()))?;
    let mut writer = writer
        .lock()
        .map_err(|_| BackendError::Process("TypeFacts writer poisoned".into()))?;
    writer.write_all(&length.to_le_bytes())?;
    writer.write_all(&payload)?;
    writer.flush()?;
    Ok(payload.len())
}

pub(crate) fn read_frame(output: &mut ChildStdout) -> Result<Vec<u8>, BackendError> {
    let mut prefix = [0_u8; 4];
    std::io::Read::read_exact(output, &mut prefix)?;
    let length = u32::from_le_bytes(prefix) as usize;
    if length > solid_ts_facts::MAX_MESSAGE_BYTES {
        return Err(BackendError::Process(format!(
            "TypeFacts response exceeds message limit: {length}"
        )));
    }
    let mut payload = vec![0; length];
    std::io::Read::read_exact(output, &mut payload)?;
    Ok(payload)
}
