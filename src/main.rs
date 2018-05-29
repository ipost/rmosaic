use std::collections::HashMap;
use std::fs::read_dir;
use std::io::*;
use std::path::PathBuf;

mod lib;
use lib::index::*;
use lib::params::{color_caching, parameters, set_color_caching, set_print_timings, set_verbosity,
                  verbosity};
use lib::timing::*;

use std::sync::Mutex;
extern crate rayon;
use rayon::prelude::*;

extern crate image;
use image::{FilterType, GenericImage, ImageBuffer};
extern crate md5;
#[macro_use]
extern crate serde_derive;
extern crate serde_json;

const INDEX_FILENAME: &'static str = ".mosaic_index";
const PROGRESS_SECTIONS: usize = 20;

macro_rules! vprintln {
    ($fmt:expr) => { if verbosity(1) { println!($fmt) } };
    ($fmt:expr, $($arg:tt)*) => { if verbosity(1) { println!($fmt, $($arg)*) } };
}

macro_rules! vvprintln {
    ($fmt:expr) => { if verbosity(2) { println!($fmt) } };
    ($fmt:expr, $($arg:tt)*) => { if verbosity(2) { println!($fmt, $($arg)*) } };
}

fn main() {
    let (
        source_image_path,
        library_dir_path,
        out_file_path,
        print_timings_config,
        color_caching_config,
        verbosity_config,
        pixel_group_size,
        magnification_factor,
        threads,
    ) = parameters();
    set_print_timings(print_timings_config);
    set_color_caching(color_caching_config);
    set_verbosity(verbosity_config);
    rayon::ThreadPoolBuilder::new()
        .num_threads(threads)
        .build_global()
        .unwrap();

    vprintln!("Using magnification: {}", magnification_factor);
    vprintln!("Using pixel group size: {}", pixel_group_size);
    vprintln!("Recreating: {}", source_image_path);
    vprintln!("Using library at: {}", library_dir_path);
    vprintln!("Running with {} threads", threads);
    vprintln!(
        "Color caching is {}",
        if color_caching() {
            "ENABLED"
        } else {
            "DISABLED"
        }
    );

    let source_image = image::open(&source_image_path)
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
    vvprintln!("New starting dimensions: {} x {}", new_width, new_height);
    let source_image: image::ImageBuffer<image::Rgb<u8>, std::vec::Vec<u8>> = source_image
        .resize_exact(new_width, new_height, FilterType::Nearest)
        .to_rgb();

    let result_image = Mutex::new(ImageBuffer::new(
        source_image.width() * magnification_factor,
        source_image.height() * magnification_factor,
    ));
    let library_cache: Mutex<HashMap<&PathBuf, image::DynamicImage>> = Mutex::new(HashMap::new());
    let color_cache = Mutex::new(HashMap::new());
    vprintln!("Building image...");
    let timer = start_timer();
    let regions = {
        let mut regions = Vec::with_capacity(
            (source_image.width() * source_image.height() / (pixel_group_size.pow(2))) as usize,
        );
        for x_region in 0..(source_image.width() / pixel_group_size) {
            for y_region in 0..(source_image.height() / pixel_group_size) {
                regions.push((x_region, y_region));
            }
        }
        regions
    };
    let total_regions = regions.len();
    let progress_increment = total_regions / PROGRESS_SECTIONS;
    vvprintln!("{} regions", total_regions);
    let progress = Mutex::new(0);
    regions.par_iter().for_each(|(x_region, y_region)| {
        let region_pixels = sub_image_pixels(
            &source_image,
            x_region * pixel_group_size,
            y_region * pixel_group_size,
            pixel_group_size,
            pixel_group_size,
        );
        let average_color = average_color(region_pixels.iter().collect());

        let closest_image_path = if color_caching() {
            let mut l_color_cache = color_cache.lock().unwrap();
            if l_color_cache.contains_key(&average_color) {
                l_color_cache.get(&average_color).unwrap()
            } else {
                drop(l_color_cache);
                let closest_image_path = closest_image(average_color);
                color_cache
                    .lock()
                    .unwrap()
                    .insert(average_color, closest_image_path);
                closest_image_path
            }
        } else {
            closest_image(average_color)
        };

        let source_image: image::DynamicImage = {
            let mut l_library_cache = library_cache.lock().unwrap();
            if l_library_cache.contains_key(&closest_image_path) {
                l_library_cache.get(closest_image_path).unwrap().clone()
            } else {
                drop(l_library_cache);
                let i = image::open(closest_image_path)
                    .expect(&format!("Error reading image {}", source_image_path))
                    .resize_exact(
                        pixel_group_size * magnification_factor,
                        pixel_group_size * magnification_factor,
                        FilterType::Nearest,
                    );
                let i_clone = i.clone();
                library_cache.lock().unwrap().insert(closest_image_path, i);
                i_clone
            }
        };

        let mut pixels = vec![];
        for x in 0..(pixel_group_size * magnification_factor) {
            for y in 0..(pixel_group_size * magnification_factor) {
                pixels.push((
                    (x_region * pixel_group_size * magnification_factor) + x,
                    (y_region * pixel_group_size * magnification_factor) + y,
                    source_image.get_pixel(x, y),
                ));
            }
        }
        let mut result_image = result_image.lock().unwrap();
        for (x, y, p) in pixels {
            result_image.put_pixel(x, y, p);
        }
        let mut p = progress.lock().unwrap();
        *p += 1;
        if *p % progress_increment == 0 {
            let bar = *p / progress_increment;
            print!(
                "{}[{}{}] {}%",
                "\x08".repeat(PROGRESS_SECTIONS + 9),
                "=".repeat(bar),
                " ".repeat(PROGRESS_SECTIONS - bar),
                100 * *p / total_regions
            );
            std::io::stdout()
                .flush()
                .expect("Something went wrong while writing to STDOUT");
        }
    });
    println!("");
    stop_timer(timer, "Image build time: ");
    let timer = start_timer();
    result_image
        .lock()
        .unwrap()
        .save(&out_file_path)
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
    let index: Mutex<HashMap<PathBuf, IndexData>> = Mutex::new(
        if index_file_path.exists() {
        vprintln!("Existing index found");
        read_index(&index_file_path)
    } else {
        vprintln!("No index found");
        HashMap::new()
    }.into_iter() // filter out entries whose backing file is gone
            .filter(|(path, _data)| path.exists())
            .collect(),
    );

    vprintln!("Indexing...");
    let timer = start_timer();
    let dir_iter = read_dir(PathBuf::from(&dir))
        .expect("Unable to read image library dir")
        .collect::<Vec<std::result::Result<std::fs::DirEntry, std::io::Error>>>()
        .into_par_iter();
    dir_iter.for_each(|file| {
        let file_path = file.unwrap().path();
        if file_path == index_file_path {
            return;
        }
        let bytes = read_as_bytes(&file_path);
        let hash = format!("{:x}", md5::compute(&bytes));
        let l_index = index.lock().unwrap();
        if l_index.contains_key(&file_path) && l_index.get(&file_path).unwrap().hash == hash {
            vvprintln!(
                "file {} has not changed",
                file_path.to_string_lossy().to_string()
            );
        } else {
            drop(l_index);
            vvprintln!("Indexing file {}", file_path.to_string_lossy().to_string());
            if let Ok(img) = image::load_from_memory(&bytes) {
                let rgb = average_color(img.to_rgb().pixels().collect());
                index.lock().unwrap().insert(
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
    });
    let index = index.into_inner().unwrap();
    write_index(&index_file_path, &index);
    println!("Indexing complete");
    stop_timer(timer, "Indexing time: ");
    index
}

fn average_color(pixels: Vec<&image::Rgb<u8>>) -> (u8, u8, u8) {
    let total_pixels = pixels.len();
    let (r, g, b) = pixels.into_iter().fold((0u64, 0u64, 0u64), |acc, pixel| {
        (
            acc.0 + (pixel.data[0] as u64).pow(2),
            acc.1 + (pixel.data[1] as u64).pow(2),
            acc.2 + (pixel.data[2] as u64).pow(2),
        )
    });
    (
        ((r / total_pixels as u64) as f64).sqrt() as u8,
        ((g / total_pixels as u64) as f64).sqrt() as u8,
        ((b / total_pixels as u64) as f64).sqrt() as u8,
    )
}

fn sub_image_pixels(
    img: &image::ImageBuffer<image::Rgb<u8>, std::vec::Vec<u8>>,
    x: u32,
    y: u32,
    width: u32,
    height: u32,
) -> Vec<image::Rgb<u8>> {
    let mut rgbs = Vec::with_capacity((width * height) as usize);
    for x_new in 0..width {
        for y_new in 0..height {
            rgbs.push(img[(x + x_new, y + y_new)]);
        }
    }
    rgbs
}
