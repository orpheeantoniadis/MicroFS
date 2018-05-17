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
    test_fs.add("tests/test1.txt");
    
    assert_eq!(bytes_to_str(&test_fs.entries[0].name), "tests/test1.txt");
    assert_eq!(test_fs.entries[0].start, 3);
    assert_eq!(test_fs.entries[0].size, fs::metadata("tests/test1.txt").expect("Failed getting metadata!").len() as u32);
    
    fs::remove_file("fs1_test.img").expect("Failed removing the file");
}

#[test]
fn save() {
    let mut cmp_fs =  MicroFS {
        image: "fs1_test.img".to_string(),
        sb: SuperBlock::new("test", 1, 100000),
        fat: Vec::new(),
        entries:  Vec::new(),
        new_entries: Vec::new()
    };
    
    let mut test_fs = MicroFS::new("fs1_test.img");
    test_fs.create("test", 1, 100000);
    test_fs.add("tests/test1.txt");
    test_fs.save();
    
    cmp_fs.set_fat();
    assert_eq!(test_fs.fat, cmp_fs.fat);
    
    cmp_fs.set_entries();
    assert_eq!(test_fs.entries[0].name, cmp_fs.entries[0].name);
    assert_eq!(test_fs.entries[0].start, cmp_fs.entries[0].start);
    assert_eq!(test_fs.entries[0].size, cmp_fs.entries[0].size);
    
    let mut raw_data = [0; SECTOR_SIZE];
    let mut file = File::open(test_fs.image.clone()).expect("File not found !");
    file.seek(SeekFrom::Start(1536)).expect("File seek failed !");
    file.read(&mut raw_data).expect("Something went wrong reading the file !");
    for i in 0..SECTOR_SIZE {
        assert_eq!(raw_data[i], TEST1[i]);
    }
    
    fs::remove_file("fs1_test.img").expect("Failed removing the file");
}

#[test]
fn remove() {
    let mut cmp_fs =  MicroFS {
        image: "fs1_test.img".to_string(),
        sb: SuperBlock::new("test", 1, 100000),
        fat: Vec::new(),
        entries:  Vec::new(),
        new_entries: Vec::new()
    };
    
    let mut test_fs = MicroFS::new("fs1_test.img");
    test_fs.create("test", 1, 100000);
    test_fs.add("tests/test1.txt");
    test_fs.save();
    test_fs.del("tests/test1.txt");
    test_fs.save();
    
    cmp_fs.set_fat();
    assert_eq!(&cmp_fs.fat[..], &[0xff; 195][..]);
    
    cmp_fs.set_entries();
    assert_eq!(cmp_fs.entries.len(), 0);
    
    fs::remove_file("fs1_test.img").expect("Failed removing the file");
}

#[test]
fn multiple_blocks() {
    let mut test_fs = MicroFS::new("fs1_test.img");
    test_fs.create("test", 1, 100000);
    test_fs.add("tests/test1.txt");
    test_fs.add("tests/test2.txt");
    test_fs.save();
    
    let mut raw_data = [0; SECTOR_SIZE*5];
    let mut file = File::open(test_fs.image.clone()).expect("File not found !");
    file.seek(SeekFrom::Start(2048)).expect("File seek failed !");
    file.read(&mut raw_data).expect("Something went wrong reading the file !");
    
    assert_eq!(&raw_data[..], &TEST2[..]);
    fs::remove_file("fs1_test.img").expect("Failed removing the file");
}

