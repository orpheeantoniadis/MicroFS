use super::*;

impl MicroFS {
    pub fn add(&mut self, filename: &str) {
        let mut entry = Entry::new(filename);
        
        // read file
        let mut file_buffer = Vec::new();
        let mut file = match File::open(filename) {
            Ok(f) => f,
            Err(_) => {
                println!("File not found.");
                return;
            }
        };
        file.read_to_end(&mut file_buffer).expect("Something went wrong when reading the file !");
        entry.size = file_buffer.len() as u32;
        let entry_block_size = file_buffer.len() / (SECTOR_SIZE * self.sb.block_size as usize) + 1;
        println!("Entry size (bytes) = {}", file_buffer.len());
        println!("Entry size (sectors) = {}", file_buffer.len() / SECTOR_SIZE + 1);
        println!("Entry size (blocks) = {}", entry_block_size);
        
        let mut entries = self.empty_blocks(&mut entry);
        if entries.len() < entry_block_size {
            println!("\nFile too large for File System.");
        } else {
            self.entries.push(entry);
            self.update_fat(&mut entries, true);
        }
    }
}