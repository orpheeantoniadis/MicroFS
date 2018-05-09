use micro_fs::add::Entry;
use std::mem;
use std::fs;
use std::fs::File;
use std::io::prelude::*;
use std::io::SeekFrom;

mod create;
mod add;

use self::create::*;

const MAGIC: u16 = 0xaa55;
const SECTOR_SIZE: usize = 0x200;

pub struct MicroFS {
    image: String,
    sb: SuperBlock,
    fat: *mut [u8],
    entries: *mut Vec<Entry>
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
                        fat: mem::uninitialized(),
                        entries:  mem::uninitialized()
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
                fat: mem::uninitialized(),
                entries:  mem::uninitialized()
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
        self.fat = &mut (raw_fat[SECTOR_SIZE..(SECTOR_SIZE+self.fat_size())]);
    }
    
    fn set_entries(&mut self) {
        unsafe {
            let mut file = File::open(self.image.clone()).expect("File not found !");
            
            let mut entries = Vec::new();
            let mut cnt = 0;
            println!("Size = {}", self.entries_size());
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
                    entries.push(Entry { name: raw_name, start: start, size: size });
                }
                cnt += mem::size_of::<Entry>();
            }
            self.entries = &mut entries;
        }
    }
}