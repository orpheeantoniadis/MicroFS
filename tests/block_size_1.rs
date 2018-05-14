extern crate micro_fs;
use micro_fs::*;
use micro_fs::utils::bytes_to_str;

use std::mem;
use std::fs;
use std::fs::File;
use std::io::prelude::*;
use std::io::SeekFrom;

#[test]
fn constructors() {
    let mut test_fs = MicroFS::new("fs1_test.img");
    test_fs.sb = SuperBlock::new("test", 1, 100000);
    assert_eq!(test_fs.image, "fs1_test.img");
    assert_eq!(test_fs.sb.sector_size, SECTOR_SIZE as u16);
    assert_eq!(test_fs.sb.block_size, 1);
    assert_eq!(test_fs.sb.fat_size, 195);
    assert_eq!(test_fs.sb.version, 1);
    assert_eq!(test_fs.sb.root_entry, 2);
    assert_eq!(test_fs.sb.label, [b't', b'e', b's', b't', b'\0', b'\0', b'\0', b'\0']);
    assert_eq!(test_fs.sb.signature, MAGIC);
}

#[test]
fn create() {
    let mut test_fs = MicroFS::new("fs1_test.img");
    test_fs.create("test", 1, 100000);
    let mut raw_sb = [0; SECTOR_SIZE];
    let mut file = File::open(test_fs.image.clone()).expect("File not found !");
    file.read(&mut raw_sb).expect("Something went wrong reading the file !");
    
    let signature = unsafe { mem::transmute::<[u8;2], u16>([raw_sb[510], raw_sb[511]]) };
    assert_eq!(signature, MAGIC);
    let bs = raw_sb[13];
    assert_eq!(bs, 1);
    let raw_label = &raw_sb[82..90];
    let label = bytes_to_str(raw_label);
    assert_eq!(label, "test");
    let size = fs::metadata(test_fs.image.clone()).expect("Failed getting metadata!").len() as usize;
    assert_eq!(size, 99840);
    
    let mut raw_fat = [0; 195];
    file.read(&mut raw_fat).expect("Something went wrong reading the file !");
    assert_eq!(&raw_fat[..], &[0xff; 195][..]);
    
    file.seek(SeekFrom::Start(1024)).expect("File seek failed !");
    let mut raw_entries = [0; 512];
    file.read(&mut raw_entries).expect("Something went wrong reading the file !");
    assert_eq!(&raw_entries[..], &[0; 512][..]);
    
    fs::remove_file("fs1_test.img").expect("Failed removing the file");
}

#[test]
fn add() {
    let mut test_fs = MicroFS::new("fs1_test.img");
    test_fs.create("test", 1, 100000);
    test_fs.add("README.md");
    
    assert_eq!(bytes_to_str(&test_fs.entries[0].name), "README.md");
    assert_eq!(test_fs.entries[0].start, 3);
    assert_eq!(test_fs.entries[0].size, fs::metadata("README.md").expect("Failed getting metadata!").len() as u32);
    
    fs::remove_file("fs1_test.img").expect("Failed removing the file");
}

#[test]
fn save() {
    let mut cmp_fs =  MicroFS {
        image: "fs1_test.img".to_string(),
        sb: SuperBlock::new("test", 1, 100000),
        fat: Vec::new(),
        entries:  Vec::new()
    };
    
    let mut test_fs = MicroFS::new("fs1_test.img");
    test_fs.create("test", 1, 100000);
    test_fs.add("README.md");
    test_fs.save();
    
    println!("{:?}", cmp_fs.fat);
    cmp_fs.set_fat();
    assert_eq!(test_fs.fat, cmp_fs.fat);
    
    cmp_fs.set_entries();
    assert_eq!(test_fs.entries[0].name, cmp_fs.entries[0].name);
    assert_eq!(test_fs.entries[0].start, cmp_fs.entries[0].start);
    assert_eq!(test_fs.entries[0].size, cmp_fs.entries[0].size);
    
    fs::remove_file("fs1_test.img").expect("Failed removing the file");
}