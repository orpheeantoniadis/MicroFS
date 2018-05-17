use super::*;

impl MicroFS {
    pub fn add(&mut self, path: &str) {
        // read file
        let mut file_buffer = Vec::new();
        let mut file = match File::open(path) {
            Ok(f) => f,
            Err(_) => {
                println!("File not found.");
                return;
            }
        };
        file.read_to_end(&mut file_buffer).expect("Something went wrong when reading the file !");
        
        let mut entry = Entry::new(path);
        entry.size = file_buffer.len() as u32;
        let entry_block_size = file_buffer.len() / (SECTOR_SIZE * self.sb.block_size as usize) + 1;
        println!("Entry size (bytes) = {}", file_buffer.len());
        println!("Entry size (sectors) = {}", file_buffer.len() / SECTOR_SIZE + 1);
        println!("Entry size (blocks) = {}", entry_block_size);
        
        let mut blocks = self.empty_blocks(&mut entry);
        if blocks.len() < entry_block_size {
            println!("\nFile too large for File System.");
        } else {
            self.new_entries.push(entry);
            self.update_fat(&mut blocks, true);
        }
    }
}