use std::{collections::HashMap, io::Write, path::Path};

use rand::distr::SampleString;
use serde::{Deserialize, Serialize};
use sha2::Digest;

type Username = String;

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct State {
    users: HashMap<Username, User>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct User {
    pub username: Username,
    password_salt: String,
    password_hash: Vec<u8>,
    pub paste_ids: Vec<String>,
}

impl State {
    pub fn load(path: &Path) -> anyhow::Result<Self> {
        let Ok(reader) = std::fs::File::open(path) else {
            return Ok(Self::default());
        };
        let value = serde_json::from_reader(reader)?;
        Ok(value)
    }

    pub fn dump(&self, path: &Path) -> anyhow::Result<()> {
        let mut tmp_name = path.file_name().unwrap().to_owned();
        tmp_name.push("~");
        let tmp_path = path.with_file_name(tmp_name);

        let writer = std::fs::File::create(&tmp_path)?;
        serde_json::to_writer_pretty(writer, &self)?;
        std::fs::rename(tmp_path, path)?;
        Ok(())
    }

    pub fn create(&mut self, username: &str, password: &str) -> &User {
        let salt = gen_salt();
        let hash = hashed_password(&password, &salt);

        self.users.insert(
            username.to_owned(),
            User {
                username: username.to_owned(),
                password_hash: hash,
                password_salt: salt,
                paste_ids: Vec::new(),
            },
        );
        self.users.get(username).unwrap()
    }

    pub fn auth(&self, username: &str, password: &str) -> Option<&User> {
        let user = self.users.get(username)?;
        let hash = hashed_password(password, &user.password_salt);
        (hash == user.password_hash).then_some(user)
    }

    pub fn auth_mut(&mut self, username: &str, password: &str) -> Option<&mut User> {
        let user = self.users.get_mut(username)?;
        let hash = hashed_password(password, &user.password_salt);
        (hash == user.password_hash).then_some(user)
    }
}

fn hashed_password(password: &str, salt: &str) -> Vec<u8> {
    let mut digest = sha2::Sha256::new();
    digest
        .write_all(salt.as_bytes())
        .expect("Couldn't calculate hash");
    digest
        .write_all(password.as_bytes())
        .expect("Couldn't calculate hash");
    let hash = digest.finalize();
    Vec::from(&hash[..])
}

fn gen_salt() -> String {
    rand::distr::Alphanumeric.sample_string(&mut rand::rng(), 6)
}

#[test]
fn test_gen_salt() {
    println!("Salt: {}", gen_salt());
}
