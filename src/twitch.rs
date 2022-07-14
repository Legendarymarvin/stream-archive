use std::fs;
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};
use urlencoding::encode;
use crate::cached;
use log::{error, info};

#[derive(Serialize, Deserialize, Debug)]
struct Token {
    pub(crate) access_token: String,
    expires_in: u32,
    token_type: String,
}

#[derive(Clone)]
struct Auth {
    client_id: String,
    bearer_token: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ChannelInfo {
    pub(crate) user_login: String,
    pub(crate) user_name: String,
    pub(crate) game_name: String,
    #[serde(alias = "type")]
    pub(crate) typ: String,
    pub(crate) title: String,
    pub(crate) started_at: String,
    pub(crate) language: String,
}

impl ChannelInfo {
    pub fn new(user_login: String, user_name: String, game_name: String, typ: String, title: String, started_at: String, language: String) -> Self {
        Self { user_login, user_name, game_name, typ, title, started_at, language }
    }
}

pub fn get_live_channels(channel_names: &Vec<String>) -> Vec<ChannelInfo> {
    let auth = get_bearer_token();
    let channels = channel_names
        .iter()
        .map(|channel_name| encode(channel_name))
        .collect::<Vec<_>>()
        .join("&user_login=");
    let api_url = format!("https://api.twitch.tv/helix/streams?user_login={}", channels);

    let client = reqwest::blocking::Client::new();
    let res = client.get(&api_url)
        .header("Client-ID", auth.client_id)
        .header("Authorization", format!("Bearer {}", auth.bearer_token))
        .header("Accept", "application/vnd.twitchtv.v5+json")
        .send().unwrap().text().unwrap();

    let json: Map<String, Value> = match serde_json::from_str(&res).unwrap() {
        Value::Object(j) => j,
        _ => panic!("Unexpected Json")
    };

    let results: &Vec<Value> = match json.get("data") {
        Some(data) => {
            match data {
                Value::Array(j) => j,
                j => {
                    error!("Invalid json {:?}", j);
                    return Vec::new();
                }
            }
        }
        None => {
            error!("No data in json response {:?}", json);
            return Vec::new();
        }
    };

    parse_json_results(results)
}

fn parse_json_results(results: &Vec<Value>) -> Vec<ChannelInfo> {
    let mut live_channels: Vec<ChannelInfo> = Vec::new();
    for channel_info in results {
        match channel_info {
            Value::Object(value) => {
                live_channels.push(ChannelInfo::new(
                    extract_from_json(&value, "user_login"),
                    extract_from_json(&value, "user_name"),
                    extract_from_json(&value, "game_name"),
                    extract_from_json(&value, "type"),
                    extract_from_json(&value, "title"),
                    extract_from_json(&value, "started_at"),
                    extract_from_json(&value, "language"),
                ));
            }
            _ => { panic!("Unexpected Json") }
        }
    }
    live_channels
}

fn extract_from_json(value: &Map<String, Value>, key: &str) -> String {
    match value.get(key).unwrap() {
        Value::String(j) => j.to_string(),
        _ => panic!("Could not parse {} from json {:?}.", key, value)
    }
}

#[cached(size = 1, time = 2_500_000)]
fn get_bearer_token() -> Auth {
    info!("Getting Bearer token");
    let login: (String, String) = read_client_id_and_secret();
    let client = reqwest::blocking::Client::new();
    let url = format!("https://id.twitch.tv/oauth2/token?client_id={}&client_secret={}&grant_type=client_credentials",
                      login.0,
                      login.1);
    let res = client.post(url).send().unwrap().text().unwrap();
    let token: Token = serde_json::from_str(&res).unwrap();
    Auth { client_id: login.0, bearer_token: token.access_token }
}

fn read_client_id_and_secret() -> (String, String) {
    let data = fs::read_to_string("config.json").expect("Couldn't find config.json, create one using config-example.json.");
    let json: Map<String, Value> = match serde_json::from_str(&data).unwrap() {
        Value::Object(j) => j,
        _ => panic!("Invalid Json")
    };
    (extract_from_json(&json, "client-id"), extract_from_json(&json, "client-secret"))
}
