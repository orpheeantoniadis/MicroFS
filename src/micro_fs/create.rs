use std::fs::File;
use std::io::prelude::*;
use std::mem;
use super::MicroFS;

const MAGIC: u16 = 0xaa55;
const SECTOR_SIZE: usize = 0x200;

impl MicroFS {
    pub fn create(&mut self, image: &str, label: &str, bs: u8, size: usize) {
        self.file = File::create(image).expect("Failed to create file!");
        self.write_super_block(label, bs);
        for _i in 0..self.sb.fat_size {
            self.file.write_all(&[0xff;SECTOR_SIZE]).expect("Failed to write in file!");
        }
        let rest = (size / SECTOR_SIZE) - (self.sb.fat_size as usize + 1);
        for _i in 0..rest {
            self.file.write_all(&[0;SECTOR_SIZE]).expect("Failed to write in file!");
        }
    }

    fn write_super_block(&mut self, label: &str, bs: u8) {
        unsafe {
            self.sb = SuperBlock::new(label, bs);
            self.file.write_all(&[0;11]).expect("Failed to write in file!");
            self.file.write_all(&(mem::transmute::<u16, [u8; 2]>(self.sb.sector_size))).expect("Failed to write in file!");
            self.file.write_all(&(mem::transmute::<u8, [u8; 1]>(self.sb.block_size))).expect("Failed to write in file!");
            self.file.write_all(&[0;22]).expect("Failed to write in file!");
            self.file.write_all(&(mem::transmute::<u32, [u8; 4]>(self.sb.fat_size))).expect("Failed to write in file!");
            self.file.write_all(&[0;2]).expect("Failed to write in file!");
            self.file.write_all(&(mem::transmute::<u16, [u8; 2]>(self.sb.version))).expect("Failed to write in file!");
            self.file.write_all(&(mem::transmute::<u32, [u8; 4]>(self.sb.root_entry))).expect("Failed to write in file!");
            self.file.write_all(&[0;34]).expect("Failed to write in file!");
            self.file.write_all(&(self.sb.label)).expect("Failed to write in file!");
            self.file.write_all(&[0;420]).expect("Failed to write in file!");
            self.file.write_all(&(mem::transmute::<u16, [u8; 2]>(self.sb.signature))).expect("Failed to write in file!");
        }
    }
}

#[repr(C, packed)]
pub struct SuperBlock {
    sector_size: u16,
    block_size: u8,
    fat_size: u32,
    version: u16,
    root_entry: u32,
    label: [u8;8],
    signature: u16
}
impl SuperBlock {
    fn new(label: &str, bs: u8) -> SuperBlock {
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

#[repr(C, packed)]
struct Entry {
    name: [u8;26],
    start: u16,
    size: u32
}