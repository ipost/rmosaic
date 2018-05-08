use std::collections::HashMap;
use std::fs::read_dir;
use std::path::PathBuf;

mod lib;
use lib::index::*;

extern crate image;
extern crate md5;
#[macro_use]
extern crate serde_derive;
extern crate serde_json;

fn main() {
    let library = load_library("./images".to_string());
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
            println!(
                "file {} has not changed",
                file_path.to_string_lossy().to_string()
            );
        } else {
            if let Ok(img) = image::load_from_memory(&bytes) {
                let img = img.to_rgb();
                let (r, g, b) = img.enumerate_pixels()
                    .fold((0u32, 0u32, 0u32), |acc, pixel| {
                        (
                            acc.0 + (pixel.2.data[0] as u32).pow(2),
                            acc.1 + (pixel.2.data[1] as u32).pow(2),
                            acc.2 + (pixel.2.data[2] as u32).pow(2),
                        )
                    });
                let total_pixels = img.width() * img.height();
                let rgb = (
                    ((r / total_pixels) as f64).sqrt() as u8,
                    ((g / total_pixels) as f64).sqrt() as u8,
                    ((b / total_pixels) as f64).sqrt() as u8,
                );
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
