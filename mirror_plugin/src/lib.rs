use serde::Deserialize;
use serde_json;
use std::ffi::CStr;

use plugin_interface::{PluginInterfaceError, RGBA_CHANNELS};

#[derive(Debug, Default, Deserialize)]
#[serde(default, deny_unknown_fields)]
struct SettingsMirror {
    horizontal: bool,
    vertical: bool,
}

fn apply_mirror(
    width: usize,
    height: usize,
    pixels: &mut [u8],
    params: &str,
) -> Result<(), PluginInterfaceError> {
    let settings = serde_json::from_str::<SettingsMirror>(params)?;

    if settings.vertical {
        for row in 0..height / 2 {
            for col in 0..width {
                let first = row * width + col;
                let second = (height - 1 - row) * width + col;
                swap_pixels(pixels, first, second);
            }
        }
    }

    if settings.horizontal {
        for row in 0..height {
            for col in 0..width / 2 {
                let first = row * width + col;
                let second = row * width + (width - 1 - col);
                swap_pixels(pixels, first, second);
            }
        }
    }

    Ok(())
}

fn swap_pixels(pixels: &mut [u8], first_pixel: usize, second_pixel: usize) {
    let first_start = first_pixel * RGBA_CHANNELS;
    let second_start = second_pixel * RGBA_CHANNELS;

    for channel in 0..RGBA_CHANNELS {
        pixels.swap(first_start + channel, second_start + channel);
    }
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

    match apply_mirror(width as usize, height as usize, pixels, settings) {
        Err(_) => -4,
        Ok(_) => 0,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::ffi::CString;

    fn pixel(value: u8) -> [u8; RGBA_CHANNELS] {
        [value, value, value, 255]
    }

    #[test]
    fn horizontal_mirror() {
        let mut pixels = [pixel(1), pixel(2), pixel(3), pixel(4), pixel(5), pixel(6)].concat();

        let params = CString::new(
            r#"{
                "horizontal": true,
                "vertical": false
            }"#,
        )
        .unwrap();

        let result = process_image(3, 2, pixels.as_mut_ptr(), params.as_ptr());
        assert_eq!(result, 0);

        let expected = [pixel(3), pixel(2), pixel(1), pixel(6), pixel(5), pixel(4)].concat();
        assert_eq!(pixels, expected);
    }

    #[test]
    fn vertical_mirror() {
        let mut pixels = [pixel(1), pixel(2), pixel(3), pixel(4), pixel(5), pixel(6)].concat();

        let params = CString::new(
            r#"{
                "horizontal": false,
                "vertical": true
            }"#,
        )
        .unwrap();

        let result = process_image(3, 2, pixels.as_mut_ptr(), params.as_ptr());
        assert_eq!(result, 0);

        let expected = [pixel(4), pixel(5), pixel(6), pixel(1), pixel(2), pixel(3)].concat();
        assert_eq!(pixels, expected);
    }
}
