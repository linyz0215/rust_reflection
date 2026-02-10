use anyhow::Result;
use serde::{Serialize, Deserialize};

// serde derive
#[derive(Serialize, Deserialize)]
pub struct User {
    name: String,
    age: u32,
    fields: Vec<String>,
}


pub fn main() -> Result<(), anyhow::Error> {
    let user = User {
        name: "Alice".to_string(),
        age: 30,
        fields: vec!["field1".to_string(), "field2".to_string()],
    };
    let string = serde_json::to_string(&user)?;
    println!("Serialized User: {}", &string);
    let _user: User = serde_json::from_str(&string)?;
    Ok(())
}