use std::collections::HashMap;
use std::fs::read_dir;
use std::fs::File;
use std::io::Write;
use std::path::PathBuf;

mod lib;
use lib::index::*;

extern crate image;
use image::{FilterType, GenericImage, ImageBuffer, SubImage};
extern crate md5;
#[macro_use]
extern crate serde_derive;
extern crate serde_json;

fn main() {
    let source_image_path = "1.jpg";
    let library_dir_path = "./images";
    let out_file_path = "out.png";
    // group pixels into NxN sections for replacement
    let pixel_group_size = 10;
    // the original image dimensions will increase by this factor
    let magnification_factor = 1;

    let source_image = image::open(source_image_path)
        .expect(&format!("Error reading source image {}", source_image_path));
    let library = load_library(library_dir_path.to_string());
    let closest_image = |(r, g, b): (u8, u8, u8)| -> &PathBuf {
        library
            .iter()
            .min_by_key(|(pb, d)| {
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
    println!("New starting dimensions: {} x {}", new_width, new_height);
    let mut source_image = source_image
        .resize_exact(new_width, new_height, FilterType::Nearest)
        .to_rgb();

    let mut img = ImageBuffer::new(
        source_image.width() * magnification_factor,
        source_image.height() * magnification_factor,
    );
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
            let source_image = image::open(ci)
                .expect(&format!("Error reading image {}", source_image_path))
                .resize_exact(
                    pixel_group_size * magnification_factor,
                    pixel_group_size * magnification_factor,
                    FilterType::Nearest,
                );
            for x in 0..(pixel_group_size * magnification_factor) {
                for y in 0..(pixel_group_size * magnification_factor) {
                    img.put_pixel(
                        (x_offset * pixel_group_size * magnification_factor) + x,
                        (y_offset * pixel_group_size * magnification_factor) + y,
                        source_image.get_pixel(x, y)
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
    img.save(out_file_path);
}

fn load_library(dir: String) -> HashMap<PathBuf, IndexData> {
    let index_file_path = {
        let mut p = PathBuf::from(&dir);
        p.push(".mosaic_index");
        p
    };
    let mut index: HashMap<PathBuf, IndexData> = if index_file_path.exists() {
        println!("Existing index found");
        read_index(&index_file_path)
    } else {
        println!("No index found");
        HashMap::new()
    }.into_iter() // filter out entries whose backing file is gone
            .filter(|(path, _data)| path.exists())
            .collect();

    println!("Indexing...");
    for file in read_dir(PathBuf::from(&dir)).unwrap() {
        let file_path = file.unwrap().path();
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
                println!(
                    "Skipping unsupported file {}",
                    file_path.to_string_lossy().to_string()
                );
            }
        }
    }
    write_index(&index_file_path, &index);
    index
}

fn average_color(img: image::ImageBuffer<image::Rgb<u8>, std::vec::Vec<u8>>) -> (u8, u8, u8) {
    let (r, g, b) = img.enumerate_pixels()
        .fold((0u32, 0u32, 0u32), |acc, pixel| {
            (
                acc.0 + (pixel.2.data[0] as u32).pow(2),
                acc.1 + (pixel.2.data[1] as u32).pow(2),
                acc.2 + (pixel.2.data[2] as u32).pow(2),
            )
        });
    let total_pixels = img.width() * img.height();
    (
        ((r / total_pixels) as f64).sqrt() as u8,
        ((g / total_pixels) as f64).sqrt() as u8,
        ((b / total_pixels) as f64).sqrt() as u8,
    )
}
