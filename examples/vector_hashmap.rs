use std::collections::HashMap;


pub struct MyKey {
    name: String,
    value: i32,
}

impl MyKey {
    pub fn new(name: String, value: i32) -> Self {
        MyKey {
            name,
            value,
        }
    }
}




pub fn main() {
    let mut v = vec![1,2,3];   
    let first = &v[0];
    println!("{}", first);
    v.push(6);
    println!("{:?}", v);
    let _second = match v.get(1) {
        Some(x) => x,
        None => &0,
    };
    for i in &v {
        println!("{}", i);
    }
    let mut v2: Vec<i32> = Vec::with_capacity(10);
    v2.extend([1,2,3]);
    v.append(&mut v2);
    println!("{:?}", v);
    println!("{:?}", v2);
    assert_eq!(&v[0..3], &[1,2,3]);
    let teams_list = vec![
        ("中国队".to_string(), 100),
        ("美国队".to_string(), 10),
        ("日本队".to_string(), 50),
    ];

    let mut my_hash_map = teams_list.into_iter().collect::<HashMap<String, i32>>();
    println!("{:?}", my_hash_map);
    let my_string = "hello world".to_string();
    let old = my_hash_map.insert("world".to_string(),1);
    let str = my_string.clone().split_off(6);
    println!("{}", str);
    let _value: Option<&i32> =  my_hash_map.get(&my_string);

    let _value2 = my_hash_map.entry("hello".to_string()).or_insert(5); 
    println!("{:?}",old);
    println!("{:?}",my_hash_map);

    let person_vec = vec![
        MyKey::new("Alice".to_string(), 30),
        MyKey::new("Bob".to_string(), 25),
        MyKey::new("Charlie".to_string(), 35),
    ];
    let person_map = person_vec.into_iter().map(|person| (person.name, person.value)).collect::<HashMap<String, i32>>();
    println!("{:?}", person_map);
    let string = "hello world hello rust hello uu".to_string();
    let mut word_count = HashMap::new();
    for word in string.split_whitespace() {
        let count = word_count.entry(word).or_insert(0);
        *count += 1;
    }    
    println!("{:?}", word_count);

    let vec1 = vec!["hello".to_string(), "world".to_string(), "halo".to_string(), "rust".to_string()];
    let vec2 = vec![1, 2, 3, 4];
    let hashmap = vec1.iter().zip(vec2.into_iter()).collect::<HashMap<_,_>>();
    for (i,value) in vec1.iter().enumerate() {
        println!("{}: {}", i, value); 
    }
    let hashmap1 = hashmap.iter().map(|(k,v)| {let mut str = (*k).clone(); str.insert_str(0, "key: "); (str, *v+1)}).collect::<HashMap<_,_>>();
    println!("{:?}", hashmap);
    println!("{:?}", hashmap1);
    for (key, value) in hashmap1.iter() {
        println!("{}: {}", key, value); 
    }
    let _hashmap2 = hashmap1.iter().filter(|(_,v)| **v>3).collect::<HashMap<_,_>>();

}