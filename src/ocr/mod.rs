use std::error::Error;
pub mod paddle;
pub use paddle as ocr;

pub trait Ocr {
    fn ocr(&mut self, img: &image::DynamicImage) ->  Result<Vec<String>, Box<dyn Error>>;
}