use std::fs::OpenOptions;
use std::io::SeekFrom;
use super::*;

impl MicroFS {
    pub fn fat_size(&mut self) -> usize {
        (self.sb.fat_size as usize) * SECTOR_SIZE
    }
    
    pub fn root_entry(&mut self) -> usize {
        self.sb.root_entry as usize * self.sb.block_size as usize * SECTOR_SIZE
    }
    
    pub fn entries_size(&mut self) -> usize {
        self.fat_size() * mem::size_of::<Entry>()
    }
    
    pub fn set_fat(&mut self) {
        let mut file = File::open(self.image.clone()).expect("File not found !");
        let mut raw_fat = Vec::new();
        file.read_to_end(&mut raw_fat).expect("Something went wrong reading the file !");
        for i in SECTOR_SIZE..(SECTOR_SIZE+self.fat_size()) {
            self.fat.push(raw_fat[i]);
        }
    }
    
    pub fn set_entries(&mut self) {
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
    
    pub fn empty_blocks(&mut self, entry: &mut Entry) -> Vec<usize> {
        let entries_blocks_size = self.entries_size() / (SECTOR_SIZE * (self.sb.block_size as usize));
        let data_start = entries_blocks_size + (self.sb.root_entry as usize);
        
        let mut cnt = 0;
        let mut blocks = Vec::new();
        for i in data_start..self.fat.len() {
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
    
    pub fn get_blocks(&mut self, entry: &mut Entry) -> Vec<usize> {
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
    
    pub fn update_fat(&mut self, blocks: &mut Vec<usize>, add: bool) {
        for i in 0..blocks.len() {
            if add {
                if i == (blocks.len() - 1) {
                    (self.fat)[blocks[i]] = 0;
                } else {
                    (self.fat)[blocks[i]] = blocks[i+1] as u8;
                }
            } else {
                (self.fat)[blocks[i]] = 0xff;
            }
        }
    }
    
    pub fn write_data(&mut self, entries: &mut Vec<usize>, data: Vec<u8>) {
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