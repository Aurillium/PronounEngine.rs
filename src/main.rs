mod commands;
mod shared;

mod engine;
mod sentences;
mod socktest;

use engine::{PronounSet, genderify_text, parse_set};
use sentences::SentenceType;
use mysql_async::Pool;
use shared::console_stamp as cs;

use once_cell::sync::Lazy;

use std::env;
use std::fs;
use std::time::Instant;

use serde::{Deserialize, Serialize};

use tokio::io::{self, AsyncReadExt, AsyncWriteExt};
use tokio::net::{UnixStream, UnixListener};

#[derive(Debug, Serialize, Deserialize)]
struct Config {
    testing_mode: bool,
    database: DBConfig,
    engine_socket: String
}
#[derive(Debug, Serialize, Deserialize)]
struct DBConfig {
    address: String,
    port: i32,
    username: String,
    password: String,
    database: String,
    salt: String
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag="name")]
#[serde(rename_all = "snake_case")]
enum Command {
    Genderify {
        text: String,
        names: Vec<String>,
        sets: Vec<PronounSet>
    },
    Sentences(SentenceType),
    Parse {
        raw: String
    }
}

static CONFIG: Lazy<Config> = Lazy::new(|| {
    let contents = fs::read_to_string("config.json")
        .expect("Config file reading failed.");
    serde_json::from_str(contents.as_str())
        .expect("JSON deserialisation error.")
});

static DB: Lazy<Pool> = Lazy::new(|| {
    let url = format!("mysql://{}:{}@{}:{}/{}", CONFIG.database.username, CONFIG.database.password, CONFIG.database.address, CONFIG.database.port, CONFIG.database.database);
    mysql_async::Pool::new(url.as_str())
});

async fn handle_client(stream: UnixStream) {
    let conn = match DB.get_conn().await {
        Ok(conn) => conn,
        Err(error) => {
            print!("{}Error connecting to database: {error}", cs());
            return;
        }
    };

    let (mut reader, mut writer) = io::split(stream);

    loop {
        // Start your timer!
        let now = Instant::now();

        let mut buf = vec![0; 128];
        let mut full_data = Vec::<u8>::new();

        loop {
            let n = reader.read(&mut buf).await.unwrap();
            full_data.append(&mut buf);
            if n == 0 {
                break;
            }
        }

        let command: Command = match serde_json::from_slice(&full_data) {
            Ok(command) => command,
            Err(error) => {
                // We should send back a signal that the input was invalid
                println!("{}Error at {}ms: {error}", cs(), now.elapsed().as_millis());
                return;
            }
        };

        match command {
            Command::Genderify { text, names, sets } => {
                println!("gender");
            },
            Command::Sentences(t) => {
                println!("setences");
            },
            Command::Parse { raw } => {
                println!("parse");
            }
        }

        // How long did it take?
        println!("{}{}: {}ms", cs(), "Processed request", now.elapsed().as_millis());
    }
}

#[tokio::main]
async fn main() {
    let socket_path = match env::var("XDG_RUNTIME_DIR") {
        Ok(var) => var,
        Err(error) => {
            println!("{}Couldn't get run directory environment variable: {error}", cs());
            return;
        }
    } + "/" + &CONFIG.engine_socket;

    let listener = match UnixListener::bind(socket_path) {
        Ok(listener) => listener,
        Err(error) => {
            println!("{}Could bind socket: {error}", cs());
            return;
        }
    };

    println!("{}Accepting connections now!", cs());
    loop {
        match listener.accept().await {
            Ok((stream, _addr)) => {
                println!("{}New client!", cs());
                handle_client(stream).await;
            }
            Err(error) => {
                println!("{}Connection error: {error}", cs());
            }
        }
    }
}
// println!("{}Error on command '{}': {}", cs(), interaction.data.name, error.to_string());