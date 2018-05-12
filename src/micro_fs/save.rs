use super::*;

impl MicroFS {
    pub fn save(&mut self) {
        unsafe {
            let mut image = OpenOptions::new().read(true).write(true).open(self.image.clone()).expect("File not found !");
            image.seek(SeekFrom::Start(SECTOR_SIZE as u64)).expect("File seek failed !");
            image.write_all(&(self.fat)).expect("Failed to write in file!");
            
            image.seek(SeekFrom::Start(self.root_entry() as u64)).expect("File seek failed !");
            let size = fs::metadata(self.image.clone()).expect("Failed getting metadata!").len() as usize;
            let rest = (size / SECTOR_SIZE) - (self.sb.root_entry as usize * self.sb.block_size as usize);
            for _i in 0..rest {
                image.write_all(&[0;SECTOR_SIZE]).expect("Failed to write in file!");
            }
            
            image.seek(SeekFrom::Start(self.root_entry() as u64)).expect("File seek failed !");
            for entry in self.entries.clone() {
                image.write_all(&(entry.name)).expect("Failed to write in file!");
                image.write_all(&(mem::transmute::<u16, [u8; 2]>(entry.start))).expect("Failed to write in file!");
                image.write_all(&(mem::transmute::<u32, [u8; 4]>(entry.size))).expect("Failed to write in file!");
                
                let mut file_buffer = Vec::new();
                let s = bytes_to_str(&(entry.name));
                let mut file = File::open(s).expect("File not found !");
                file.read_to_end(&mut file_buffer).expect("Something went wrong reading the file !");
                let mut blocks = self.get_blocks(&mut entry.clone());
                self.write_data(&mut blocks, file_buffer);
            }
        }
    }
}