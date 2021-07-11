use lazy_static::lazy_static;
use std::env;
use std::fs::File;
use std::io::{self, BufRead};

type Store = Vec<String>;

lazy_static! {
    static ref STORE: Store = {
        let path = env::var("TOKEN_FILE").expect("Token file not set");
        let file = File::open(path).expect("cannot open token file");
        let mut store = Vec::new();
        for line in io::BufReader::new(file).lines() {
            if let Ok(x) = line {
                if x.starts_with("#") || x.len() == 0 {
                    continue;
                }
                store.push(x);
            }
        }
        info!("{} servers are authorized", store.len());
        store
    };
}

pub fn init() {
    info!("Initializing Tokens");
    lazy_static::initialize(&STORE);
}

pub fn has_token(token: &str) -> bool {
    STORE.iter()
        .any(|x| x.as_str() == token)
}