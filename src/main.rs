use std::collections::HashMap;
use std::fs::read_dir;
use std::path::PathBuf;

mod lib;
use lib::index::*;

extern crate time;
use time::PreciseTime;

extern crate clap;
use clap::{App, Arg};

extern crate image;
use image::{FilterType, GenericImage, ImageBuffer};
extern crate md5;
#[macro_use]
extern crate serde_derive;
extern crate serde_json;

const DEFAULT_MAGNIFICATION: &'static str = "2";
const DEFAULT_PIXEL_GROUP_SIZE: &'static str = "16";
const INDEX_FILENAME: &'static str = ".mosaic_index";

static mut PRINT_TIMING: bool = false;
static mut VERBOSE: bool = false;

macro_rules! vprintln {
    ($fmt:expr) => { if unsafe { VERBOSE } { println!($fmt) } };
    ($fmt:expr, $($arg:tt)*) => { if unsafe { VERBOSE } { println!($fmt, $($arg)*) } };
}

fn main() {
    let params = get_parameters();
    let source_image_path = params.value_of("INPUT").unwrap();
    let library_dir_path = params.value_of("LIBRARY").unwrap();
    let out_file_path = params.value_of("OUT_FILE").unwrap();
    if params.is_present("print-timings") {
        unsafe {
            PRINT_TIMING = true;
        }
    }

    if params.is_present("verbose") {
        unsafe {
            VERBOSE = true;
        }
    }

    let pixel_group_size = params
        .value_of("SIZE")
        .unwrap_or("16")
        .parse::<u32>()
        .unwrap();
    let magnification_factor = params
        .value_of("MAGNIFICATION_FACTOR")
        .unwrap_or(DEFAULT_MAGNIFICATION)
        .parse::<u32>()
        .unwrap();

    vprintln!("Using magnification: {}", magnification_factor);
    vprintln!("Using pixel group size: {}", pixel_group_size);
    vprintln!("Recreating: {}", source_image_path);
    vprintln!("Using library at: {}", library_dir_path);

    let source_image = image::open(source_image_path)
        .expect(&format!("Error reading source image {}", source_image_path));
    let library = load_library(library_dir_path.to_string());
    let closest_image = |(r, g, b): (u8, u8, u8)| -> &PathBuf {
        library
            .iter()
            .min_by_key(|(_pb, d)| {
                ((((d.average.0 as i32).pow(2) - (r as i32).pow(2)).abs()
                    + ((d.average.1 as i32).pow(2) - (g as i32).pow(2)).abs()
                    + ((d.average.2 as i32).pow(2) - (b as i32).pow(2)).abs())
                    as f64)
                    .sqrt() as i32
            })
            .unwrap()
            .0
    };

    let (width, height) = source_image.dimensions();
    let new_width = (width as f32 / pixel_group_size as f32).round() as u32 * pixel_group_size;
    let new_height = (height as f32 / pixel_group_size as f32).round() as u32 * pixel_group_size;
    vprintln!("New starting dimensions: {} x {}", new_width, new_height);
    let mut source_image = source_image
        .resize_exact(new_width, new_height, FilterType::Nearest)
        .to_rgb();

    let mut img = ImageBuffer::new(
        source_image.width() * magnification_factor,
        source_image.height() * magnification_factor,
    );
    let mut library_cache = HashMap::new();
    vprintln!("Building image...");
    let timer = start_timer();
    for x_offset in 0..(source_image.width() / pixel_group_size) {
        for y_offset in 0..(source_image.height() / pixel_group_size) {
            let subimg = source_image.sub_image(
                x_offset * pixel_group_size,
                y_offset * pixel_group_size,
                pixel_group_size,
                pixel_group_size,
            );
            let ac = average_color(subimg.to_image());
            let ci = closest_image(ac);
            if !library_cache.contains_key(&ci) {
                library_cache.insert(
                    ci,
                    image::open(ci)
                        .expect(&format!("Error reading image {}", source_image_path))
                        .resize_exact(
                            pixel_group_size * magnification_factor,
                            pixel_group_size * magnification_factor,
                            FilterType::Nearest,
                        ),
                );
            }
            let source_image = library_cache.get(ci).unwrap();
            for x in 0..(pixel_group_size * magnification_factor) {
                for y in 0..(pixel_group_size * magnification_factor) {
                    img.put_pixel(
                        (x_offset * pixel_group_size * magnification_factor) + x,
                        (y_offset * pixel_group_size * magnification_factor) + y,
                        source_image.get_pixel(x, y),
                    );
                }
            }
            // println!(
            //     "closest color for {} {}: {}",
            //     x_offset * pixel_group_size,
            //     y_offset * pixel_group_size,
            //     ci.to_string_lossy().to_string()
            // );
        }
    }
    stop_timer(timer, "Image build time: ");
    let timer = start_timer();
    img.save(out_file_path)
        .expect("Failed to save result image");
    stop_timer(timer, "Image write time: ");
    println!("Wrote {}", out_file_path);
}

