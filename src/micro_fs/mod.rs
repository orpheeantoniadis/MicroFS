use std::mem;
use std::str;
use std::io::prelude::*;
use std::io::SeekFrom;
use std::fs;
use std::fs::File;
use std::fs::OpenOptions;

pub mod utils;
use self::utils::*;

mod create;
mod add;
mod save;
mod del;
mod list;
mod info;

pub const MAGIC: u16 = 0x55aa;
pub const SECTOR_SIZE: usize = 0x200;

#[derive(Debug)]
pub struct MicroFS {
    pub image: String,
    pub sb: SuperBlock,
    pub fat: Vec<u8>,
    pub entries: Vec<Entry>
}
impl MicroFS {
    pub fn new(image: &str) -> MicroFS {
        unsafe {
            if fs::metadata(image).is_ok() {
                let mut raw_sb = [0; SECTOR_SIZE];
                let mut file = File::open(image).expect("File not found !");
                file.read(&mut raw_sb).expect("Something went wrong reading the file !");
                let signature = mem::transmute::<[u8;2], u16>([raw_sb[510], raw_sb[511]]);
                if signature == MAGIC {
                    // get super block
                    let bs = raw_sb[13];
                    let raw_label: Vec<u8> = Vec::from(&(raw_sb[82..90]));
                    let label = String::from_utf8(raw_label).unwrap();
                    let size = fs::metadata(image.clone()).expect("Failed getting metadata!").len() as usize;
                    let sb = SuperBlock::new(&label, bs, size);
                    let mut fs = MicroFS {
                        image: image.to_string(),
                        sb: sb,
                        fat: Vec::new(),
                        entries: Vec::new()
                    };
                    fs.set_fat();
                    fs.set_entries();
                    println!("\n{} is a valid image. You can modify it using the menu.", image);
                    return fs;
                }
            }
            println!("\n{} does not exist. You should create it first with the menu.", image);
            return MicroFS {
                image: image.to_string(),
                sb: mem::uninitialized(),
                fat: Vec::new(),
                entries:  Vec::new()
            };
        }
    }
}

#[derive(Debug, Copy, Clone)]
#[repr(C)]
pub struct SuperBlock {
    pub sector_size: u16,
    pub block_size: u8,
    pub fat_size: u32,
    pub version: u16,
    pub root_entry: u32,
    pub label: [u8;8],
    pub signature: u16
}
impl SuperBlock {
    pub fn new(label: &str, bs: u8, size: usize) -> SuperBlock {
        let mut raw_label : [u8;8] = [0;8];
        let mut i = 0;
        for byte in label.bytes() {
            raw_label[i] = byte;
            i += 1;
            if i == 8 { break; }
        }
        let fat_size = size / (SECTOR_SIZE * bs as usize);
        let mut root_entry = (SECTOR_SIZE + fat_size) / (SECTOR_SIZE * bs as usize);
        if ((SECTOR_SIZE + fat_size) % (SECTOR_SIZE * bs as usize)) != 0 {
            root_entry += 1;
        }
        SuperBlock {
            sector_size: SECTOR_SIZE as u16,
            block_size: bs,
            fat_size: fat_size as u32,
            version: 1,
            root_entry: root_entry as u32,
            label: raw_label,
            signature: MAGIC
        }
    }
}

#[derive(Debug, Copy, Clone)]
#[repr(C)]
pub struct Entry {
    pub name: [u8;26],
    pub start: u16,
    pub size: u32
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