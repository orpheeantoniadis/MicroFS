use std::io::prelude::*;
use std::io::SeekFrom;
use std::fs::OpenOptions;
use std::fs::metadata;
use std::str;
use super::*;

impl MicroFS {
    pub fn save(&mut self) {
        unsafe {
            let mut image = OpenOptions::new().read(true).write(true).open(self.image.clone()).expect("File not found !");
            image.seek(SeekFrom::Start(SECTOR_SIZE as u64)).expect("File seek failed !");
            image.write_all(&(self.fat)).expect("Failed to write in file!");
            
            // let size = metadata(self.image.clone()).expect("Failed getting metadata!").len() as usize;
            // let rest = (size / SECTOR_SIZE) - (self.sb.fat_size as usize + 1);
            // for _i in 0..rest {
            //     image.write_all(&[0;SECTOR_SIZE]).expect("Failed to write in file!");
            // }
            // 
            // image.seek(SeekFrom::Start(self.root_entry() as u64)).expect("File seek failed !");
            
            for entry in self.entries.clone() {
                image.write_all(&(entry.name)).expect("Failed to write in file!");
                image.write_all(&(mem::transmute::<u16, [u8; 2]>(entry.start))).expect("Failed to write in file!");
                image.write_all(&(mem::transmute::<u32, [u8; 4]>(entry.size))).expect("Failed to write in file!");
                
                let mut file_buffer = Vec::new();
                let s = bytes_to_str(&(entry.name));
                let mut file = File::open(s).expect("File not found !");
                file.read_to_end(&mut file_buffer).expect("Something went wrong reading the file !");
                let mut blocks = self.get_blocks(&mut entry.clone());
                self.write_data(&mut blocks, file_buffer);
            }
        }
    }
    
    fn get_blocks(&mut self, entry: &mut Entry) -> Vec<usize> {
        let mut blocks = Vec::new();
        let mut block = entry.start as usize;
        blocks.push(block);
        loop {
            block = self.fat[block] as usize;
            if block != 0 {
                blocks.push(block);
            } else {
                break;
            }
        }
        return blocks;
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

fn bytes_to_str(bytes: &[u8]) -> &str {
    let mut cnt = 0;
    for &byte in bytes {
        if byte == 0 {
            break;
        }
        cnt += 1;
    }
    str::from_utf8(&bytes[0..cnt]).expect("Found invalid UTF-8")
}