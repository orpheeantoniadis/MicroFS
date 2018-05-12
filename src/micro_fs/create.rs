use super::*;

impl MicroFS {
    pub fn create(&mut self, label: &str, bs: u8, size: usize) {
        let mut file = File::create(self.image.clone()).expect("Failed to create file!");
        self.sb = SuperBlock::new(label, bs, size);
        self.write_super_block(&mut file);
        println!("Super block written to image.");
        for _i in 0..self.sb.fat_size {
            file.write_all(&[0xff;1]).expect("Failed to write in file!");
        }
        self.set_fat();
        println!("FAT written to image.");
        file.seek(SeekFrom::Start(self.root_entry() as u64)).expect("File seek failed !");
        let rest = (size / SECTOR_SIZE) - (self.sb.root_entry as usize * self.sb.block_size as usize);
        for _i in 0..rest {
            file.write_all(&[0;SECTOR_SIZE]).expect("Failed to write in file!");
        }
        self.set_entries();
        println!("Rest of image written.");
        println!("Total bytes = {}", (size / SECTOR_SIZE) * SECTOR_SIZE);
        println!("Total sectors = {}", size / SECTOR_SIZE);
        println!("Total blocks = {}", size / (SECTOR_SIZE * bs as usize));
    }

    fn write_super_block(&mut self, file: &mut File) {
        unsafe {
            file.write_all(&[0;11]).expect("Failed to write in file!");
            file.write_all(&(mem::transmute::<u16, [u8; 2]>(self.sb.sector_size))).expect("Failed to write in file!");
            file.write_all(&(mem::transmute::<u8, [u8; 1]>(self.sb.block_size))).expect("Failed to write in file!");
            file.write_all(&[0;22]).expect("Failed to write in file!");
            file.write_all(&(mem::transmute::<u32, [u8; 4]>(self.sb.fat_size))).expect("Failed to write in file!");
            file.write_all(&[0;2]).expect("Failed to write in file!");
            file.write_all(&(mem::transmute::<u16, [u8; 2]>(self.sb.version))).expect("Failed to write in file!");
            file.write_all(&(mem::transmute::<u32, [u8; 4]>(self.sb.root_entry))).expect("Failed to write in file!");
            file.write_all(&[0;34]).expect("Failed to write in file!");
            file.write_all(&(self.sb.label)).expect("Failed to write in file!");
            file.write_all(&[0;420]).expect("Failed to write in file!");
            file.write_all(&(mem::transmute::<u16, [u8; 2]>(self.sb.signature))).expect("Failed to write in file!");
        }
    }
}