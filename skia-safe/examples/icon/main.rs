// Author: Alberto González Palomo https://sentido-labs.com
// ©2019 Alberto González Palomo https://sentido-labs.com
// Released under the MIT license: https://opensource.org/licenses/MIT

// Renders the animated version of the rust-skia icon
// as a sequence of PNG files in the current directory.

#![allow(clippy::many_single_char_names)]

mod renderer;
use renderer::render_frame;

const USAGE: &str = r#"icon [size]
With <size> parameter, produce a single PNG image.
Without parameters, produce PNG frames for the whole animation."#;

use std::fs::File;
use std::io::Write;

use skia_safe::{Color, EncodedImageFormat, Surface};

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let (size, single_frame) = match args.len() {
        1 => (512, false),
        2 => {
            if args[1].starts_with('-') {
                panic!("Usage: {}", USAGE)
            } else {
                match args[1].parse::<i32>() {
                    Ok(integer) => (integer, true),
                    Err(e) => panic!("Error: {}\nUsage: {}", e.to_string(), USAGE),
                }
            }
        }
        _ => panic!("Usage: {}", USAGE),
    };

    let fps = 60;
    let bpm = 60;

    let mut surface =
        Surface::new_raster_n32_premul((size, size)).expect("No SKIA surface available.");

    let mut frame: usize = 0;
    let mut frames_left = 1;
    while frames_left > 0 {
        eprint!("Rendering frame {}:{}\r", frame, frame + frames_left);

        surface.canvas().clear(Color::WHITE);

        frames_left = render_frame(frame, fps, bpm, surface.canvas());

        let file_name = if single_frame {
            frames_left = 0;
            format!("rust-skia-icon_{}x{}.png", size, size)
        } else {
            format!("rust-skia-icon-{:0>4}.png", frame)
        };
        let mut file = File::create(file_name).unwrap();
        let image = surface.image_snapshot();
        match image.encode_to_data(EncodedImageFormat::PNG) {
            Some(data) => {
                file.write_all(data.as_bytes()).unwrap();
            }
            None => {
                eprintln!("ERROR: failed to encode image as PNG.");
            }
        }

        frame += 1;
    }

    if !single_frame {
        eprintln!("Rendered {} frames.          ", frame);
        eprintln!("Encode as video with:\nffmpeg -framerate {fps} -i rust-skia-icon-%04d.png -vcodec libvpx-vp9 -crf 15 -b:v 0 -auto-alt-ref 0 -pass 1 -f webm /dev/null && ffmpeg -framerate {fps} -i rust-skia-icon-%04d.png -vcodec libvpx-vp9 -pix_fmt yuv444p -crf 15 -b:v 0 -auto-alt-ref 0 -pass 2 rust-skia-icon.webm", fps=fps);
        eprintln!("Play in a loop with:\nmpv --loop rust-skia-icon.webm");
    }
}
