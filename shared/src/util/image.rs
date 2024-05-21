use std::error::Error;
use image::{RgbaImage, RgbImage};
use image::io::Reader as ImageReader;

pub fn load_image_data_rgb(path: String) -> Result<RgbImage, Box<dyn Error>> {
    let img = ImageReader::open(path)?.with_guessed_format()?.decode()?.flipv();
    Ok(img.to_rgb8())
}

pub fn load_image_data_rgba(path: String) -> Result<RgbaImage, Box<dyn Error>> {
    let img = ImageReader::open(path)?.with_guessed_format()?.decode()?.flipv();
    Ok(img.to_rgba8())
}

pub fn load_image_data_rgb_without_flip(path: String) -> Result<RgbImage, Box<dyn Error>> {
    let img = ImageReader::open(path)?.with_guessed_format()?.decode()?;
    Ok(img.to_rgb8())
}

pub fn load_image_data_rgba_without_flip(path: String) -> Result<RgbaImage, Box<dyn Error>> {
    let img = ImageReader::open(path)?.with_guessed_format()?.decode()?;
    Ok(img.to_rgba8())
}