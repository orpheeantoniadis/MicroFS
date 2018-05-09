use std::fs::File;
use std::io::prelude::*;
use std::io::SeekFrom;
use std::fs::OpenOptions;
use std::mem;
use super::*;

impl MicroFS {
    pub fn add(&mut self, filename: &str) {
        let mut entry = Entry::new(filename);
        
        // read file
        let mut file_buffer = Vec::new();
        let mut file = File::open(filename).expect("File not found !");
        file.read_to_end(&mut file_buffer).expect("Something went wrong reading the file !");
        entry.size = file_buffer.len() as u32;
        println!("Entry size (bytes) = {}", file_buffer.len());
        println!("Entry size (sectors) = {}", file_buffer.len() / SECTOR_SIZE + 1);
        println!("Entry size (blocks) = {}", file_buffer.len() / (SECTOR_SIZE * self.sb.block_size as usize) + 1);
        
        let mut entries = self.empty_entries(&mut entry);
        self.write_fat_entries(&mut entries);
        self.write_root_entry(&mut entry);
        self.write_data(&mut entries, file_buffer);
    }
    
    fn empty_entries(&mut self, entry: &mut Entry) -> Vec<usize> {
        unsafe {
            let entries_sector_size = (self.sb.fat_size as usize) * SECTOR_SIZE * mem::size_of::<Entry>();
            let entries_sector_size = entries_sector_size / (SECTOR_SIZE * (self.sb.block_size as usize));
            
            let mut cnt = 0;
            let mut blocks = Vec::new();
            for i in (entries_sector_size + (self.sb.root_entry as usize))..(*(self.fat)).len() {
                if (*(self.fat))[i] == 0xff {
                    if cnt == 0 {
                        entry.start = i as u16;
                    }
                    blocks.push(i);
                    cnt += 1;
                }
                if cnt >= (entry.size / SECTOR_SIZE as u32 + 1) {
                    break;
                }
            }
            return blocks;
        }
    }
    
    fn write_fat_entries(&mut self, entries: &mut Vec<usize>) {
        unsafe {
            let mut file = OpenOptions::new().read(true).write(true).open(self.image.clone()).expect("File not found !");
            for i in 0..entries.len() {
                if i == (entries.len() - 1) {
                    (*(self.fat))[entries[i]] = 0;
                } else {
                    (*(self.fat))[entries[i]] = entries[i+1] as u8;
                }
            }
            file.seek(SeekFrom::Start(SECTOR_SIZE as u64)).expect("File seek failed !");
            match file.write_all(&*(self.fat)) {
                Ok(..) => {},
                Err(e) => println!("{}", e),
            }
        }
    }
    
    fn write_root_entry(&mut self, entry: &mut Entry) {
        unsafe {
            let mut fs_buffer = Vec::new();
            let mut file = OpenOptions::new().read(true).write(true).open(self.image.clone()).expect("File not found !");
            file.read_to_end(&mut fs_buffer).expect("Something went wrong reading the file !");
            let offset = (self.sb.root_entry as usize) * SECTOR_SIZE;
            let size = (self.sb.fat_size as usize) * SECTOR_SIZE * mem::size_of::<Entry>();
            let entries = &mut (fs_buffer[offset..(offset+size)]);
            let mut i = 0;
            while i < size {
                if entries[i] == 0 {
                    file.seek(SeekFrom::Start((offset + i) as u64)).expect("File seek failed !");
                    file.write_all(&(entry.name)).expect("Failed to write in file!");
                    file.write_all(&(mem::transmute::<u16, [u8; 2]>(entry.start))).expect("Failed to write in file!");
                    file.write_all(&(mem::transmute::<u32, [u8; 4]>(entry.size))).expect("Failed to write in file!");
                    break;
                }
                i += mem::size_of::<Entry>();
            }
        }
    }
    
    fn write_data(&mut self, entries: &mut Vec<usize>, data: Vec<u8>) {
        let mut file = OpenOptions::new().read(true).write(true).open(self.image.clone()).expect("File not found !");
        let mut cnt = 0;
        for entry in entries.iter() {
            let offset = entry * (self.sb.block_size as usize) * SECTOR_SIZE;
            file.seek(SeekFrom::Start(offset as u64)).expect("File seek failed !");
            
            let data_block_start = cnt * (self.sb.block_size as usize) * SECTOR_SIZE;
            let mut data_block_end = data_block_start + (self.sb.block_size as usize) * SECTOR_SIZE;
            if data_block_end > data.len() {
                data_block_end = data.len();
            }
            file.write_all(&(data[data_block_start..data_block_end])).expect("Failed to write in file!");
            cnt += 1;
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

