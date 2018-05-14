use super::*;

impl MicroFS {
    pub fn list(&mut self) {
        println!("\nFiles :");
        for entry in self.entries.clone() {
            println!("{}", bytes_to_str(&entry.name));
            println!("{} bytes\n", entry.size);
        }
    }
}