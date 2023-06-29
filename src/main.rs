use std::env;
use fundsp::hacker32::*;
use image::io::Reader as ImageReader;
use image::{GenericImageView, Pixel, Luma};

const UP_LIMIT: f64 = 18000.0;
const DOWN_LIMIT: f64 = 25.0;
const BASE_FREQ: f32 = 32.70;
const NOTE_QUANTITY: f32 = 60.0;

const IMAGE_MAX_X: u32 = 20;
const IMAGE_MAX_Y: u32 = 20;

pub mod errors {
    #[derive(Debug, Clone)]
    pub enum Error {
        OtherError(String)
    }
    impl<T: Errorable> From<T> for Error {
        fn from(t: T) -> Self {
            t.to_err()
        }
    }

    pub trait Errorable {
        fn to_err(&self) -> Error;
    }
    impl<T: ToString> Errorable for T {
        fn to_err(&self) -> Error {
            Error::OtherError(self.to_string())
        }
    }

    pub type SimpleResult<T> = Result<T, Error>;
}
use errors::*;

fn main() -> SimpleResult<()> {
    // Get image file from command line
    let args: Vec<String> = env::args().collect();
    let mut image_path = "image.png".to_string();
    let mut output_path = "audio.wav".to_string();
    for (i, arg) in args.iter().enumerate() {
        if arg == &"--image".to_string() {
            if i + 1 < args.len() {
                image_path = args[i + 1].clone();
            } else {
                println!("Invalid image argument");
                return Ok(());
            }
        }
        if arg == &"--output".to_string() {
            if i + 1 < args.len() {
                output_path = args[i + 1].clone();
            } else {
                println!("Invalid output argument");
                return Ok(());
            }
        }
    }

    // Load image
    println!("Loading image: {}", image_path);
    let image = ImageReader::open(image_path)?.decode()?;

    // Resize image
    println!("Resizing image");
    let image = image.resize_to_fill(IMAGE_MAX_X, IMAGE_MAX_Y, image::imageops::FilterType::Nearest);

    // Convert image to grayscale
    println!("Converting image to grayscale");
    let image = image.grayscale();

    // Get pixels
    println!("Getting pixels");
    let pixels: Vec<(u32, u32, Luma<u8>)> = image.pixels().map(|x| (x.0, x.1, x.2.to_luma())).collect();

    // Empty AudioUnit64
    let mut audio_unit = Net32::new(0, 2);

    // Iterate through pixels
    println!("Iterating through pixels");
    for (x, y, luma) in pixels {
        let value = luma.0[0] as f32;
        let value = value / 255.0;

        if value < 0.1 {
            continue;
        }
        
        let note = ((image.height() - y) as f32 / image.height() as f32) * NOTE_QUANTITY;

        let freq = BASE_FREQ * 2.0_f32.powf(1.0/12.0).powi(note as i32);

        let stereo = (((x as f32) / (image.width() as f32)) * 2.0) - 1.0;

        let mono_au = sine_hz(freq) * value;
        let stereo_au = mono_au >> pan(stereo);

        audio_unit = audio_unit + stereo_au;
    }

    // Render audio
    println!("Rendering audio");
    let wave1 = Wave32::render(44100.0, 10.0, &mut audio_unit);

    // Save audio
    println!("Saving audio");
    wave1.save_wav16(output_path)?;
    Ok(())
}
