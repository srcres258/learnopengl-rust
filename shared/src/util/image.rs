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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::filesystem;

    #[test]
    fn load_image_data_rgb_test_existing() {
        let img = load_image_data_rgb(
            filesystem::get_path("resources/textures/awesomeface.png".to_string()))
            .expect("The file should exist.");
        assert_ne!(img.len(), 0, "The file should have contents.");
    }

    #[test]
    fn load_image_data_rgb_test_not_existing() {
        let img = load_image_data_rgb("I_AM_NOT_EXISTING".to_string());
        if let Ok(_) = img {
            panic!("The file shouldn't exist.");
        }
    }

    #[test]
    fn load_image_data_rgba_test_existing() {
        let img = load_image_data_rgba(
            filesystem::get_path("resources/textures/awesomeface.png".to_string()))
            .expect("The file should exist.");
        assert_ne!(img.len(), 0, "The file should have contents.");
    }

    #[test]
    fn load_image_data_rgba_test_not_existing() {
        let img = load_image_data_rgba("I_AM_NOT_EXISTING".to_string());
        if let Ok(_) = img {
            panic!("The file shouldn't exist.");
        }
    }

    #[test]
    fn load_image_data_rgb_without_flip_test_existing() {
        let img = load_image_data_rgb_without_flip(
            filesystem::get_path("resources/textures/awesomeface.png".to_string()))
            .expect("The file should exist.");
        assert_ne!(img.len(), 0, "The file should have contents.");
    }

    #[test]
    fn load_image_data_rgb_without_flip_test_not_existing() {
        let img = load_image_data_rgb_without_flip("I_AM_NOT_EXISTING".to_string());
        if let Ok(_) = img {
            panic!("The file shouldn't exist.");
        }
    }

    #[test]
    fn load_image_data_rgba_without_flip_test_existing() {
        let img = load_image_data_rgba_without_flip(
            filesystem::get_path("resources/textures/awesomeface.png".to_string()))
            .expect("The file should exist.");
        assert_ne!(img.len(), 0, "The file should have contents.");
    }

    #[test]
    fn load_image_data_rgba_without_flip_test_not_existing() {
        let img = load_image_data_rgba_without_flip("I_AM_NOT_EXISTING".to_string());
        if let Ok(_) = img {
            panic!("The file shouldn't exist.");
        }
    }
}