use std::fs::File;
use std::io::prelude::*;
use super::*;

impl MicroFS {
    pub fn add(&mut self, filename: &str) {
        let mut entry = Entry::new(filename);
        let mut buffer = vec![0; 10];
        let mut file = File::open(filename).expect("File not found !");
        file.read_to_end(&mut buffer).expect("Something went wrong reading the file !");
        entry.size = buffer.len() as u32;
        println!("Entry size (bytes) = {}", buffer.len());
        println!("Entry size (sectors) = {}", buffer.len() / SECTOR_SIZE + 1);
    }
    
    // fn find_empty_entry(&mut self) {
    // 
    // }
}

#[repr(C, packed)]
struct Entry {
    name: [u8;26],
    start: u16,
    size: u32
}
impl Entry {
    fn new(name: &str) -> Entry {
        let mut raw_name : [u8;26] = [0;26];
        let mut i = 0;
        for byte in name.bytes() {
            raw_name[i] = byte;
            i += 1;
            if i == 26 { break; }
        }
        Entry {
            name: raw_name,
            start: 0,
            size: 0
        }
    }
}

