use clap::Parser;
use image::{ImageReader, RgbaImage};
use libloading::{Library, Symbol};
use plugin_interface::ProcessImageFn;
use std::ffi::CString;
use std::path::PathBuf;
use thiserror::Error;

#[derive(Debug, Parser)]
#[command(name = "image processor")]
struct Cli {
    #[arg(long)]
    input: PathBuf,

    #[arg(long)]
    output: PathBuf,

    #[arg(long)]
    plugin: String,

    #[arg(long)]
    params: PathBuf,

    #[arg(long)]
    plugin_path: Option<PathBuf>,
}

#[derive(Debug, Error)]
enum AppErrors {
    #[error("File not found: {0}")]
    FileNotFound(String),

    #[error("IO Error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Image processing error: {0}")]
    Image(#[from] image::ImageError),

    #[error("Plugin loading error: {0}")]
    Library(#[from] libloading::Error),

    #[error("Plugin returned error code: {0}")]
    PluginStatus(i32),

    #[error("Parameters contain an internal null byte: {0}")]
    InvalidCString(#[from] std::ffi::NulError),

    #[error("Failed to reconstruct output image")]
    InvalidImageBuffer,
}

use std::env::consts::{DLL_PREFIX, DLL_SUFFIX};

fn create_library_path(name: &str, dir_path: Option<PathBuf>) -> PathBuf {
    let directory = dir_path.unwrap_or_else(|| PathBuf::from("target").join("debug"));

    let file_name = format!("{DLL_PREFIX}{name}{DLL_SUFFIX}");
    directory.join(file_name)
}

fn main() -> Result<(), AppErrors> {
    let args = Cli::parse();

    if !args.input.exists() {
        return Err(AppErrors::FileNotFound(args.input.display().to_string()));
    }

    if !args.params.exists() {
        return Err(AppErrors::FileNotFound(args.params.display().to_string()));
    }

    let plugin_path = create_library_path(&args.plugin, args.plugin_path);

    println!("Plugin path: {:?}", plugin_path);

    if !plugin_path.exists() {
        return Err(AppErrors::FileNotFound(plugin_path.display().to_string()));
    }

    let input_img = ImageReader::open(&args.input)?.decode()?.to_rgba8();
    let width = input_img.width();
    let height = input_img.height();
    let mut pixels = input_img.into_raw();
    let params_text = std::fs::read_to_string(&args.params)?;
    let params = CString::new(params_text)?;

    let lib = unsafe { Library::new(&plugin_path)? };
    let process: Symbol<ProcessImageFn> = unsafe { lib.get(b"process_image\0")? };

    let status = unsafe { process(width, height, pixels.as_mut_ptr(), params.as_ptr()) };

    println!("Plugin status: {status}");

    if status != 0 {
        return Err(AppErrors::PluginStatus(status));
    }

    let output_img =
        RgbaImage::from_raw(width, height, pixels).ok_or(AppErrors::InvalidImageBuffer)?;

    output_img.save(&args.output)?;

    Ok(())
}