const TEST1 : [u8;SECTOR_SIZE] = [
    0x4c, 0x6f, 0x72, 0x65, 0x6d, 0x20, 0x69, 0x70, 
    0x73, 0x75, 0x6d, 0x20, 0x64, 0x6f, 0x6c, 0x6f, 
    0x72, 0x20, 0x73, 0x69, 0x74, 0x20, 0x61, 0x6d, 
    0x65, 0x74, 0x2c, 0x20, 0x63, 0x6f, 0x6e, 0x73, 
    0x65, 0x63, 0x74, 0x65, 0x74, 0x75, 0x72, 0x20, 
    0x61, 0x64, 0x69, 0x70, 0x69, 0x73, 0x69, 0x63, 
    0x69, 0x6e, 0x67, 0x20, 0x65, 0x6c, 0x69, 0x74, 
    0x2c, 0x20, 0x73, 0x65, 0x64, 0x20, 0x64, 0x6f, 
    0x20, 0x65, 0x69, 0x75, 0x73, 0x6d, 0x6f, 0x64, 
    0x20, 0x74, 0x65, 0x6d, 0x70, 0x6f, 0x72, 0x20, 
    0x69, 0x6e, 0x63, 0x69, 0x64, 0x69, 0x64, 0x75, 
    0x6e, 0x74, 0x20, 0x75, 0x74, 0x20, 0x6c, 0x61, 
    0x62, 0x6f, 0x72, 0x65, 0x20, 0x65, 0x74, 0x20, 
    0x64, 0x6f, 0x6c, 0x6f, 0x72, 0x65, 0x20, 0x6d, 
    0x61, 0x67, 0x6e, 0x61, 0x20, 0x61, 0x6c, 0x69, 
    0x71, 0x75, 0x61, 0x2e, 0x20, 0x55, 0x74, 0x20, 
    0x65, 0x6e, 0x69, 0x6d, 0x20, 0x61, 0x64, 0x20, 
    0x6d, 0x69, 0x6e, 0x69, 0x6d, 0x20, 0x76, 0x65, 
    0x6e, 0x69, 0x61, 0x6d, 0x2c, 0x20, 0x71, 0x75, 
    0x69, 0x73, 0x20, 0x6e, 0x6f, 0x73, 0x74, 0x72, 
    0x75, 0x64, 0x20, 0x65, 0x78, 0x65, 0x72, 0x63, 
    0x69, 0x74, 0x61, 0x74, 0x69, 0x6f, 0x6e, 0x20, 
    0x75, 0x6c, 0x6c, 0x61, 0x6d, 0x63, 0x6f, 0x20, 
    0x6c, 0x61, 0x62, 0x6f, 0x72, 0x69, 0x73, 0x20, 
    0x6e, 0x69, 0x73, 0x69, 0x20, 0x75, 0x74, 0x20, 
    0x61, 0x6c, 0x69, 0x71, 0x75, 0x69, 0x70, 0x20, 
    0x65, 0x78, 0x20, 0x65, 0x61, 0x20, 0x63, 0x6f, 
    0x6d, 0x6d, 0x6f, 0x64, 0x6f, 0x20, 0x63, 0x6f, 
    0x6e, 0x73, 0x65, 0x71, 0x75, 0x61, 0x74, 0x2e, 
    0x20, 0x44, 0x75, 0x69, 0x73, 0x20, 0x61, 0x75, 
    0x74, 0x65, 0x20, 0x69, 0x72, 0x75, 0x72, 0x65, 
    0x20, 0x64, 0x6f, 0x6c, 0x6f, 0x72, 0x20, 0x69, 
    0x6e, 0x20, 0x72, 0x65, 0x70, 0x72, 0x65, 0x68, 
    0x65, 0x6e, 0x64, 0x65, 0x72, 0x69, 0x74, 0x20, 
    0x69, 0x6e, 0x20, 0x76, 0x6f, 0x6c, 0x75, 0x70, 
    0x74, 0x61, 0x74, 0x65, 0x20, 0x76, 0x65, 0x6c, 
    0x69, 0x74, 0x20, 0x65, 0x73, 0x73, 0x65, 0x20, 
    0x63, 0x69, 0x6c, 0x6c, 0x75, 0x6d, 0x20, 0x64, 
    0x6f, 0x6c, 0x6f, 0x72, 0x65, 0x20, 0x65, 0x75, 
    0x20, 0x66, 0x75, 0x67, 0x69, 0x61, 0x74, 0x20, 
    0x6e, 0x75, 0x6c, 0x6c, 0x61, 0x20, 0x70, 0x61, 
    0x72, 0x69, 0x61, 0x74, 0x75, 0x72, 0x2e, 0x20, 
    0x45, 0x78, 0x63, 0x65, 0x70, 0x74, 0x65, 0x75, 
    0x72, 0x20, 0x73, 0x69, 0x6e, 0x74, 0x20, 0x6f, 
    0x63, 0x63, 0x61, 0x65, 0x63, 0x61, 0x74, 0x20, 
    0x63, 0x75, 0x70, 0x69, 0x64, 0x61, 0x74, 0x61, 
    0x74, 0x20, 0x6e, 0x6f, 0x6e, 0x20, 0x70, 0x72, 
    0x6f, 0x69, 0x64, 0x65, 0x6e, 0x74, 0x2c, 0x20, 
    0x73, 0x75, 0x6e, 0x74, 0x20, 0x69, 0x6e, 0x20, 
    0x63, 0x75, 0x6c, 0x70, 0x61, 0x20, 0x71, 0x75, 
    0x69, 0x20, 0x6f, 0x66, 0x66, 0x69, 0x63, 0x69, 
    0x61, 0x20, 0x64, 0x65, 0x73, 0x65, 0x72, 0x75, 
    0x6e, 0x74, 0x20, 0x6d, 0x6f, 0x6c, 0x6c, 0x69, 
    0x74, 0x20, 0x61, 0x6e, 0x69, 0x6d, 0x20, 0x69, 
    0x64, 0x20, 0x65, 0x73, 0x74, 0x20, 0x6c, 0x61, 
    0x62, 0x6f, 0x72, 0x75, 0x6d, 0x2e, 0x0, 0x0, 
    0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 
    0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 
    0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 
    0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 
    0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 
    0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 
    0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 
    0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 
];

