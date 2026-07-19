//! Framing for the retained TypeFacts sidecar transport.

use std::{
    io::BufWriter,
    process::{ChildStdin, ChildStdout},
    sync::{Arc, Mutex},
};

use crate::BackendError;

pub(crate) fn write_frame(
    writer: &Arc<Mutex<BufWriter<ChildStdin>>>,
    value: &solid_ts_facts::v3::Request,
) -> Result<usize, BackendError> {
    let payload = solid_ts_facts::encode_sidecar_request(value)?;
    let mut writer = writer
        .lock()
        .map_err(|_| BackendError::Process("TypeFacts writer poisoned".into()))?;
    solid_ts_facts::write_frame(&mut *writer, &payload)?;
    Ok(payload.len())
}

pub(crate) fn read_frame(output: &mut ChildStdout) -> Result<Vec<u8>, BackendError> {
    solid_ts_facts::read_frame(output).map_err(BackendError::from)
}
