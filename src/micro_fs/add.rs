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
        
        // find fat entries
        let entries = self.get_fat_entries(&mut entry);
        
        // write fat entries
        self.write_fat_entries(entries);
        
        // write root entry
        self.write_root_entry(&mut entry);
        
        
    }
    
    fn get_fat_entries(&mut self, entry: &mut Entry) -> Vec<usize> {
        let mut fs_buffer = Vec::new();
        let mut file = File::open(self.image.clone()).expect("File not found !");
        file.read_to_end(&mut fs_buffer).expect("Something went wrong reading the file !");
        let fat = &mut (fs_buffer[SECTOR_SIZE..(self.sb.fat_size as usize + 1)*SECTOR_SIZE]);
        let mut cnt = 0;
        let mut blocks = Vec::new();
        for i in (self.sb.root_entry as usize)..fat.len() {
            if fat[i] == 0xff {
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
    
    fn write_fat_entries(&mut self, entries: Vec<usize>) {
        let mut fs_buffer = Vec::new();
        let mut file = OpenOptions::new().read(true).write(true).open(self.image.clone()).expect("File not found !");
        file.read_to_end(&mut fs_buffer).expect("Something went wrong reading the file !");
        let fat = &mut (fs_buffer[SECTOR_SIZE..(self.sb.fat_size as usize + 1)*SECTOR_SIZE]);
        for i in 0..entries.len() {
            if i == (entries.len() - 1) {
                fat[entries[i]] = 0;
            } else {
                fat[entries[i]] = entries[i+1] as u8;
            }
        }
        file.seek(SeekFrom::Start(SECTOR_SIZE as u64)).expect("File seek failed !");
        match file.write_all(fat) {
            Ok(..) => {},
            Err(e) => println!("{}", e),
        }
    }
    
    fn write_root_entry(&mut self, entry: &mut Entry) {
        unsafe {
            let mut fs_buffer = Vec::new();
            let mut file = OpenOptions::new().read(true).write(true).open(self.image.clone()).expect("File not found !");
            file.read_to_end(&mut fs_buffer).expect("Something went wrong reading the file !");
            let offset = (self.sb.fat_size as usize + 1) * SECTOR_SIZE;
            let size = (self.sb.fat_size as usize) * SECTOR_SIZE * mem::size_of::<Entry>();
            let entries = &mut (fs_buffer[offset..(offset+size)]);
            println!("Offset = {}", offset);
            println!("Size = {}", size);
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

