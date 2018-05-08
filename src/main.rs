#![crate_name = "micro_fs"]

use std::io;
use std::process;
use std::env;

extern crate micro_fs;
use micro_fs::*;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
        println!("Usage : cargo run <image>");
        process::exit(1);
    }
    let image = &args[1];
    let mut fs = MicroFS::new(image);
    
    loop {
        let mut choice = String::new();
        
        println!("\nMENU\n");
        println!("0: quit");
		println!("1: create <label> <block_size> <fs_size>");
		println!("2: add <file>");
		println!("3: del <file>");
		println!("4: list");
		println!("5: info");
        
        io::stdin().read_line(&mut choice).expect("Failed to read line !");
        let choice : u8 = match choice.trim().parse() {
            Ok(num) => num,
            Err(_) => {
                println!("Not a number !");
                continue;
            }
        };
        
        match choice {
            0 => break,
            1 => {
                println!("\n[0] Label :");
                let mut label = String::new();
                io::stdin().read_line(&mut label).expect("Failed to read line !");
                label = label.trim().to_string();
                println!("[1] Block size :");
                let mut str_bs = String::new();
                io::stdin().read_line(&mut str_bs).expect("Failed to read line !");
                let bs = match str_bs.trim().parse() {
                    Ok(num) => num,
                    Err(_) => {
                        println!("Not a number !");
                        continue;
                    }
                };
                println!("[2] FS size :");
                let mut str_size = String::new();
                io::stdin().read_line(&mut str_size).expect("Failed to read line !");
                let size = match str_size.trim().parse() {
                    Ok(num) => num,
                    Err(_) => {
                        println!("Not a number !");
                        continue;
                    }
                };
                println!("");
                fs.create(&label, bs, size);
            },
            2 => {
                println!("\n[0] File :");
                let mut file = String::new();
                io::stdin().read_line(&mut file).expect("Failed to read line !");
                file = file.trim().to_string();
                fs.add(&file);
            },
            3 => println!("del"),
            4 => println!("list"),
            5 => println!("info"),
            _ => println!("Choice {} does not exist", choice),
        }
    }
    process::exit(0);
}
