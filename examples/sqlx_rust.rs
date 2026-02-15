use core::hash;
use std::{env, fs::File, path::PathBuf};

use anyhow::Result;
use chrono::{DateTime, Utc};
use serde::Deserialize;
use sha1::{Digest, digest::generic_array::sequence::Lengthen};
use sqlx::FromRow;
use thiserror::Error;
use tokio::fs;
use sqlx::Executor;
#[derive(Deserialize, Debug)]
struct Config {
    database: DatabaseConfig,
}

#[derive(Deserialize, Debug)]
struct DatabaseConfig {
    url: String,
}

#[derive(Error, Debug)]
pub enum MyError {}
/*
CREATE TYPE user_type AS ENUM(
  'regular',
  'admin'
);

CREATE TABLE IF NOT EXISTS users(
  id bigserial PRIMARY KEY,
  name varchar(64) NOT NULL,
  email varchar(64) NOT NULL,
  password varchar(64) NOT NULL,
  type user_type NOT NULL
);
*/
#[derive(sqlx::Type, Debug)]
#[sqlx(type_name = "user_type", rename_all = "snake_case")]
enum UserType {
    regular,
    admin,
}

/*
CREATE TYPE chat_type AS ENUM(
  'single',
  'group',
  'private_channel',
  'public_channel'
);
*/
#[derive(sqlx::Type, Debug)]
#[sqlx(type_name = "chat_type", rename_all = "snake_case")]
enum ChatType {
    single,
    group,
    private_channel,
    public_channel,
}
/*

CREATE TABLE IF NOT EXISTS chats(
  chat_id bigserial PRIMARY KEY,
  chat_name varchar(64) NOT NULL,
  type chat_type NOT NULL
);
*/
#[derive(FromRow, Debug)]
struct Chat {
    chat_id: i64,
    chat_name: String,
    r#type: ChatType,
    members: Vec<i64>,
}

#[derive(FromRow, Debug)]
struct users {
    id: i64,
    name: String,
    email: String,
    password: String,
    r#type: UserType,
}
/*
CREATE TABLE IF NOT EXISTS messages (
  message_id bigserial PRIMARY KEY,
  chat_id bigint NOT NULL REFERENCES chats(chat_id),
  sender_id bigint NOT NULL REFERENCES users(id),
  content text NOT NULL,
  file text[] DEFAULT '{}',
  created_at timestamptz DEFAULT CURRENT_TIMESTAMP
);
*/
#[derive(FromRow, Debug)]

struct messages {
    message_id: i64,
    chat_id: i64,
    sender_id: i64,
    content: String,
    file: Vec<String>,
    created_at: DateTime<Utc>,
}
#[tokio::main]
async fn main() -> Result<()> {
    let config = Config::load()?;
    let pool = sqlx::PgPool::connect(&config.database.url).await?;
    let sql = include_str!("../fixtures/test.sql").split(';');
    let mut ts = pool.begin().await.expect("begin transaction failed");
    
    for s in sql {
        if s.trim().is_empty() {
            continue;
        }
        ts.execute(s).await.expect("execute sql failed");
    }
    ts.commit().await.expect("commit transaction failed");
    let user: Vec<users> = sqlx::query_as(
        r#"
        SELECT * FROM users
        "#,
    )
    .fetch_all(&pool).await?;
    let path1 = "/Users/linyz/snapshot/cats.PNG";
    let ext1 = path1.split(".").last().unwrap_or("");
    let file_path = PathBuf::from("/Users/linyz/snapshot/cats.PNG");
    let file = fs::read(&file_path).await?;
    let sha1_hash_png = format!("{:x}", sha1::Sha1::digest(&file));
    let hash_path_png = format!("{}.{}", sha1_hash_png, ext1);
    // let delete_user: Vec<users> = sqlx::query_as(
    //     r#"
    //     DELETE FROM users
    //     WHERE id > 2
    //     RETURNING id, name, email, password, type
    //     "#
    // )
    // .fetch_all(&pool).await?;
    // println!("{:?}", delete_user);
    let text = "hello world";
    fs::write("/Users/linyz/snapshot/hello.txt", text).await?;
    let text = fs::read_to_string("/Users/linyz/snapshot/hello.txt").await?;
    let sh1_hash_text = format!("{:x}", sha1::Sha1::digest(text.as_bytes()));
    let hash_path_text = format!("{}.{}", sh1_hash_text, "txt");
    let messages: Vec<messages> = sqlx::query_as(
        r#"
        INSERT INTO messages (chat_id, sender_id, content, file)
        VALUES ($1, $2, $3, $4), 
               ($5, $6, $7, $8)
        RETURNING message_id, chat_id, sender_id, content, file, created_at
        "#,
    )
    .bind(1)
    .bind(&user[0].id)
    .bind("hello Bob!".to_string())
    .bind(vec![hash_path_png.clone()])
    .bind(1)
    .bind(&user[1].id)
    .bind("hello Alice!".to_string())
    .bind(vec![hash_path_text.clone()])
    .fetch_all(&pool)
    .await?;
    Ok(())
}

impl Config {
    fn load() -> Result<Self> {
        let config = match (
            File::open("app.yml"),
            File::open("/etc/app.yml"),
            env::var("CHAT_CONFIG"),
        ) {
            (Ok(file), _, _) => serde_yaml::from_reader(file),
            (_, Ok(file), _) => serde_yaml::from_reader(file),
            (_, _, Ok(env_var)) => serde_yaml::from_str(&env_var),
            _ => anyhow::bail!("No configuration file found"),
        }?;
        Ok(config)
    }
}
