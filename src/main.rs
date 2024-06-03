use clap::{error::Result, Parser};
use image::Rgba;
use image::{self, imageops::FilterType::Nearest, GenericImageView, Pixel};
use owo_colors::{OwoColorize, Stream::Stdout};
use std::sync::{mpsc, Arc};
use std::thread;
use std::{
    io::{self, Write},
    path::PathBuf,
};

// Image to Ascii converter
#[derive(Parser, Debug)]
#[command(author = "klliio", version, about, long_about = None, arg_required_else_help = true)]
struct Args {
    // Path to image
    #[arg(short, long, default_value = "/home/klliio/Images/kirby.webp")]
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
    #[arg(short, long, default_value_t = 10)]
    scale: u8,

    // Threads
    #[arg(short, long, default_value_t = 1)]
    threads: u8,
}

fn main() {
    let args = Args::parse();
    let image = image::open(args.path)
        .unwrap()
        .adjust_contrast(f32::from(args.constrast));

    let image = {
        // unable to get size when piping to a program that ends early like head
        if args.scale == 0 {
            let size: (u32, u32) = match termsize::get() {
                Some(size) => (u32::from(size.cols), u32::from(size.rows)),
                None => {
                    println!("Unable to get terminal size\nDefaulting to 50x50");
                    (50, 50)
                }
            };
            image.resize(size.0, size.1, Nearest)
        } else {
            let n_width: u32 =
                (f64::from(image.width()) * (f64::from(args.scale) / 100.0)).round() as u32;
            let n_height: u32 =
                (f64::from(image.height()) * (f64::from(args.scale) / 100.0)).round() as u32;

            image.resize(n_width, n_height, Nearest)
        }
    };

    let characters: [char; 17] = [
        '$', '#', 'B', '%', '*', 'o', 'c', ';', ':', '<', '~', '^', '"', '\'', ',', '.', ' ',
    ];

    let mut thread_vec = vec![];
    let (tx, rx) = mpsc::channel();

    let image = Arc::new(image);
    let arg_toggles = Arc::new((args.colour, args.no_bg));

    let section_height: u32 = image.height() / u32::from(args.threads);
    for thread in 0..args.threads {
        let tx = tx.clone();
        let image = Arc::clone(&image);
        let arg_toggles = Arc::clone(&arg_toggles);

        let thread_pos: (u32, u32) = {
            let start = section_height * u32::from(thread);
            let end = if thread != args.threads - 1 {
                start + section_height
            } else {
                start + section_height + (image.height() % u32::from(args.threads))
            };
            (start, end)
        };

        let handle = thread::spawn(move || {
            // Y
            for row in thread_pos.0..thread_pos.1 {
                // X
                for col in 0..=image.width() {
                    let (luma, rgba, character);
                    let index = ((image.width() + 1) * row + col) as usize;

                    if col < image.width() {
                        (luma, rgba) = get_pixel_info(*arg_toggles, &image.get_pixel(col, row));
                        character = characters[(255 - usize::from(luma)) / characters.len()];
                    } else {
                        // newlines don't get shown if the alpha is 0
                        rgba = Rgba([0, 0, 0, 255]);
                        character = '\n';
                    }

                    match tx.send((index, character, rgba)) {
                        Ok(()) => (),
                        Err(error) => eprintln!("ERROR: {}", error),
                    }
                }
            }
        });
        thread_vec.push(handle);
    }
    drop(tx);

    // add 1 to width to account for newlines
    let mut char_vec: Vec<char> =
        vec!['E'; usize::try_from((image.width() + 1) * image.height()).unwrap()];
    let mut colour_vec: Vec<Rgba<u8>> = vec![
        Rgba([255, 255, 255, 255]);
        usize::try_from((image.width() + 1) * image.height())
            .unwrap()
    ];

    for message in rx {
        char_vec[message.0] = message.1;
        colour_vec[message.0] = message.2;
    }

    for handle in thread_vec {
        handle.join().unwrap();
    }

    if let Err(error) = output(&char_vec, &colour_vec) {
        eprintln!("{}", error);
    }
}

fn get_pixel_info((colour, no_bg): (bool, bool), pixel: &Rgba<u8>) -> (u8, Rgba<u8>) {
    let luma = pixel.to_luma()[0];

    let rgba: Rgba<u8> = if colour {
        pixel.to_rgba()
    } else {
        Rgba([255, 255, 255, if no_bg { pixel.0[3] } else { 255 }])
    };

    (luma, rgba)
}

fn output(char_vec: &[char], colour_vec: &[image::Rgba<u8>]) -> Result<(), io::Error> {
    let stdout = io::stdout();
    let mut stdout = stdout.lock();

    for char in char_vec.iter().enumerate() {
        if colour_vec[char.0][3] == 0 {
            write!(stdout, "  ");
        } else {
            write!(
                stdout,
                " {}",
                char.1.if_supports_color(Stdout, |text| text.truecolor(
                    colour_vec[char.0][0],
                    colour_vec[char.0][1],
                    colour_vec[char.0][2]
                ))
            );
        }
    }

    Ok(())
}
