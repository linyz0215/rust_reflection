
use std::{sync::atomic::{AtomicBool, Ordering}, thread::{self, JoinHandle}, time};



static mut DATA: i32 = 0;
static FLAG: AtomicBool = AtomicBool::new(false);


pub fn write_data() -> JoinHandle<()>{
    thread::spawn( move || {
        unsafe {
            DATA += 1;
        }
        FLAG.store(true, Ordering::Release);
    })
}

#[allow(static_mut_refs)]
pub fn read_data() -> JoinHandle<()>{
    thread::spawn( move || {
        thread::sleep(time::Duration::from_secs(5));
        while !FLAG.load(Ordering::Acquire) {}
        unsafe {
                println!("Data: {}", DATA);
        }
    })
}

pub fn main() {
    let _writer = write_data();
    let reader = read_data();

    //writer.join().unwrap();
    reader.join().unwrap();
}


