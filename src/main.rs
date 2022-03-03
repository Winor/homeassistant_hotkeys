use async_std::task;
use hass_rs::client;
use mki::{Keyboard};
use once_cell::sync::OnceCell;
use core::panic;
use std::{io, sync::{Arc, Mutex}, str::FromStr, fs, path::PathBuf};
use serde::{Serialize, Deserialize};
use clap::Parser;
use directories::ProjectDirs;

#[derive(Debug, PartialEq, Serialize, Deserialize)]
struct ConfigFlie {
    hass_host: String,
    hass_port: u16,
    hass_token: String,
    actions: Vec<ConfigEntry>,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
struct ConfigEntry {
    action_type: String,
    description: String,
    keys: Vec<String>,
    domain: Option<String>,
    service: Option<String>,
    service_data: Option<serde_json::Value>,
}

static CLIENT: OnceCell<std::sync::Arc<std::sync::Mutex<hass_rs::HassClient>>> = OnceCell::new();

#[async_std::main]
async fn main() {
    // get path
    let path = proj_conf_path();
    //read config
    let config_file = read_config(path);
    let (config,key, host,port) = parse_config(config_file);
    // init the client
    init_client(host, port, key).await.ok();
    // create hotkeys
    for entry in config {
        create_hotkey(&entry.0, entry.1, entry.2, entry.3);
    }
    
    // do not exit
    io::stdin().read_line(&mut String::new()).unwrap();
}

fn proj_conf_path() -> PathBuf {
    let path = ProjectDirs::from("", "",  "hass_hotkeys").unwrap();
    let path = path.config_dir();
    fs::create_dir_all(path).expect("Can't create project dir");
    let path = path.join("config.yaml");
    if !path.exists() {
        fs::write(&path, b"# example config.yaml\r\nhass_host: #replace your home assistant ip or domain (string)\r\nhass_port: 8123 #replace with your home assistant websocket port (number)\r\nhass_token: #replace with a long lived access token (string)\r\nactions:\r\n  - action_type: call_service\r\n    description: Toggle Lab lights when pressing LeftCtrl & R\r\n    keys:\r\n      - LeftControl\r\n      - R\r\n    domain: light\r\n    service: toggle\r\n    service_data:\r\n       entity_id: light.lab_lights");
        panic!("Config file created at {:?}, please edit the file to match your setup.", path)
    }
    path
}

async fn init_client(host: String, port: u16, key: String) -> Result<(), Box<dyn std::error::Error>> {
    println!("Creating the Websocket Client and Authenticate the session");
    let client = Arc::new(Mutex::new(client::connect(&host, port).await?));
    client.lock().unwrap().auth_with_longlivedtoken(&key).await?;
    CLIENT.set(client).ok();
    println!("WebSocket connection and authethication works");
    Ok(())
}

fn create_hotkey(key: &[Keyboard], domain: String, service: String, data: serde_json::Value) {
   mki::register_hotkey(key, move || task::block_on(make_call(domain.to_owned(), service.to_owned(), data.to_owned())));
}

async fn make_call(domain: String, service: String, data: serde_json::Value ) {
    let mut client = CLIENT.get().unwrap().lock().unwrap();
    match client.call_service(domain, service, Some(data)).await {
        Ok(v) => println!("{:?}", v),
        Err(err) => println!("Oh no, an error: {}", err),
    }
}

fn read_config(path: PathBuf) -> ConfigFlie  {
    let config = match fs::read_to_string(path) {
        Ok(config) => config,
        Err(error) =>  panic!("Problem loading config file: {:?}", error),
    };
    let config = match serde_yaml::from_str::<ConfigFlie>(&config) {
        Ok(config) => config,
        Err(error) => panic!("Invalid config: {:?}", error),
    };
    config
}

fn parse_config(config: ConfigFlie) -> (Vec<(Vec<Keyboard>, String, String, serde_json::Value)>, String, String, u16) {
    let mut result: Vec<(Vec<Keyboard>, String, String, serde_json::Value)> = Vec::new();
    for entry in config.actions {
        let mut keys = Vec::new();
        for key in entry.keys {
            match Keyboard::from_str(&key) {
                Ok(key) => keys.push(key),
                Err(_) => panic!("Invalid key '{}' in config file", key),
            }
        };
        if entry.action_type == "call_service" {
          let domain = entry.domain.expect("Action type is 'call_service' but config file is missing key 'domain'");
           let service = entry.service.expect("Action type is 'call_service' but config file is missing key 'service'");
           let service_data = entry.service_data.expect("Action type is 'call_service' but config file is missing key 'service_data'");
           result.push((keys, domain, service, service_data));
        }
    }
    (result, config.hass_token, config.hass_host, config.hass_port)
}