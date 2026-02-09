use std::{borrow::Cow, str::from_utf8};
fn main() {
    let mut s = String::from("hello world");

    let word = first_word(s.clone());
    s.clear(); // error!
    println!("{}",word);
    
    println!("{}",s);
    let s = '😊';
    println!("char 大小: {} 字节", std::mem::size_of::<char>());  // 4
    println!("UTF-8 字节数: {}", s.len_utf8());  // 4

    let string2 = String::from("😊");
    let _str2 = "😊".as_bytes();
    let str3 = from_utf8(&[240, 159, 152, 138]).unwrap();
    println!("{:?}",str3);
    let bytes = string2.as_bytes();
    println!("{:?}",bytes);  // [240, 159, 152, 138]
    //println!("{}",string2.len());  // 4

    let mut string3 = String::from("hello world");
    //字符串各种操作
    let s1 = string3.replace("hello", "goodbye");
    println!("{}",s1); // goodbye world
    let s2 = string3.replacen("h", "H", 1);
    println!("{}", s2);
    let s3 = string3.to_uppercase();
    println!("{}", s3);
    let s4 = string3.split_off(6);//从索引6处分割字符串,返回后半部分，原字符串保留前半部分
    println!("s4: {}, string3: {}", s4, string3);
    let s5 = format!("{} {}", string3, s4); // 拼接字符串
    println!("{}", s5);
    let hello_string = b"hello";
    println!("{:?}", hello_string);
    let _hello_str2 = match String::from_utf8_lossy(hello_string) {
        Cow::Borrowed(s) => s,
        Cow::Owned(_) => "Invalid UTF-8",
    };

    let hello_str = std::str::from_utf8(hello_string).unwrap();
    println!("{}", hello_str);
    let _str6: String = "hello world".into();
    let str7 = &"hello world".to_string();
    let str8 = str7.clone();
}


fn first_word(s: String) -> String {
    s[..1].to_string()
}
