use std::mem;
use std::fs;
use std::fs::File;
use std::io::prelude::*;

mod create;
mod add;

use self::create::*;

const MAGIC: u16 = 0xaa55;
const SECTOR_SIZE: usize = 0x200;

pub struct MicroFS {
    file: File,
    sb: SuperBlock
}
impl MicroFS {
    pub fn new(image: &str) -> MicroFS {
        unsafe {
            if fs::metadata(image).is_ok() {
                let mut buffer = [0; SECTOR_SIZE];
                let mut file = File::open(image).expect("File not found !");
                file.read(&mut buffer).expect("Something went wrong reading the file !");
                let signature = mem::transmute::<[u8;2], u16>([buffer[510], buffer[511]]);
                if signature == MAGIC {
                    let bs = buffer[13];
                    let vector: Vec<u8> = Vec::from(&(buffer[82..90]));
                    let label = String::from_utf8(vector).unwrap();
                    let sb = SuperBlock::new(&label, bs);
                    println!("\n{} already exists, you can modify it using the menu", image);
                    return MicroFS { file: file, sb: sb };
                }
            }
            println!("\n{} does not exist, create it first with the menu", image);
            return MicroFS { file: mem::uninitialized(), sb: mem::uninitialized() };
        }
    }
}