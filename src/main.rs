use clap::{error::Result, Parser};
use core::panic;
use image::{self, imageops::FilterType::Nearest, DynamicImage, GenericImageView, Pixel};
use owo_colors::{OwoColorize, Stream::Stdout};
use std::{
    io::{self, Write},
    path::PathBuf,
};

// Image to Ascii converter
#[derive(Parser, Debug)]
#[command(author = "klliio", version, about, long_about = None, arg_required_else_help = true)]
struct Args {
    // Path to image
    #[arg(short, long)]
    path: PathBuf,

    // enable colours
    #[arg(long, default_value_t = false)]
    colour: bool,

    // remove background
    #[arg(long, default_value_t = false)]
    no_bg: bool,

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

    // handle if the total pixels is greater than u32 can hold
    if image.width().checked_mul(image.height()).is_none() {
        panic!("u32 overflow, Image too large");
    }

    let image_info: (Vec<char>, Vec<image::Rgba<u8>>) = process(&image, &args.colour, &args.no_bg);

    if let Err(error) = output(&image_info.0, &image_info.1, &image.width()) {
        eprintln!("{}", error);
    }
}

fn process(image: &DynamicImage, colour: &bool, no_bg: &bool) -> (Vec<char>, Vec<image::Rgba<u8>>) {
    let characters: [char; 17] = [
        '$', '#', 'B', '%', '*', 'o', 'c', ';', ':', '<', '~', '^', '"', '\'', ',', '.', ' ',
    ];

    let mut char_vec: Vec<char> = vec![];
    let mut colour_vec: Vec<image::Rgba<u8>> = vec![];

    let mut row: u32 = 0;
    while row < image.height() {
        let mut column: u32 = 0;

        while column < image.width() {
            let pix = image.get_pixel(column, row);
            let luma = pix.to_luma()[0];

            let alpha: u8 = if *no_bg { 0 } else { pix.0[3] };
            let rgba: image::Rgba<u8> = {
                if *colour {
                    pix.to_rgba()
                } else {
                    image::Rgba([255, 255, 255, alpha])
                }
            };

            colour_vec.push(rgba);
            char_vec.push(
                characters[((255 - luma) / u8::try_from(characters.len()).unwrap()) as usize],
            );

            column += 1;
        }

        row += 1;
    }

    (char_vec, colour_vec)
}

fn output(
    char_vec: &[char],
    colour_vec: &[image::Rgba<u8>],
    width: &u32,
) -> Result<(), io::Error> {
    let stdout = io::stdout();
    let mut stdout = stdout.lock();

    for char in char_vec.iter().enumerate() {
        if colour_vec[char.0][3] == 0 {
            write!(stdout, "  ");
        } else {
            write!(
                stdout,
                "{} ",
                char.1.if_supports_color(Stdout, |text| text.truecolor(
                    colour_vec[char.0][0],
                    colour_vec[char.0][1],
                    colour_vec[char.0][2]
                ))
            );
        }

        if u32::try_from(char.0).expect("Unable to convert to u32. The Image too large.") % width
            == 0
        {
            write!(stdout, "\n");
        }
    }

    Ok(())
}
