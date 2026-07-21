use serde::Deserialize;
use serde_json;
use std::ffi::CStr;

use plugin_interface::{PluginInterfaceError, RGBA_CHANNELS};

#[derive(Debug, Default, Deserialize)]
#[serde(default, deny_unknown_fields)]
struct SettingsMirror {
    radius: usize,
    iterations: usize,
}

fn apply_blur(
    width: usize,
    height: usize,
    pixels: &mut [u8],
    params: &str,
) -> Result<(), PluginInterfaceError> {
    if width == 0 || height == 0 {
        return Ok(());
    }

    let settings = serde_json::from_str::<SettingsMirror>(params)?;
    for _ in 0..settings.iterations {
        let mut pixels_new = vec![0u8; RGBA_CHANNELS * height * width];

        for col in 0..width {
            for row in 0..height {
                let avr_pxl = avr_pixel_value(pixels, width, height, col, row, settings.radius);
                let indx = (row * width + col) * RGBA_CHANNELS;

                for channel in 0..RGBA_CHANNELS {
                    pixels_new[indx + channel] = avr_pxl[channel] as u8;
                }
            }
        }

        pixels.copy_from_slice(&pixels_new);
    }

    Ok(())
}

fn avr_pixel_value(
    pixels: &[u8],
    width: usize,
    height: usize,
    col: usize,
    row: usize,
    radius: usize,
) -> [u8; RGBA_CHANNELS] {
    let mut sum = [0u32; RGBA_CHANNELS];
    let mut divider: u32 = 0;

    let col_start = col.saturating_sub(radius);
    let row_start = row.saturating_sub(radius);
    let col_end = (col + radius).min(width - 1);
    let row_end = (row + radius).min(height - 1);

    for curr_col in col_start..=col_end {
        for curr_row in row_start..=row_end {
            divider += 1;
            for i in 0..4 {
                sum[i] += pixels[(curr_row * width + curr_col) * RGBA_CHANNELS + i] as u32;
            }
        }
    }

    let mut result = [0u8; RGBA_CHANNELS];

    for channel in 0..RGBA_CHANNELS {
        result[channel] = (sum[channel] / divider) as u8;
    }
    result
}

#[unsafe(no_mangle)]
pub extern "C" fn process_image(
    width: u32,
    height: u32,
    rgba_data: *mut u8,
    params: *const std::os::raw::c_char,
) -> i32 {
    if rgba_data.is_null() {
        return -1;
    }
    if params.is_null() {
        return -2;
    }

    let pixels_len = match (width as usize)
        .checked_mul(height as usize)
        .and_then(|value| value.checked_mul(RGBA_CHANNELS))
    {
        Some(value) => value,
        None => return -5,
    };

    let pixels = unsafe { std::slice::from_raw_parts_mut(rgba_data, pixels_len) };

    let params_cstr = unsafe { CStr::from_ptr(params) };

    let settings = match params_cstr.to_str() {
        Ok(value) => value,
        Err(_) => return -3,
    };

    match apply_blur(width as usize, height as usize, pixels, settings) {
        Err(_) => -4,
        Ok(_) => 0,
    }
}
