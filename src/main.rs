use std::fs;
use std::fs::File;
use std::io::Read;

extern crate image;

extern crate md5;

extern crate serde;
#[macro_use]
extern crate serde_derive;
//#[macro_use]
extern crate serde_json;

#[derive(Serialize, Deserialize, Debug)]
struct IndexEntry {
    file_path: String,
    hash: String,
    average: (u8, u8, u8),
}

fn main() {
    let library = fs::read_dir("./images").unwrap();
    let mut index: Vec<IndexEntry> = vec![];
    for file in library {
        let file_path = file.unwrap().path();
        let bytes = {
            let mut file = File::open(&file_path).expect("file not found");
            let mut contents: Vec<u8> = vec![];
            file.read_to_end(&mut contents)
                .expect("something went wrong reading the file");
            contents
        };
        let hash = md5::compute(&bytes);
        let img = image::load_from_memory(&bytes).unwrap().to_rgb();
        let (r, g, b) = img.enumerate_pixels()
            .fold((0u32, 0u32, 0u32), |acc, pixel| {
                (
                    acc.0 + (pixel.2.data[0] as u32).pow(2),
                    acc.1 + (pixel.2.data[1] as u32).pow(2),
                    acc.2 + (pixel.2.data[2] as u32).pow(2),
                )
            });
        //println!("r: {:?}, g: {:?}, b: {:?}", r, g, b);
        let total_pixels = img.width() * img.height();
        let r = ((r / total_pixels) as f64).sqrt() as u8;
        let g = ((g / total_pixels) as f64).sqrt() as u8;
        let b = ((b / total_pixels) as f64).sqrt() as u8;
        //println!("r: {:?}, g: {:?}, b: {:?}", r, g, b);
        index.push(IndexEntry {
            file_path: file_path.to_string_lossy().to_string(),
            hash: format!("{:x}", hash),
            average: (r, g, b),
        });
    }
    let serialized = serde_json::to_string(&index).unwrap();
    println!("serialized = {}", serialized);
}
