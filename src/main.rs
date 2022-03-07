#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
use async_std::{task, sync::{Mutex, Arc}};
use hass_rs::client;
use mki::{Keyboard};
use once_cell::sync::{OnceCell, Lazy};
use core::panic;
use std::{str::FromStr, fs, path::PathBuf, sync::mpsc};
use serde::{Serialize, Deserialize};
use directories::ProjectDirs;
use tray_item::TrayItem;
use log::*;
use simplelog::*;
use edit;
use msgbox::IconType;

macro_rules! crash {
    ( $( $p:tt ),* ) => {
            {
            error!($($p),*);
            msgbox::create("Hss hotkeys - Error", format!($($p),*).as_str(), IconType::Error).unwrap();
            panic!($($p),*);
        }
    };
}

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

static CLIENT: OnceCell<std::sync::Arc<async_std::sync::Mutex<hass_rs::HassClient>>> = OnceCell::new();
static CONFIG_DIR: Lazy<PathBuf> = Lazy::new(||{
    let path = ProjectDirs::from("", "",  "hass_hotkeys").unwrap();
    let path = path.config_dir();
    fs::create_dir_all(path).expect("Can't create project dir");
    let path = path.join("config.yaml");
    if !path.exists() {
        fs::write(&path, b"# example config.yaml\r\nhass_host: #replace your home assistant ip or domain (string)\r\nhass_port: 8123 #replace with your home assistant websocket port (number)\r\nhass_token: #replace with a long lived access token (string)\r\nactions:\r\n  - action_type: call_service\r\n    description: Toggle Lab lights when pressing LeftCtrl & R\r\n    keys:\r\n      - LeftControl\r\n      - R\r\n    domain: light\r\n    service: toggle\r\n    service_data:\r\n       entity_id: light.lab_lights").unwrap();
        crash!("Config file created at {:?}, please edit the file to match your setup.", path)
    }
    path
});

fn main() {
    CombinedLogger::init(
        vec![
            TermLogger::new(LevelFilter::Warn, Config::default(), TerminalMode::Mixed, ColorChoice::Auto),
            WriteLogger::new(LevelFilter::Info, Config::default(), fs::File::create(CONFIG_DIR.as_path().parent().unwrap().parent().unwrap().join("log.txt")).unwrap()),
        ]
    ).unwrap();
    task::block_on(app());
}

#[cfg(windows)]
enum Message {
    Quit,
}
#[cfg(windows)]
fn tray() {
    let mut tray = TrayItem::new("Hass Hotkeys", "ico").unwrap();

    tray.add_label("Hass Hotkeys").unwrap();

    tray.add_menu_item("Edit Config", || {
        edit::edit_file(CONFIG_DIR.as_path()).unwrap();
    })
    .unwrap();

    let (tx, rx) = mpsc::channel();

    tray.add_menu_item("Quit", move || {
        println!("Quit");
        tx.send(Message::Quit).unwrap();
    })
    .unwrap();

    loop {
        match rx.recv() {
            Ok(Message::Quit) => break,
            _ => {}
        }
    }
}

async fn app(){
    //read config
    let config_file = read_config(CONFIG_DIR.to_path_buf());
    let (config,key, host,port) = parse_config(config_file);
    // init the client
    init_client(host, port, key).await;
    // create hotkeys
    for entry in config {
        create_hotkey(&entry.0, entry.1, entry.2, entry.3);
    }
     // create tray
    tray();
    
    // do not exit
    //io::stdin().read_line(&mut String::new()).unwrap();
}

async fn init_client(host: String, port: u16, key: String){
    info!("Creating the Websocket Client and Authenticate the session");
    let client = Arc::new(Mutex::new(
        match client::connect(&host, port).await {
            Ok(hass) => hass,
            Err(e) => crash!("Config Error\n{}", e),
        }
    ));
    match client.lock().await.auth_with_longlivedtoken(&key).await {
        Ok(_) => (),
        Err(e) => crash!("Auth Error\n{}", e)
    };
    CLIENT.set(client).ok();
    info!("WebSocket connection and authethication works");
}

fn create_hotkey(key: &[Keyboard], domain: String, service: String, data: serde_json::Value) {
   mki::register_hotkey(key, move || task::block_on(make_call(domain.to_owned(), service.to_owned(), data.to_owned())));
}

async fn make_call(domain: String, service: String, data: serde_json::Value ) {
    let mut client = CLIENT.get().unwrap().lock().await;
    match client.call_service(domain, service, Some(data)).await {
        Ok(v) => info!("{:?}", v),
        Err(err) => {
            msgbox::create("Hss hotkeys - Error", format!("Error calling service: {}", err).as_str(), IconType::Info).unwrap();
            error!("Error calling service: {}", err)
        },
    }
}

fn read_config(path: PathBuf) -> ConfigFlie  {
    let config = match fs::read_to_string(path) {
        Ok(config) => config,
        Err(error) =>  crash!("Problem loading config file: {:?}", error),
    };
    let config = match serde_yaml::from_str::<ConfigFlie>(&config) {
        Ok(config) => config,
        Err(error) => crash!("Invalid config: {:?}", error),
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
                Err(_) => crash!("Invalid key '{}' in config file", key),
            }
        };
        if entry.action_type == "call_service" {
          let domain = match entry.domain {
              Some(s) => s,
              _ => crash!("Action type is 'call_service' but config file is missing key 'domain'")
          };
          let service = match entry.service {
            Some(s) => s,
            _ => crash!("Action type is 'call_service' but config file is missing key 'service'")
        };
        let service_data = match entry.service_data {
            Some(s) => s,
            _ => crash!("Action type is 'call_service' but config file is missing key 'service_data'")
        };
           result.push((keys, domain, service, service_data));
        }
    }
    (result, config.hass_token, config.hass_host, config.hass_port)
}