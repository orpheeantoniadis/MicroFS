#![crate_name = "micro_fs"]

use std::io;
use std::process;

#[macro_use]
extern crate clap;
use clap::App;

extern crate micro_fs;
use micro_fs::*;

fn main() {
    let yaml = load_yaml!("cli.yml");
    let matches = App::from_yaml(yaml).get_matches();
    
    let image = matches.value_of("image").unwrap();
    let mut fs = MicroFS::new(image);
    
    println!("{:?}", matches.subcommand());
    match matches.subcommand() {
        ("create", Some(create_matches)) => {
            let label = create_matches.value_of("label").unwrap();
            let bs = value_t!(create_matches.value_of("block_size"), u8).unwrap_or_else(|e| e.exit());
            let size = value_t!(create_matches.value_of("size"), usize).unwrap_or_else(|e| e.exit());
            fs.create(label, bs, size);
        },
        ("add", Some(add_matches)) => {
            fs.add(add_matches.value_of("file").unwrap());
            fs.save();
        },
        ("del", Some(del_matches)) => {
            fs.del(del_matches.value_of("file").unwrap());
            fs.save();
        },
        ("list", Some(_matches)) => fs.list(),
        ("info", Some(_matches)) => fs.info(),
        ("", None)        => {
            loop {
                let mut choice = String::new();
                
                println!("\nMENU\n");
                println!("0: quit");
        		println!("1: create <label> <block_size> <fs_size>");
        		println!("2: add <file>");
        		println!("3: del <file>");
        		println!("4: list");
        		println!("5: info");
                println!("6: save");
                
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
                        let mut filename = String::new();
                        io::stdin().read_line(&mut filename).expect("Failed to read line !");
                        filename = filename.trim().to_string();
                        println!("");
                        fs.add(&filename);
                    },
                    3 => {
                        println!("\n[0] File :");
                        let mut filename = String::new();
                        io::stdin().read_line(&mut filename).expect("Failed to read line !");
                        filename = filename.trim().to_string();
                        println!("");
                        fs.del(&filename);
                    },
                    4 => fs.list(),
                    5 => fs.info(),
                    6 => fs.save(),
                    _ => println!("Choice {} does not exist", choice),
                }
            }
        },
        _           => unreachable!(),
    }
    process::exit(0);
}
