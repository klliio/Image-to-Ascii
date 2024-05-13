use clap::{error::Result, Parser};
use image::{self, imageops::FilterType::Nearest, DynamicImage, GenericImageView, Pixel};
use std::{
    io::{self, Stdout, Write},
    path::PathBuf,
};
use termsize;

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
    #[arg(short, long, default_value_t = 0)]
    scale: u8,
}

fn main() {
    let args = Args::parse();
    let mut image = image::open(args.path)
        .unwrap()
        .adjust_contrast(f32::from(args.constrast));

    // unable to get size when piping to a program that ends early like head
    if args.scale == 0 {
        let size: (u32, u32) = match termsize::get() {
            Some(size) => (u32::from(size.cols), u32::from(size.rows)),
            None => {
                println!("Unable to get terminal size\nDefaulting to 50x50");
                (50, 50)
            }
        };
        image = image.resize(size.0, size.1, Nearest);
    } else if args.scale != 0 {
        let n_width: u32 =
            (f64::from(image.width()) * (f64::from(args.scale) / 100.0)).round() as u32;

        let n_height: u32 =
            (f64::from(image.height()) * (f64::from(args.scale) / 100.0)).round() as u32;

        image = image.resize(n_width, n_height, Nearest);
    }
    
    if let Err(error) = output(image) {
        eprintln!("{}", error);
    }
}

fn output(image: DynamicImage) -> Result<(), io::Error> {
    let characters: [char; 17] = [
        '$', '#', 'B', '%', '*', 'o', 'c', ';', ':', '<', '~', '^', '"', '\'', ',', '.', ' ',
    ];

    let stdout = io::stdout();
    let mut stdout = stdout.lock();

    let mut i = 1;
    for pix in image.pixels() {
        let luma = pix.2.to_luma()[0];

        write!(
            stdout,
            "{} ",
            characters[((255 - luma) / u8::try_from(characters.len()).unwrap()) as usize]
        );
        if (i % image.width()) == 0 {
            write!(stdout, "\n",);
        }
        i += 1;
    }

    Ok(())
}
