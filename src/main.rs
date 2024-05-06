use clap::Parser;
use image::{self, imageops::FilterType::Nearest, DynamicImage, GenericImageView, Pixel};
use std::path::PathBuf;

// Ascii to image converter
#[derive(Parser, Debug)]
#[command(author = "klliio", version, about, long_about = None, arg_required_else_help = true)]
struct Args {
    // Path to image
    #[arg(short, long)]
    path: PathBuf,

    // Contrast [-128 - 127]
    #[arg(short, long, default_value_t = 0)]
    constrast: i8,

    // Scale% [0 - 255]
    #[arg(short, long, default_value_t = 100)]
    scale: u8,
}

fn main() {
    let args = Args::parse();
    let mut image = image::open(args.path)
        .unwrap()
        .adjust_contrast(f32::from(args.constrast));

    if args.scale != 100 {
        let n_width: u32 =
            (f64::from(image.width()) * (f64::from(args.scale) / 100.0)).round() as u32;
        let n_height: u32 =
            (f64::from(image.height()) * (f64::from(args.scale) / 100.0)).round() as u32;

        image = image.resize(n_width, n_height, Nearest);
    }

    let characters: [char; 17] = [
        '$', '#', 'B', '%', '*', 'o', 'c', ';', ':', '<', '~', '^', '"', '\'', ',', '.', ' ',
    ];

    let mut i = 1;
    for pix in image.pixels() {
        let luma = pix.2.to_luma()[0];

        print!(
            "{} ",
            characters[((255 - luma) / u8::try_from(characters.len()).unwrap()) as usize]
        );
        if (i % image.width()) == 0 {
            println!();
        }
        i += 1;
    }
}