const TEST2 : [u8;SECTOR_SIZE*5] = [
    0x4c, 0x6f, 0x72, 0x65, 0x6d, 0x20, 0x69, 0x70, 
    0x73, 0x75, 0x6d, 0x20, 0x64, 0x6f, 0x6c, 0x6f, 
    0x72, 0x20, 0x73, 0x69, 0x74, 0x20, 0x61, 0x6d, 
    0x65, 0x74, 0x2c, 0x20, 0x63, 0x6f, 0x6e, 0x73, 
    0x65, 0x63, 0x74, 0x65, 0x74, 0x75, 0x72, 0x20, 
    0x61, 0x64, 0x69, 0x70, 0x69, 0x73, 0x69, 0x63, 
    0x69, 0x6e, 0x67, 0x20, 0x65, 0x6c, 0x69, 0x74, 
    0x2c, 0x20, 0x73, 0x65, 0x64, 0x20, 0x64, 0x6f, 
    0x20, 0x65, 0x69, 0x75, 0x73, 0x6d, 0x6f, 0x64, 
    0x20, 0x74, 0x65, 0x6d, 0x70, 0x6f, 0x72, 0x20, 
    0x69, 0x6e, 0x63, 0x69, 0x64, 0x69, 0x64, 0x75, 
    0x6e, 0x74, 0x20, 0x75, 0x74, 0x20, 0x6c, 0x61, 
    0x62, 0x6f, 0x72, 0x65, 0x20, 0x65, 0x74, 0x20, 
    0x64, 0x6f, 0x6c, 0x6f, 0x72, 0x65, 0x20, 0x6d, 
    0x61, 0x67, 0x6e, 0x61, 0x20, 0x61, 0x6c, 0x69, 
    0x71, 0x75, 0x61, 0x2e, 0x20, 0x55, 0x74, 0x20, 
    0x65, 0x6e, 0x69, 0x6d, 0x20, 0x61, 0x64, 0x20, 
    0x6d, 0x69, 0x6e, 0x69, 0x6d, 0x20, 0x76, 0x65, 
    0x6e, 0x69, 0x61, 0x6d, 0x2c, 0x20, 0x71, 0x75, 
    0x69, 0x73, 0x20, 0x6e, 0x6f, 0x73, 0x74, 0x72, 
    0x75, 0x64, 0x20, 0x65, 0x78, 0x65, 0x72, 0x63, 
    0x69, 0x74, 0x61, 0x74, 0x69, 0x6f, 0x6e, 0x20, 
    0x75, 0x6c, 0x6c, 0x61, 0x6d, 0x63, 0x6f, 0x20, 
    0x6c, 0x61, 0x62, 0x6f, 0x72, 0x69, 0x73, 0x20, 
    0x6e, 0x69, 0x73, 0x69, 0x20, 0x75, 0x74, 0x20, 
    0x61, 0x6c, 0x69, 0x71, 0x75, 0x69, 0x70, 0x20, 
    0x65, 0x78, 0x20, 0x65, 0x61, 0x20, 0x63, 0x6f, 
    0x6d, 0x6d, 0x6f, 0x64, 0x6f, 0x20, 0x63, 0x6f, 
    0x6e, 0x73, 0x65, 0x71, 0x75, 0x61, 0x74, 0x2e, 
    0x20, 0x44, 0x75, 0x69, 0x73, 0x20, 0x61, 0x75, 
    0x74, 0x65, 0x20, 0x69, 0x72, 0x75, 0x72, 0x65, 
    0x20, 0x64, 0x6f, 0x6c, 0x6f, 0x72, 0x20, 0x69, 
    0x6e, 0x20, 0x72, 0x65, 0x70, 0x72, 0x65, 0x68, 
    0x65, 0x6e, 0x64, 0x65, 0x72, 0x69, 0x74, 0x20, 
    0x69, 0x6e, 0x20, 0x76, 0x6f, 0x6c, 0x75, 0x70, 
    0x74, 0x61, 0x74, 0x65, 0x20, 0x76, 0x65, 0x6c, 
    0x69, 0x74, 0x20, 0x65, 0x73, 0x73, 0x65, 0x20, 
    0x63, 0x69, 0x6c, 0x6c, 0x75, 0x6d, 0x20, 0x64, 
    0x6f, 0x6c, 0x6f, 0x72, 0x65, 0x20, 0x65, 0x75, 
    0x20, 0x66, 0x75, 0x67, 0x69, 0x61, 0x74, 0x20, 
    0x6e, 0x75, 0x6c, 0x6c, 0x61, 0x20, 0x70, 0x61, 
    0x72, 0x69, 0x61, 0x74, 0x75, 0x72, 0x2e, 0x20, 
    0x45, 0x78, 0x63, 0x65, 0x70, 0x74, 0x65, 0x75, 
    0x72, 0x20, 0x73, 0x69, 0x6e, 0x74, 0x20, 0x6f, 
    0x63, 0x63, 0x61, 0x65, 0x63, 0x61, 0x74, 0x20, 
    0x63, 0x75, 0x70, 0x69, 0x64, 0x61, 0x74, 0x61, 
    0x74, 0x20, 0x6e, 0x6f, 0x6e, 0x20, 0x70, 0x72, 
    0x6f, 0x69, 0x64, 0x65, 0x6e, 0x74, 0x2c, 0x20, 
    0x73, 0x75, 0x6e, 0x74, 0x20, 0x69, 0x6e, 0x20, 
    0x63, 0x75, 0x6c, 0x70, 0x61, 0x20, 0x71, 0x75, 
    0x69, 0x20, 0x6f, 0x66, 0x66, 0x69, 0x63, 0x69, 
    0x61, 0x20, 0x64, 0x65, 0x73, 0x65, 0x72, 0x75, 
    0x6e, 0x74, 0x20, 0x6d, 0x6f, 0x6c, 0x6c, 0x69, 
    0x74, 0x20, 0x61, 0x6e, 0x69, 0x6d, 0x20, 0x69, 
    0x64, 0x20, 0x65, 0x73, 0x74, 0x20, 0x6c, 0x61, 
    0x62, 0x6f, 0x72, 0x75, 0x6d, 0x2e, 0xa, 0x4c, 
    0x6f, 0x72, 0x65, 0x6d, 0x20, 0x69, 0x70, 0x73, 
    0x75, 0x6d, 0x20, 0x64, 0x6f, 0x6c, 0x6f, 0x72, 
    0x20, 0x73, 0x69, 0x74, 0x20, 0x61, 0x6d, 0x65, 
    0x74, 0x2c, 0x20, 0x63, 0x6f, 0x6e, 0x73, 0x65, 
    0x63, 0x74, 0x65, 0x74, 0x75, 0x72, 0x20, 0x61, 
    0x64, 0x69, 0x70, 0x69, 0x73, 0x69, 0x63, 0x69, 
    0x6e, 0x67, 0x20, 0x65, 0x6c, 0x69, 0x74, 0x2c, 
    0x20, 0x73, 0x65, 0x64, 0x20, 0x64, 0x6f, 0x20, 
    0x65, 0x69, 0x75, 0x73, 0x6d, 0x6f, 0x64, 0x20, 
    0x74, 0x65, 0x6d, 0x70, 0x6f, 0x72, 0x20, 0x69, 
    0x6e, 0x63, 0x69, 0x64, 0x69, 0x64, 0x75, 0x6e, 
    0x74, 0x20, 0x75, 0x74, 0x20, 0x6c, 0x61, 0x62, 
    0x6f, 0x72, 0x65, 0x20, 0x65, 0x74, 0x20, 0x64, 
    0x6f, 0x6c, 0x6f, 0x72, 0x65, 0x20, 0x6d, 0x61, 
    0x67, 0x6e, 0x61, 0x20, 0x61, 0x6c, 0x69, 0x71, 
    0x75, 0x61, 0x2e, 0x20, 0x55, 0x74, 0x20, 0x65, 
    0x6e, 0x69, 0x6d, 0x20, 0x61, 0x64, 0x20, 0x6d, 
    0x69, 0x6e, 0x69, 0x6d, 0x20, 0x76, 0x65, 0x6e, 
    0x69, 0x61, 0x6d, 0x2c, 0x20, 0x71, 0x75, 0x69, 
    0x73, 0x20, 0x6e, 0x6f, 0x73, 0x74, 0x72, 0x75, 
    0x64, 0x20, 0x65, 0x78, 0x65, 0x72, 0x63, 0x69, 
    0x74, 0x61, 0x74, 0x69, 0x6f, 0x6e, 0x20, 0x75, 
    0x6c, 0x6c, 0x61, 0x6d, 0x63, 0x6f, 0x20, 0x6c, 
    0x61, 0x62, 0x6f, 0x72, 0x69, 0x73, 0x20, 0x6e, 
    0x69, 0x73, 0x69, 0x20, 0x75, 0x74, 0x20, 0x61, 
    0x6c, 0x69, 0x71, 0x75, 0x69, 0x70, 0x20, 0x65, 
    0x78, 0x20, 0x65, 0x61, 0x20, 0x63, 0x6f, 0x6d, 
    0x6d, 0x6f, 0x64, 0x6f, 0x20, 0x63, 0x6f, 0x6e, 
    0x73, 0x65, 0x71, 0x75, 0x61, 0x74, 0x2e, 0x20, 
    0x44, 0x75, 0x69, 0x73, 0x20, 0x61, 0x75, 0x74, 
    0x65, 0x20, 0x69, 0x72, 0x75, 0x72, 0x65, 0x20, 
    0x64, 0x6f, 0x6c, 0x6f, 0x72, 0x20, 0x69, 0x6e, 
    0x20, 0x72, 0x65, 0x70, 0x72, 0x65, 0x68, 0x65, 
    0x6e, 0x64, 0x65, 0x72, 0x69, 0x74, 0x20, 0x69, 
    0x6e, 0x20, 0x76, 0x6f, 0x6c, 0x75, 0x70, 0x74, 
    0x61, 0x74, 0x65, 0x20, 0x76, 0x65, 0x6c, 0x69, 
    0x74, 0x20, 0x65, 0x73, 0x73, 0x65, 0x20, 0x63, 
    0x69, 0x6c, 0x6c, 0x75, 0x6d, 0x20, 0x64, 0x6f, 
    0x6c, 0x6f, 0x72, 0x65, 0x20, 0x65, 0x75, 0x20, 
    0x66, 0x75, 0x67, 0x69, 0x61, 0x74, 0x20, 0x6e, 
    0x75, 0x6c, 0x6c, 0x61, 0x20, 0x70, 0x61, 0x72, 
    0x69, 0x61, 0x74, 0x75, 0x72, 0x2e, 0x20, 0x45, 
    0x78, 0x63, 0x65, 0x70, 0x74, 0x65, 0x75, 0x72, 
    0x20, 0x73, 0x69, 0x6e, 0x74, 0x20, 0x6f, 0x63, 
    0x63, 0x61, 0x65, 0x63, 0x61, 0x74, 0x20, 0x63, 
    0x75, 0x70, 0x69, 0x64, 0x61, 0x74, 0x61, 0x74, 
    0x20, 0x6e, 0x6f, 0x6e, 0x20, 0x70, 0x72, 0x6f, 
    0x69, 0x64, 0x65, 0x6e, 0x74, 0x2c, 0x20, 0x73, 
    0x75, 0x6e, 0x74, 0x20, 0x69, 0x6e, 0x20, 0x63, 
    0x75, 0x6c, 0x70, 0x61, 0x20, 0x71, 0x75, 0x69, 
    0x20, 0x6f, 0x66, 0x66, 0x69, 0x63, 0x69, 0x61, 
    0x20, 0x64, 0x65, 0x73, 0x65, 0x72, 0x75, 0x6e, 
    0x74, 0x20, 0x6d, 0x6f, 0x6c, 0x6c, 0x69, 0x74, 
    0x20, 0x61, 0x6e, 0x69, 0x6d, 0x20, 0x69, 0x64, 
    0x20, 0x65, 0x73, 0x74, 0x20, 0x6c, 0x61, 0x62, 
    0x6f, 0x72, 0x75, 0x6d, 0x2e, 0xa, 0x4c, 0x6f, 
    0x72, 0x65, 0x6d, 0x20, 0x69, 0x70, 0x73, 0x75, 
    0x6d, 0x20, 0x64, 0x6f, 0x6c, 0x6f, 0x72, 0x20, 
    0x73, 0x69, 0x74, 0x20, 0x61, 0x6d, 0x65, 0x74, 
    0x2c, 0x20, 0x63, 0x6f, 0x6e, 0x73, 0x65, 0x63, 
    0x74, 0x65, 0x74, 0x75, 0x72, 0x20, 0x61, 0x64, 
    0x69, 0x70, 0x69, 0x73, 0x69, 0x63, 0x69, 0x6e, 
    0x67, 0x20, 0x65, 0x6c, 0x69, 0x74, 0x2c, 0x20, 
    0x73, 0x65, 0x64, 0x20, 0x64, 0x6f, 0x20, 0x65, 
    0x69, 0x75, 0x73, 0x6d, 0x6f, 0x64, 0x20, 0x74, 
    0x65, 0x6d, 0x70, 0x6f, 0x72, 0x20, 0x69, 0x6e, 
    0x63, 0x69, 0x64, 0x69, 0x64, 0x75, 0x6e, 0x74, 
    0x20, 0x75, 0x74, 0x20, 0x6c, 0x61, 0x62, 0x6f, 
    0x72, 0x65, 0x20, 0x65, 0x74, 0x20, 0x64, 0x6f, 
    0x6c, 0x6f, 0x72, 0x65, 0x20, 0x6d, 0x61, 0x67, 
    0x6e, 0x61, 0x20, 0x61, 0x6c, 0x69, 0x71, 0x75, 
    0x61, 0x2e, 0x20, 0x55, 0x74, 0x20, 0x65, 0x6e, 
    0x69, 0x6d, 0x20, 0x61, 0x64, 0x20, 0x6d, 0x69, 
    0x6e, 0x69, 0x6d, 0x20, 0x76, 0x65, 0x6e, 0x69, 
    0x61, 0x6d, 0x2c, 0x20, 0x71, 0x75, 0x69, 0x73, 
    0x20, 0x6e, 0x6f, 0x73, 0x74, 0x72, 0x75, 0x64, 
    0x20, 0x65, 0x78, 0x65, 0x72, 0x63, 0x69, 0x74, 
    0x61, 0x74, 0x69, 0x6f, 0x6e, 0x20, 0x75, 0x6c, 
    0x6c, 0x61, 0x6d, 0x63, 0x6f, 0x20, 0x6c, 0x61, 
    0x62, 0x6f, 0x72, 0x69, 0x73, 0x20, 0x6e, 0x69, 
    0x73, 0x69, 0x20, 0x75, 0x74, 0x20, 0x61, 0x6c, 
    0x69, 0x71, 0x75, 0x69, 0x70, 0x20, 0x65, 0x78, 
    0x20, 0x65, 0x61, 0x20, 0x63, 0x6f, 0x6d, 0x6d, 
    0x6f, 0x64, 0x6f, 0x20, 0x63, 0x6f, 0x6e, 0x73, 
    0x65, 0x71, 0x75, 0x61, 0x74, 0x2e, 0x20, 0x44, 
    0x75, 0x69, 0x73, 0x20, 0x61, 0x75, 0x74, 0x65, 
    0x20, 0x69, 0x72, 0x75, 0x72, 0x65, 0x20, 0x64, 
    0x6f, 0x6c, 0x6f, 0x72, 0x20, 0x69, 0x6e, 0x20, 
    0x72, 0x65, 0x70, 0x72, 0x65, 0x68, 0x65, 0x6e, 
    0x64, 0x65, 0x72, 0x69, 0x74, 0x20, 0x69, 0x6e, 
    0x20, 0x76, 0x6f, 0x6c, 0x75, 0x70, 0x74, 0x61, 
    0x74, 0x65, 0x20, 0x76, 0x65, 0x6c, 0x69, 0x74, 
    0x20, 0x65, 0x73, 0x73, 0x65, 0x20, 0x63, 0x69, 
    0x6c, 0x6c, 0x75, 0x6d, 0x20, 0x64, 0x6f, 0x6c, 
    0x6f, 0x72, 0x65, 0x20, 0x65, 0x75, 0x20, 0x66, 
    0x75, 0x67, 0x69, 0x61, 0x74, 0x20, 0x6e, 0x75, 
    0x6c, 0x6c, 0x61, 0x20, 0x70, 0x61, 0x72, 0x69, 
    0x61, 0x74, 0x75, 0x72, 0x2e, 0x20, 0x45, 0x78, 
    0x63, 0x65, 0x70, 0x74, 0x65, 0x75, 0x72, 0x20, 
    0x73, 0x69, 0x6e, 0x74, 0x20, 0x6f, 0x63, 0x63, 
    0x61, 0x65, 0x63, 0x61, 0x74, 0x20, 0x63, 0x75, 
    0x70, 0x69, 0x64, 0x61, 0x74, 0x61, 0x74, 0x20, 
    0x6e, 0x6f, 0x6e, 0x20, 0x70, 0x72, 0x6f, 0x69, 
    0x64, 0x65, 0x6e, 0x74, 0x2c, 0x20, 0x73, 0x75, 
    0x6e, 0x74, 0x20, 0x69, 0x6e, 0x20, 0x63, 0x75, 
    0x6c, 0x70, 0x61, 0x20, 0x71, 0x75, 0x69, 0x20, 
    0x6f, 0x66, 0x66, 0x69, 0x63, 0x69, 0x61, 0x20, 
    0x64, 0x65, 0x73, 0x65, 0x72, 0x75, 0x6e, 0x74, 
    0x20, 0x6d, 0x6f, 0x6c, 0x6c, 0x69, 0x74, 0x20, 
    0x61, 0x6e, 0x69, 0x6d, 0x20, 0x69, 0x64, 0x20, 
    0x65, 0x73, 0x74, 0x20, 0x6c, 0x61, 0x62, 0x6f, 
    0x72, 0x75, 0x6d, 0x2e, 0xa, 0x4c, 0x6f, 0x72, 
    0x65, 0x6d, 0x20, 0x69, 0x70, 0x73, 0x75, 0x6d, 
    0x20, 0x64, 0x6f, 0x6c, 0x6f, 0x72, 0x20, 0x73, 
    0x69, 0x74, 0x20, 0x61, 0x6d, 0x65, 0x74, 0x2c, 
    0x20, 0x63, 0x6f, 0x6e, 0x73, 0x65, 0x63, 0x74, 
    0x65, 0x74, 0x75, 0x72, 0x20, 0x61, 0x64, 0x69, 
    0x70, 0x69, 0x73, 0x69, 0x63, 0x69, 0x6e, 0x67, 
    0x20, 0x65, 0x6c, 0x69, 0x74, 0x2c, 0x20, 0x73, 
    0x65, 0x64, 0x20, 0x64, 0x6f, 0x20, 0x65, 0x69, 
    0x75, 0x73, 0x6d, 0x6f, 0x64, 0x20, 0x74, 0x65, 
    0x6d, 0x70, 0x6f, 0x72, 0x20, 0x69, 0x6e, 0x63, 
    0x69, 0x64, 0x69, 0x64, 0x75, 0x6e, 0x74, 0x20, 
    0x75, 0x74, 0x20, 0x6c, 0x61, 0x62, 0x6f, 0x72, 
    0x65, 0x20, 0x65, 0x74, 0x20, 0x64, 0x6f, 0x6c, 
    0x6f, 0x72, 0x65, 0x20, 0x6d, 0x61, 0x67, 0x6e, 
    0x61, 0x20, 0x61, 0x6c, 0x69, 0x71, 0x75, 0x61, 
    0x2e, 0x20, 0x55, 0x74, 0x20, 0x65, 0x6e, 0x69, 
    0x6d, 0x20, 0x61, 0x64, 0x20, 0x6d, 0x69, 0x6e, 
    0x69, 0x6d, 0x20, 0x76, 0x65, 0x6e, 0x69, 0x61, 
    0x6d, 0x2c, 0x20, 0x71, 0x75, 0x69, 0x73, 0x20, 
    0x6e, 0x6f, 0x73, 0x74, 0x72, 0x75, 0x64, 0x20, 
    0x65, 0x78, 0x65, 0x72, 0x63, 0x69, 0x74, 0x61, 
    0x74, 0x69, 0x6f, 0x6e, 0x20, 0x75, 0x6c, 0x6c, 
    0x61, 0x6d, 0x63, 0x6f, 0x20, 0x6c, 0x61, 0x62, 
    0x6f, 0x72, 0x69, 0x73, 0x20, 0x6e, 0x69, 0x73, 
    0x69, 0x20, 0x75, 0x74, 0x20, 0x61, 0x6c, 0x69, 
    0x71, 0x75, 0x69, 0x70, 0x20, 0x65, 0x78, 0x20, 
    0x65, 0x61, 0x20, 0x63, 0x6f, 0x6d, 0x6d, 0x6f, 
    0x64, 0x6f, 0x20, 0x63, 0x6f, 0x6e, 0x73, 0x65, 
    0x71, 0x75, 0x61, 0x74, 0x2e, 0x20, 0x44, 0x75, 
    0x69, 0x73, 0x20, 0x61, 0x75, 0x74, 0x65, 0x20, 
    0x69, 0x72, 0x75, 0x72, 0x65, 0x20, 0x64, 0x6f, 
    0x6c, 0x6f, 0x72, 0x20, 0x69, 0x6e, 0x20, 0x72, 
    0x65, 0x70, 0x72, 0x65, 0x68, 0x65, 0x6e, 0x64, 
    0x65, 0x72, 0x69, 0x74, 0x20, 0x69, 0x6e, 0x20, 
    0x76, 0x6f, 0x6c, 0x75, 0x70, 0x74, 0x61, 0x74, 
    0x65, 0x20, 0x76, 0x65, 0x6c, 0x69, 0x74, 0x20, 
    0x65, 0x73, 0x73, 0x65, 0x20, 0x63, 0x69, 0x6c, 
    0x6c, 0x75, 0x6d, 0x20, 0x64, 0x6f, 0x6c, 0x6f, 
    0x72, 0x65, 0x20, 0x65, 0x75, 0x20, 0x66, 0x75, 
    0x67, 0x69, 0x61, 0x74, 0x20, 0x6e, 0x75, 0x6c, 
    0x6c, 0x61, 0x20, 0x70, 0x61, 0x72, 0x69, 0x61, 
    0x74, 0x75, 0x72, 0x2e, 0x20, 0x45, 0x78, 0x63, 
    0x65, 0x70, 0x74, 0x65, 0x75, 0x72, 0x20, 0x73, 
    0x69, 0x6e, 0x74, 0x20, 0x6f, 0x63, 0x63, 0x61, 
    0x65, 0x63, 0x61, 0x74, 0x20, 0x63, 0x75, 0x70, 
    0x69, 0x64, 0x61, 0x74, 0x61, 0x74, 0x20, 0x6e, 
    0x6f, 0x6e, 0x20, 0x70, 0x72, 0x6f, 0x69, 0x64, 
    0x65, 0x6e, 0x74, 0x2c, 0x20, 0x73, 0x75, 0x6e, 
    0x74, 0x20, 0x69, 0x6e, 0x20, 0x63, 0x75, 0x6c, 
    0x70, 0x61, 0x20, 0x71, 0x75, 0x69, 0x20, 0x6f, 
    0x66, 0x66, 0x69, 0x63, 0x69, 0x61, 0x20, 0x64, 
    0x65, 0x73, 0x65, 0x72, 0x75, 0x6e, 0x74, 0x20, 
    0x6d, 0x6f, 0x6c, 0x6c, 0x69, 0x74, 0x20, 0x61, 
    0x6e, 0x69, 0x6d, 0x20, 0x69, 0x64, 0x20, 0x65, 
    0x73, 0x74, 0x20, 0x6c, 0x61, 0x62, 0x6f, 0x72, 
    0x75, 0x6d, 0x2e, 0xa, 0x4c, 0x6f, 0x72, 0x65, 
    0x6d, 0x20, 0x69, 0x70, 0x73, 0x75, 0x6d, 0x20, 
    0x64, 0x6f, 0x6c, 0x6f, 0x72, 0x20, 0x73, 0x69, 
    0x74, 0x20, 0x61, 0x6d, 0x65, 0x74, 0x2c, 0x20, 
    0x63, 0x6f, 0x6e, 0x73, 0x65, 0x63, 0x74, 0x65, 
    0x74, 0x75, 0x72, 0x20, 0x61, 0x64, 0x69, 0x70, 
    0x69, 0x73, 0x69, 0x63, 0x69, 0x6e, 0x67, 0x20, 
    0x65, 0x6c, 0x69, 0x74, 0x2c, 0x20, 0x73, 0x65, 
    0x64, 0x20, 0x64, 0x6f, 0x20, 0x65, 0x69, 0x75, 
    0x73, 0x6d, 0x6f, 0x64, 0x20, 0x74, 0x65, 0x6d, 
    0x70, 0x6f, 0x72, 0x20, 0x69, 0x6e, 0x63, 0x69, 
    0x64, 0x69, 0x64, 0x75, 0x6e, 0x74, 0x20, 0x75, 
    0x74, 0x20, 0x6c, 0x61, 0x62, 0x6f, 0x72, 0x65, 
    0x20, 0x65, 0x74, 0x20, 0x64, 0x6f, 0x6c, 0x6f, 
    0x72, 0x65, 0x20, 0x6d, 0x61, 0x67, 0x6e, 0x61, 
    0x20, 0x61, 0x6c, 0x69, 0x71, 0x75, 0x61, 0x2e, 
    0x20, 0x55, 0x74, 0x20, 0x65, 0x6e, 0x69, 0x6d, 
    0x20, 0x61, 0x64, 0x20, 0x6d, 0x69, 0x6e, 0x69, 
    0x6d, 0x20, 0x76, 0x65, 0x6e, 0x69, 0x61, 0x6d, 
    0x2c, 0x20, 0x71, 0x75, 0x69, 0x73, 0x20, 0x6e, 
    0x6f, 0x73, 0x74, 0x72, 0x75, 0x64, 0x20, 0x65, 
    0x78, 0x65, 0x72, 0x63, 0x69, 0x74, 0x61, 0x74, 
    0x69, 0x6f, 0x6e, 0x20, 0x75, 0x6c, 0x6c, 0x61, 
    0x6d, 0x63, 0x6f, 0x20, 0x6c, 0x61, 0x62, 0x6f, 
    0x72, 0x69, 0x73, 0x20, 0x6e, 0x69, 0x73, 0x69, 
    0x20, 0x75, 0x74, 0x20, 0x61, 0x6c, 0x69, 0x71, 
    0x75, 0x69, 0x70, 0x20, 0x65, 0x78, 0x20, 0x65, 
    0x61, 0x20, 0x63, 0x6f, 0x6d, 0x6d, 0x6f, 0x64, 
    0x6f, 0x20, 0x63, 0x6f, 0x6e, 0x73, 0x65, 0x71, 
    0x75, 0x61, 0x74, 0x2e, 0x20, 0x44, 0x75, 0x69, 
    0x73, 0x20, 0x61, 0x75, 0x74, 0x65, 0x20, 0x69, 
    0x72, 0x75, 0x72, 0x65, 0x20, 0x64, 0x6f, 0x6c, 
    0x6f, 0x72, 0x20, 0x69, 0x6e, 0x20, 0x72, 0x65, 
    0x70, 0x72, 0x65, 0x68, 0x65, 0x6e, 0x64, 0x65, 
    0x72, 0x69, 0x74, 0x20, 0x69, 0x6e, 0x20, 0x76, 
    0x6f, 0x6c, 0x75, 0x70, 0x74, 0x61, 0x74, 0x65, 
    0x20, 0x76, 0x65, 0x6c, 0x69, 0x74, 0x20, 0x65, 
    0x73, 0x73, 0x65, 0x20, 0x63, 0x69, 0x6c, 0x6c, 
    0x75, 0x6d, 0x20, 0x64, 0x6f, 0x6c, 0x6f, 0x72, 
    0x65, 0x20, 0x65, 0x75, 0x20, 0x66, 0x75, 0x67, 
    0x69, 0x61, 0x74, 0x20, 0x6e, 0x75, 0x6c, 0x6c, 
    0x61, 0x20, 0x70, 0x61, 0x72, 0x69, 0x61, 0x74, 
    0x75, 0x72, 0x2e, 0x20, 0x45, 0x78, 0x63, 0x65, 
    0x70, 0x74, 0x65, 0x75, 0x72, 0x20, 0x73, 0x69, 
    0x6e, 0x74, 0x20, 0x6f, 0x63, 0x63, 0x61, 0x65, 
    0x63, 0x61, 0x74, 0x20, 0x63, 0x75, 0x70, 0x69, 
    0x64, 0x61, 0x74, 0x61, 0x74, 0x20, 0x6e, 0x6f, 
    0x6e, 0x20, 0x70, 0x72, 0x6f, 0x69, 0x64, 0x65, 
    0x6e, 0x74, 0x2c, 0x20, 0x73, 0x75, 0x6e, 0x74, 
    0x20, 0x69, 0x6e, 0x20, 0x63, 0x75, 0x6c, 0x70, 
    0x61, 0x20, 0x71, 0x75, 0x69, 0x20, 0x6f, 0x66, 
    0x66, 0x69, 0x63, 0x69, 0x61, 0x20, 0x64, 0x65, 
    0x73, 0x65, 0x72, 0x75, 0x6e, 0x74, 0x20, 0x6d, 
    0x6f, 0x6c, 0x6c, 0x69, 0x74, 0x20, 0x61, 0x6e, 
    0x69, 0x6d, 0x20, 0x69, 0x64, 0x20, 0x65, 0x73, 
    0x74, 0x20, 0x6c, 0x61, 0x62, 0x6f, 0x72, 0x75, 
    0x6d, 0x2e, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 
    0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 
    0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 
    0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 
    0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 
    0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 
    0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 
    0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 
    0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 
    0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 
    0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 
    0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 
    0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 
    0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 
    0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 
    0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 
    0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 
    0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 
    0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 
    0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 
    0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 
    0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 
    0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 
    0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 
    0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 
    0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 
    0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 
    0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 
    0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 
    0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 
    0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 
    0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 
    0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 
    0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 
    0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 
    0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 
    0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 
    0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 
    0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 
    0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 
    0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 
];