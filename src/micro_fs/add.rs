use std::fs::File;
use std::io::prelude::*;
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
        
        let mut entries = self.empty_blocks(&mut entry);
        self.entries.push(entry);
        self.update_fat(&mut entries);
    }
    
    fn empty_blocks(&mut self, entry: &mut Entry) -> Vec<usize> {
        let entries_size = self.entries_size() / (SECTOR_SIZE * (self.sb.block_size as usize));
        
        let mut cnt = 0;
        let mut blocks = Vec::new();
        for i in (entries_size + (self.sb.root_entry as usize))..(*(self.fat)).len() {
            if (self.fat)[i] == 0xff {
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
    
    fn update_fat(&mut self, blocks: &mut Vec<usize>) {
        for i in 0..blocks.len() {
            if i == (blocks.len() - 1) {
                (self.fat)[blocks[i]] = 0;
            } else {
                (self.fat)[blocks[i]] = blocks[i+1] as u8;
            }
        }
    }
}