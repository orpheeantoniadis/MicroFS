use std::mem;
use std::fs;
use std::fs::File;
use std::io::prelude::*;
use std::io::SeekFrom;
use std::str;

mod create;
mod add;
mod save;
mod del;

const MAGIC: u16 = 0xaa55;
const SECTOR_SIZE: usize = 0x200;

pub struct MicroFS {
    image: String,
    sb: SuperBlock,
    fat: Vec<u8>,
    entries: Vec<Entry>
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
                    let sb = SuperBlock::new(&label, bs);
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
    
    fn fat_size(&mut self) -> usize {
        (self.sb.fat_size as usize) * SECTOR_SIZE
    }
    
    fn root_entry(&mut self) -> usize {
        self.sb.root_entry as usize * self.sb.block_size as usize * SECTOR_SIZE
    }
    
    fn entries_size(&mut self) -> usize {
        (self.sb.fat_size as usize) * SECTOR_SIZE * mem::size_of::<Entry>()
    }
    
    fn set_fat(&mut self) {
        let mut file = File::open(self.image.clone()).expect("File not found !");
        let mut raw_fat = Vec::new();
        file.read_to_end(&mut raw_fat).expect("Something went wrong reading the file !");
        for i in SECTOR_SIZE..(SECTOR_SIZE+self.fat_size()) {
            self.fat.push(raw_fat[i]);
        }
    }
    
    fn set_entries(&mut self) {
        unsafe {
            let mut file = File::open(self.image.clone()).expect("File not found !");
            
            let mut cnt = 0;
            while cnt < self.entries_size() {
                let mut raw_name = [0;26];
                file.seek(SeekFrom::Start((self.root_entry() + cnt) as u64)).expect("File seek failed !");
                file.read(&mut raw_name).expect("Something went wrong reading the file !");
                if raw_name[0] != 0 {
                    let mut raw_start = [0;2];
                    file.read(&mut raw_start).expect("Something went wrong reading the file !");
                    let start = mem::transmute::<[u8;2], u16>(raw_start);
                    let mut raw_size = [0;4];
                    file.read(&mut raw_size).expect("Something went wrong reading the file !");
                    let size = mem::transmute::<[u8;4], u32>(raw_size);
                    self.entries.push(Entry { name: raw_name, start: start, size: size });
                }
                cnt += mem::size_of::<Entry>();
            }
        }
    }
}

#[repr(C, packed)]
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
    pub fn new(label: &str, bs: u8) -> SuperBlock {
        let mut raw_label : [u8;8] = [0;8];
        let mut i = 0;
        for byte in label.bytes() {
            raw_label[i] = byte;
            i += 1;
            if i == 8 { break; }
        }
        // fat is one block long + rest of first block
        // so block_size + block_size - 1 (superblock is 1 sector long)
        let fat_size = 2 * (bs as u32) - 1;
        SuperBlock {
            sector_size: SECTOR_SIZE as u16,
            block_size: bs,
            fat_size: fat_size,
            version: 1,
            root_entry: 2,
            label: raw_label,
            signature: MAGIC
        }
    }
}

#[derive(Debug, Copy, Clone)]
#[repr(C, packed)]
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

pub fn bytes_to_str(bytes: &[u8]) -> &str {
    let mut cnt = 0;
    for &byte in bytes {
        if byte == 0 {
            break;
        }
        cnt += 1;
    }
    str::from_utf8(&bytes[0..cnt]).expect("Found invalid UTF-8")
}