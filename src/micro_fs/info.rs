use super::*;

impl MicroFS {
    pub fn info(&mut self) {
        println!("\nFS info :");
        println!("Label : {}", bytes_to_str(&self.sb.label));
        println!("Version : {}", self.sb.version);
        println!("Sector size : {} bytes", self.sb.sector_size);
        println!("Block size : {} sector(s)", self.sb.block_size);
        println!("FAT size : {} bytes", self.sb.fat_size);
        println!("Root entry : {}", self.sb.root_entry);
    }
}