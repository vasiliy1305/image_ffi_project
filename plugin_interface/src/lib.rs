use serde_json;
use std::os::raw::c_char;
use thiserror::Error;

pub type ProcessImageFn = unsafe extern "C" fn(u32, u32, *mut u8, *const c_char) -> i32;

pub const PROCESS_IMAGE_SYMBOL: &[u8] = b"process_image\0";

pub const RGBA_CHANNELS: usize = 4;

#[derive(Debug, Error)]
pub enum PluginInterfaceError {
    #[error("Json Error: {0}")]
    JsonError(#[from] serde_json::Error),

    #[error("Unexpected pixel length: expected {expected}, actual {actual}")]
    UnexpectedPixelLen { expected: usize, actual: usize },

    #[error("usize owerflow")]
    UsizeOverflow,
}

pub fn check_img(width: usize, height: usize, pixels: &[u8]) -> Result<(), PluginInterfaceError> {
    let expected_len = width
        .checked_mul(height)
        .and_then(|value| value.checked_mul(RGBA_CHANNELS))
        .ok_or(PluginInterfaceError::UsizeOverflow)?;

    if pixels.len() != expected_len {
        return Err(PluginInterfaceError::UnexpectedPixelLen {
            expected: expected_len,
            actual: pixels.len(),
        });
    }

    Ok(())
}