fn load_library(dir: String) -> HashMap<PathBuf, IndexData> {
    let index_file_path = {
        let mut p = PathBuf::from(&dir);
        p.push(INDEX_FILENAME);
        p
    };
    let mut index: HashMap<PathBuf, IndexData> = if index_file_path.exists() {
        vprintln!("Existing index found");
        read_index(&index_file_path)
    } else {
        vprintln!("No index found");
        HashMap::new()
    }.into_iter() // filter out entries whose backing file is gone
            .filter(|(path, _data)| path.exists())
            .collect();

    vprintln!("Indexing...");
    let timer = start_timer();
    for file in read_dir(PathBuf::from(&dir)).unwrap() {
        let file_path = file.unwrap().path();
        if file_path == index_file_path {
            continue;
        }
        let bytes = read_as_bytes(&file_path);
        let hash = format!("{:x}", md5::compute(&bytes));
        if index.contains_key(&file_path) && index.get(&file_path).unwrap().hash == hash {
            // println!(
            //     "file {} has not changed",
            //     file_path.to_string_lossy().to_string()
            // );
        } else {
            if let Ok(img) = image::load_from_memory(&bytes) {
                let rgb = average_color(img.to_rgb());
                index.insert(
                    file_path,
                    IndexData {
                        hash: hash,
                        average: rgb,
                    },
                );
            } else {
                vprintln!(
                    "Skipping unsupported file {}",
                    file_path.to_string_lossy().to_string()
                );
            }
        }
    }
    write_index(&index_file_path, &index);
    println!("Indexing complete");
    stop_timer(timer, "Indexing time: ");
    index
}

fn average_color(img: image::ImageBuffer<image::Rgb<u8>, std::vec::Vec<u8>>) -> (u8, u8, u8) {
    let (r, g, b) = img.enumerate_pixels()
        .fold((0u64, 0u64, 0u64), |acc, pixel| {
            (
                acc.0 + (pixel.2.data[0] as u64).pow(2),
                acc.1 + (pixel.2.data[1] as u64).pow(2),
                acc.2 + (pixel.2.data[2] as u64).pow(2),
            )
        });
    let total_pixels = img.width() * img.height();
    (
        ((r / total_pixels as u64) as f64).sqrt() as u8,
        ((g / total_pixels as u64) as f64).sqrt() as u8,
        ((b / total_pixels as u64) as f64).sqrt() as u8,
    )
}

fn get_parameters() -> clap::ArgMatches<'static> {
    App::new("Rust photomosaic builder")
        .version("0.1.0")
        .author("Isaac Post <post.isaac@gmail.com>")
        .about("Makes photomosaics")
        .arg(
            Arg::with_name("INPUT")
            .help("Sets the input file which will be recreated as a mosaic")
            .required(true)
            .index(1),
            )
        .arg(
            Arg::with_name("LIBRARY")
            .help("The directory containing the images to be used as mosaic tiles")
            .required(true)
            .index(2),
            )
        .arg(
            Arg::with_name("OUT_FILE")
            .help("The name of the output mosaic image")
            .required(true)
            .index(3),
            )
        .arg(
            Arg::with_name("verbose")
            .short("v")
            .long("verbose")
            .multiple(true)
            .help("Sets the level of verbosity"),
            )
        .arg(
            Arg::with_name("print-timings")
            .short("t")
            .long("print-timings")
            .help("Print timings"),
            )
        .arg(
            Arg::with_name("SIZE")
            .short("g")
            .long("pixel-group-size")
            .help(&format!("The size of the square regions, in pixels, which will be replaced in the source image. Defaults to {}", DEFAULT_PIXEL_GROUP_SIZE))
            .takes_value(true)
            .required(false),
            )
        .arg(
            Arg::with_name("MAGNIFICATION_FACTOR")
            .short("m")
            .long("magnification")
            .help(&format!("The factor by which the original image's dimensions are increased. Defaults to {}", DEFAULT_MAGNIFICATION))
            .takes_value(true)
            .required(false),
            )
        .get_matches()
}

fn start_timer() -> time::PreciseTime {
    PreciseTime::now()
}

fn stop_timer(timer: time::PreciseTime, message: &str) {
    if unsafe { PRINT_TIMING } {
        let duration = timer.to(PreciseTime::now()).num_milliseconds();
        println!("{}{}ms", message, duration);
    }
}
