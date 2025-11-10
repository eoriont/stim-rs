use crate::StimError;

pub(crate) fn to_u64(value: usize) -> Result<u64, StimError> {
    u64::try_from(value).map_err(|_| StimError::Conversion("value exceeds 64-bit range".into()))
}

pub(crate) fn to_usize(value: u64) -> Result<usize, StimError> {
    usize::try_from(value)
        .map_err(|_| StimError::Conversion("value exceeds platform usize range".into()))
}

pub(crate) fn bool_vec_from_u8(values: Vec<u8>) -> Vec<bool> {
    values.into_iter().map(|v| v != 0).collect()
}

pub(crate) fn matrix_from_flat(flat: Vec<u8>, rows: usize, cols: usize) -> Vec<Vec<bool>> {
    debug_assert_eq!(flat.len(), rows.saturating_mul(cols));
    let mut iter = flat.into_iter();
    (0..rows)
        .map(|_| {
            (0..cols)
                .map(|_| (iter.next().unwrap_or_default() != 0))
                .collect()
        })
        .collect()
}

pub(crate) fn bytes_to_string(bytes: Vec<u8>) -> String {
    match String::from_utf8(bytes) {
        Ok(s) => s,
        Err(err) => String::from_utf8_lossy(&err.into_bytes()).into_owned(),
    }
}
