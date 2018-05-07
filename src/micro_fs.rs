use std::fs::File;
use std::io::prelude::*;
use std::mem;

const IMAGE: &str = "micro_fs.iso";
const MAGIC: u16 = 0xaa55;

pub fn create() {
    write_super_block();
}

fn write_super_block() {
    unsafe {
        let superblock = SuperBlock::new();
        let mut file = File::create(IMAGE).expect("Failed to create file!");
        file.write_all(&[0;11]).expect("Failed to write in file!");
        file.write_all(&(mem::transmute::<u16, [u8; 2]>(superblock.block_size))).expect("Failed to write in file!");
        file.write_all(&[0;23]).expect("Failed to write in file!");
        file.write_all(&(mem::transmute::<u32, [u8; 4]>(superblock.fat_size))).expect("Failed to write in file!");
        file.write_all(&[0;2]).expect("Failed to write in file!");
        file.write_all(&(mem::transmute::<u16, [u8; 2]>(superblock.version))).expect("Failed to write in file!");
        file.write_all(&(mem::transmute::<u32, [u8; 4]>(superblock.root_entry))).expect("Failed to write in file!");
        file.write_all(&[0;34]).expect("Failed to write in file!");
        file.write_all(&(superblock.label)).expect("Failed to write in file!");
        file.write_all(&[0;420]).expect("Failed to write in file!");
        file.write_all(&(mem::transmute::<u16, [u8; 2]>(superblock.signature))).expect("Failed to write in file!");
    }
}

#[repr(C, packed)]
struct SuperBlock {
    block_size: u16,
    fat_size: u32,
    version: u16,
    root_entry: u32,
    label: [u8;8],
    signature: u16
}
impl SuperBlock {
    fn new() -> SuperBlock {
        SuperBlock {
            block_size: 512,
            fat_size: 1,
            version: 1,
            root_entry: 2,
            label: [b'M', b'i', b'c', b'r', b'o', b'F', b'S', b'\0'],
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