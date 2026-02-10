use std::collections::HashMap;
use std::hash::Hash;
use std::vec;
use std::{borrow::Cow, str::from_utf8};

use bytes::BytesMut;
use bytes::BufMut;
use bytes::Bytes;

struct MyStruct {
    field1: String,
    field2: i32,
}
fn main() {
    // let mut s = String::from("hello world");

    // let word = first_word(s.clone());
    // s.clear(); // error!
    // println!("{}",word);
    
    // println!("{}",s);
    // let s = '😊';
    // println!("char 大小: {} 字节", std::mem::size_of::<char>());  // 4
    // println!("UTF-8 字节数: {}", s.len_utf8());  // 4

    // let string2 = String::from("😊");
    // let _str2 = "😊".as_bytes();
    // let str3 = from_utf8(&[240, 159, 152, 138]).unwrap();
    // println!("{:?}",str3);
    // let bytes = string2.as_bytes();
    // println!("{:?}",bytes);  // [240, 159, 152, 138]
    // //println!("{}",string2.len());  // 4

    // let mut string3 = String::from("hello world");
    // //字符串各种操作
    // let s1 = string3.replace("hello", "goodbye");
    // println!("{}",s1); // goodbye world
    // let s2 = string3.replacen("h", "H", 1);
    // println!("{}", s2);
    // let s3 = string3.to_uppercase();
    // println!("{}", s3);
    // let s4 = string3.split_off(6);//从索引6处分割字符串,返回后半部分，原字符串保留前半部分
    // println!("s4: {}, string3: {}", s4, string3);
    // let s5 = format!("{} {}", string3, s4); // 拼接字符串
    // println!("{}", s5);
    // let hello_string = b"hello";
    // println!("{:?}", hello_string);
    // let _hello_str2 = match String::from_utf8_lossy(hello_string) {
    //     Cow::Borrowed(s) => s,
    //     Cow::Owned(_) => "Invalid UTF-8",
    // };

    // let hello_str = std::str::from_utf8(hello_string).unwrap();
    // println!("{}", hello_str);
    // let _str6: String = "hello world".into();
    // let str7 = &"hello world".to_string();
    // let str8 = str7.clone();

    // let mut buf = BytesMut::with_capacity(1024);
    // buf.extend_from_slice(b"hello world\n");
    // buf.put(&b"goodbye world"[..]);
    // buf.put_i64(0xdeadbeef);
    // println!("{:?}", buf);
    // let a = buf.split();
    // println!("{:?}", a);
    // println!("{:?}", buf);
    // let mut b = a.freeze();
    // println!("{:?}", b);
    // let c = b.split_to(12);
    // println!("{:?}", c);
    // println!("{:?}", b);

    // let mut vecu8 = Vec::<u8>::with_capacity(1024);
    // vecu8.extend_from_slice(b"hello world\n");
    // let bytesa = Bytes::from(vecu8);
    // let bytesb = bytesa.clone();

    // println!("{:?}", bytesa);
    let mut string1 = format!("hello world");
    let str1 = string1.as_str();

    println!("{}", std::mem::size_of::<MyStruct>()); 
    println!("{}", std::mem::size_of::<Box<MyStruct>>());
    let my_struct = MyStruct {
        field1: "hello".to_string(),
        field2: 42,
    };
    let boxed_struct = Box::new(my_struct);
    println!("{}", std::mem::size_of_val(&boxed_struct));
    string1.push_str("!!!!!!");
    let string2 = string1.split_off(11);
    let mut string2 = string1.replacen("o", "O", 2);
    string2.push('!');
    if let Some(ch) = string2.pop() {
        println!("{} {}", string2, ch);
    }
    let contains_el = string2.contains("el");
    let string3 = vec!["uu", "alice", "bob", "tom"];
    let hashmap1 = string3.into_iter().map(|s| (s.to_string(),0)).collect::<HashMap<String,i32>>();
    let mut hashmap2 = hashmap1.iter().map(|(k,v)| (k.clone().to_uppercase(), v+1)).collect::<HashMap<String,i32>>();
    hashmap2.entry("linyz".to_string()).or_insert(1);
    hashmap2.insert("laalal".to_string(), 1);
    
}



fn first_word(s: String) -> String {
    s[..1].to_string()
}

pub fn foo(str: impl AsRef<str>) -> String {
    let s = str.as_ref();
    s.to_string()
}

pub fn foo2(str: impl Into<String>) -> String {
    let s: String = str.into();
    s
}