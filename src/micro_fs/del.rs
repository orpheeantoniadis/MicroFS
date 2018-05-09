use super::*;

impl MicroFS {
    pub fn del(&mut self, filename: &str) {
        for entry in self.entries.clone() {
            if bytes_to_str(&entry.name) == filename.clone() {
                let mut blocks = self.get_blocks(&mut entry.clone());
                self.update_fat(&mut blocks, false);
            }
        }
        self.entries.retain(|e| bytes_to_str(&e.name) != filename.clone()); 
    }
}