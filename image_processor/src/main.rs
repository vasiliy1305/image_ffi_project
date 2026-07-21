use clap::Parser;
use image::{ImageReader, RgbaImage};
use std::{ path::PathBuf};
use thiserror::Error;

use libloading::{Library, Symbol};
use std::os::raw::c_char;

use plugin_interface::{
    PROCESS_IMAGE_SYMBOL,
    ProcessImageFn,
};

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
    if !plugin_path.exists() {
        return Err(AppErrors::FileNotFound(plugin_path.display().to_string()));
    }

    let input_img = ImageReader::open(&args.input)?.decode()?.to_rgba8();
    let width = input_img.width();
    let height = input_img.height();
    let pixels = input_img.into_raw();

    let output_img = RgbaImage::new(width, height);
    output_img.save(&args.output)?;

    // let lib = unsafe { Library::new(&args.plugin)? };
    // let process: Symbol<ProcessImageFn> = unsafe { lib.get(b"process_image\0")? };

    println!("{:?}", height);
    println!("ОС: {}", std::env::consts::OS);
    println!("Архитектура: {}", std::env::consts::ARCH);
    println!("Семейство ОС: {}", std::env::consts::FAMILY);

    Ok(())
}
