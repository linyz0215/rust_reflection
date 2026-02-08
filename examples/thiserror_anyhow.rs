use std::fs::{File};

use thiserror::Error;
#[derive(Error, Debug)]
pub enum MyError {
    #[error("error: {0}")]
    ParseIntError(#[from] std::num::ParseIntError),

    #[error("error: {0}")]
    IOError(#[from] std::io::Error),

    #[error("create file error: {0}")]
    CreateFileError(String)
}
pub fn main() -> Result<(), MyError> {
    let _num_str = "123a";
    let file_name = "non_existent_file.txt";
    match File::open(file_name).map_err(MyError::from) {
        Ok(_) => println!("File opened successfully"),
        Err(e) => {
            println!("Failed to open '{}': {}", file_name, e);
            match File::create("hello_world.txt").map_err(|e| MyError::CreateFileError(e.to_string())) {
                Ok(_) => println!("File created successfully"),
                Err(e) => println!("{}", e),
            }
        }
    }




    // match File::open(file_name).map_err(|e| MyError::IOError(e)) {
    //     Ok(_) => println!("File opened successfully"),
    //     Err(e) => println!("Failed to open '{}': {}", file_name, e),
    // }
    // let res = parse_num(num_str).map_err(|e| {
    //     println!("Failed to parse '{}': {}", num_str, e);
    //     e
    // });
    // assert!(matches!(res, Err(MyError::ParseIntError(_))));
    // match parse_num(num_str) {
    //     Ok(num) => println!("Parsed number: {}", num),
    //     Err(e) => println!("Failed to parse '{}': {}", num_str, e),
    // }
    Ok(())
}


pub fn parse_num(num: &str) -> Result<i32, MyError> {
    let num = num.parse::<i32>()?;
    Ok(num)
}
