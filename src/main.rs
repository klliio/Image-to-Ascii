use clap::{error::Result, Parser};
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

    process_pixel(image, args.colour, args.no_bg)
}

fn process_pixel(image: DynamicImage, colour: bool, no_bg: bool) {
    let characters: [char; 17] = [
        '$', '#', 'B', '%', '*', 'o', 'c', ';', ':', '<', '~', '^', '"', '\'', ',', '.', ' ',
    ];

    let mut buffer_chars: Vec<Vec<char>> = vec![];
    let mut buffer_colours: Vec<Vec<image::Rgba<u8>>> = vec![];

    let mut row: u32 = 0;
    while row < image.height() {
        let mut column: u32 = 0;
        let mut row_chars: Vec<char> = vec![];
        let mut row_colours: Vec<image::Rgba<u8>> = vec![];

        while column < image.width() {
            let pix = image.get_pixel(column, row);
            let luma = pix.to_luma()[0];

            let mut rgba = image::Rgba([255, 255, 255, pix.0[3]]);
            if colour {
                rgba = pix.to_rgba();
            }

            row_colours.push(rgba);
            row_chars.push(
                characters[((255 - luma) / u8::try_from(characters.len()).unwrap()) as usize],
            );

            column += 1;
        }

        buffer_colours.push(row_colours);
        buffer_chars.push(row_chars);
        row += 1;
    }

    if let Err(error) = output(buffer_chars, buffer_colours, no_bg) {
        eprintln!("{}", error);
    }
}

fn output(
    buffer_chars: Vec<Vec<char>>,
    buffer_colours: Vec<Vec<image::Rgba<u8>>>,
    no_bg: bool,
) -> Result<(), io::Error> {
    let stdout = io::stdout();
    let mut stdout = stdout.lock();

    for row in buffer_chars.iter().enumerate() {
        for row_char in row.1.iter().enumerate() {
            if (buffer_colours[row.0][row_char.0][3] == 0) && no_bg {
                write!(stdout, "  ");
            } else {
                write!(
                    stdout,
                    "{} ",
                    row_char.1.if_supports_color(Stdout, |text| text.truecolor(
                        buffer_colours[row.0][row_char.0][0],
                        buffer_colours[row.0][row_char.0][1],
                        buffer_colours[row.0][row_char.0][2]
                    ))
                );
            }
        }
        write!(stdout, "\n ");
    }

    Ok(())
}
