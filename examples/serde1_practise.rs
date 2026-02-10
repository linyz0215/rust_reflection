use argon2::password_hash::SaltString;
use argon2::{Argon2, PasswordHasher};
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use serde_with::serde_as;
use std::{fmt::Display, str::FromStr};
use serde_with::DisplayFromStr;
use base64::Engine;
use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use chacha20poly1305::{ChaCha20Poly1305};
use chacha20poly1305::aead::{Aead, KeyInit, OsRng};  // 添加 KeyInit
use anyhow::{Result};
use test_rust::serdeError;  
const KEYS: &[u8] = b"12345678901234567890123456789012";
#[serde_as]
#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
struct User {
    name: String,
    age: u32,
    fields: Vec<String>,
    data_of_birth: DateTime<Utc>,//chrono serde
    state: WorkState,
    #[serde_as(as = "DisplayFromStr")]
    sensitive_data: SensitiveData,

    #[serde(serialize_with = "b64_encode", deserialize_with = "b64_decode")]
    data: Vec<u8>,
    #[serde(serialize_with = "argon2_register", deserialize_with = "argon2_verify")]
    password: Password,
}


#[derive(Debug)]
struct Password(String);

#[derive(Debug)]
struct SensitiveData(String);

#[derive(Debug, Serialize, Deserialize)]
enum WorkState {
    Working(String),
    OnLeaving(DateTime<Utc>),
    Terminated,
}
pub fn main() -> Result<(), serdeError> {
    let user = User {
        name: "Alice".to_string(),
        age: 30,
        fields: vec!["field1".to_string(), "field2".to_string()],
        data_of_birth: Utc::now(),
        data: vec![1, 2, 3, 4, 5],
        state: WorkState::Working("Developing".to_string()),
        sensitive_data: SensitiveData("lalalla".to_string()),
        password: Password("mypassword".to_string()),
    };
    let string = serde_json::to_string(&user)?;
    println!("Serialized User: {}", &string);
    let user: User = serde_json::from_str(&string)?;
    println!("Deserialized User: {:?}", &user);
    Ok(())
}

fn b64_encode<S>(bytes: &[u8], serializer: S) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    let encoded = URL_SAFE_NO_PAD.encode(bytes);
    serializer.serialize_str(&encoded)
}


fn b64_decode<'de, D>(deserializer: D) -> Result<Vec<u8>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?;
    let decoded = URL_SAFE_NO_PAD
        .decode(s.as_bytes())
        .map_err(serde::de::Error::custom)?;
    Ok(decoded)
}


fn argon2_register<S>(password: &Password, serializer: S) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();
    let password_hash: String = argon2.hash_password(password.0.as_bytes(), &salt).map_err(serde::ser::Error::custom)?.to_string();
    serializer.serialize_str(&password_hash)
}

fn argon2_verify<'de, D>(deserializer: D) -> Result<Password, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?;
    //let parsed_hash = password_hash::PasswordHash::new(&s).map_err(serde::de::Error::custom)?;
    Ok(Password(s))
}







impl Display for SensitiveData {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let key = chacha20poly1305::Key::from_slice(KEYS);
        let cipher = ChaCha20Poly1305::new(key);
        let nonce = chacha20poly1305::Nonce::from_slice(b"unique nonce");

        let ciphertext = cipher.encrypt(nonce, self.0.as_bytes()).expect("encryption failure!");
        println!("nonce: {:?}",nonce.as_slice());
        let ciphertext = nonce.as_slice().iter().chain(ciphertext.iter()).cloned().collect::<Vec<u8>>();
        let encoded = URL_SAFE_NO_PAD.encode(&ciphertext);
        write!(f, "{}", encoded)
    }
}


impl FromStr for SensitiveData {
    type Err = serdeError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let key = chacha20poly1305::Key::from_slice(KEYS);
        let cipher = ChaCha20Poly1305::new(key);
        let decoded = URL_SAFE_NO_PAD
            .decode(s.as_bytes()).map_err(serdeError::from)?;
        println!("Decoded bytes: {:?}", decoded);
        let nonce = decoded[..12].to_vec();
        let nonce = chacha20poly1305::Nonce::from_slice(nonce.as_slice());
        let ciphertext = &decoded[12..];
        let plaintext = cipher.decrypt(nonce, ciphertext)
            .map_err(|e| serdeError::decryptError(format!("Decryption error: {}", e)))?;
        let plaintext_str = String::from_utf8(plaintext)
            .map_err(serdeError::from)?;

        Ok(SensitiveData(plaintext_str))
    }
}