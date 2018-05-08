use std::collections::HashMap;
use std::fs::File;
use std::io::Read;
use std::io::Write;
use std::path::PathBuf;

use serde_json;

#[derive(Serialize, Deserialize, Debug)]
pub struct IndexData {
    pub hash: String,
    pub average: (u8, u8, u8),
}

pub fn write_index(index_path: &PathBuf, index: &HashMap<PathBuf, IndexData>) {
    let mut file = File::create(&index_path).expect("file not found");
    let contents = serde_json::to_string(&index).unwrap();
    file.write(contents.as_bytes())
        .expect("something went wrong while writing the file");
}

pub fn read_index(index_path: &PathBuf) -> HashMap<PathBuf, IndexData> {
    let mut file = File::open(&index_path).expect("file not found");
    let mut contents: String = String::new();
    file.read_to_string(&mut contents)
        .expect("something went wrong while reading the file");
    serde_json::from_str(&contents).unwrap()
}

pub fn read_as_bytes(file_path: &PathBuf) -> Vec<u8> {
    let mut file = File::open(&file_path).expect("file not found");
    let mut contents: Vec<u8> = vec![];
    file.read_to_end(&mut contents)
        .expect("something went wrong reading the file");
    contents
}
