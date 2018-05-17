use super::*;

impl MicroFS {
    pub fn list(&mut self) {
        println!("\nFiles :");
        for entry in self.entries.clone() {
            println!("{}", bytes_to_str(&entry.name));
            println!("{} bytes\n", entry.size);
        }
        for entry in self.new_entries.clone() {
            println!("{}", bytes_to_str(&entry.name));
            println!("{} bytes\n", entry.size);
        }
    }
}