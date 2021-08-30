use std::convert::{TryFrom, TryInto};

use rand::prelude::Distribution;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use crate::api_error::ApiError;

#[derive(Serialize, Deserialize)]
pub struct GameServerInfo {
    pub name: String,
    pub uri: String,
    pub developer: bool,
    pub fallback: bool,
    pub full: bool,
    pub maintenance: bool,
    #[serde(rename= "max-clients")]
    pub max_clients: Option<u32>,
    pub games: Vec<GameServerEntry>,
}

impl TryFrom<crate::db::model::ServerInfo> for GameServerInfo {
    type Error = ApiError;

    fn try_from(value: crate::db::model::ServerInfo) -> Result<Self, Self::Error> {
        Ok(GameServerInfo {
            name: value.name,
            uri: value.uri,
            developer: value.developer,
            fallback: value.fallback,
            full: value.full,
            maintenance: value.maintenance,
            max_clients: value.max_clients.map(|x| x as u32),
            games: crate::db::model::ServerGame::find_by_info(value.id)?
                .iter()
                .map(|x| x.into())
                .collect(),
        })
    }
}

#[derive(Serialize, Deserialize)]
pub struct GameServerEntry {
    pub name: String,
    pub uri: String,
    pub rooms: u32,
    #[serde(rename = "max-rooms")]
    pub max_rooms: Option<u32>,
    pub clients: u32,
}

impl From<crate::db::model::ServerGame> for GameServerEntry {
    fn from(value: crate::db::model::ServerGame) -> Self {
        GameServerEntry {
            name: value.name,
            uri: value.uri,
            rooms: value.rooms as u32,
            max_rooms: value.max_rooms.map(|x| x as u32),
            clients: value.clients as u32,
        }
    }
}

impl From<&crate::db::model::ServerGame> for GameServerEntry {
    fn from(value: &crate::db::model::ServerGame) -> Self {
        GameServerEntry {
            name: value.name.clone(),
            uri: value.uri.clone(),
            rooms: value.rooms as u32,
            max_rooms: value.max_rooms.map(|x| x as u32),
            clients: value.clients as u32,
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct GameServer {
    pub id: String,
    #[serde(rename = "last-seen")]
    pub last_seen: String,
    #[serde(rename = "last-seen-sec")]
    pub last_seen_sec: f32,
    pub info: GameServerInfo,
}

impl GameServer {
    pub fn save(&mut self, token: &str) -> Result<(), ApiError> {
        let now = chrono::Utc::now().naive_utc();
        let id = 
            if let Some(mut old) = crate::db::model::Server::find_by_token(token).ok() {
            // if let Some(mut old) = GameServer::get_server(&self.id) {
                let info = crate::db::model::ServerInfo::find_by_server(old.id)?;
                crate::db::model::ServerGame::delete_by_info(info.id)?;
                crate::db::model::ServerInfo::delete(info.id)?;
                old.last_seen = now;
                old.token = token.to_string();
                old = crate::db::model::Server::update(old)?;
                old.id
            } else {
                let mut entry = crate::db::model::Server {
                    id: Uuid::new_v4(),
                    last_seen: now,
                    token: token.to_string(),
                    created_at: now,
                    updated_at: None,
                };
                entry = crate::db::model::Server::create(entry)?;
                entry.id
            };
        
        let mut info = crate::db::model::ServerInfo {
            id: Uuid::new_v4(),
            name: self.info.name.clone(),
            uri: self.info.uri.clone(),
            developer: self.info.developer,
            fallback: self.info.fallback,
            full: self.info.full,
            maintenance: self.info.maintenance,
            max_clients: self.info.max_clients
                .map(|x| x as i32),
            server_id: id,
            created_at: now,
            updated_at: None,
        };
        info = crate::db::model::ServerInfo::create(info)?;

        for game in &self.info.games {
            crate::db::model::ServerGame::create(
                crate::db::model::ServerGame {
                    id: Uuid::new_v4(),
                    name: game.name.clone(),
                    uri: game.uri.clone(),
                    rooms: game.rooms as i32,
                    max_rooms: game.max_rooms.map(|x| x as i32),
                    clients: game.clients as i32,
                    game_info_id: info.id,
                    created_at: now,
                    updated_at: None,
                }
            )?;
        }

        self.id = id.to_simple()
            .encode_lower(&mut Uuid::encode_buffer())
            .to_string();
        self.last_seen = now.to_string();
        self.last_seen_sec = 0.0;

        Ok(())
    }
}

impl TryFrom<crate::db::model::Server> for GameServer {
    type Error = ApiError;

    fn try_from(value: crate::db::model::Server) -> Result<Self, Self::Error> {
        Ok(GameServer {
            id: Uuid::to_simple(value.id)
                .encode_lower(&mut Uuid::encode_buffer())
                .to_string(),
            last_seen: chrono::NaiveDateTime::to_string(&value.last_seen),
            last_seen_sec: chrono::Utc::now()
                .naive_utc()
                .signed_duration_since(value.last_seen)
                .num_milliseconds() as f32
                * 0.001,
            info: crate::db::model::ServerInfo::find_by_server(value.id)?
                .try_into()?
        })
    }
}

impl TryFrom<(crate::db::model::Server, crate::db::model::ServerInfo)> for GameServer {
    type Error = ApiError;

    fn try_from((v1, v2): (crate::db::model::Server, crate::db::model::ServerInfo)) -> Result<Self, Self::Error> {
        Ok(GameServer {
            id: Uuid::to_simple(v1.id)
                .encode_lower(&mut Uuid::encode_buffer())
                .to_string(),
            last_seen: chrono::NaiveDateTime::to_string(&v1.last_seen),
            last_seen_sec: chrono::Utc::now()
                .naive_utc()
                .signed_duration_since(v1.last_seen)
                .num_milliseconds() as f32
                * 0.001,
            info: v2.try_into()?
        })
    }
}

#[derive(Serialize, Deserialize)]
pub struct UpdateResponse {
    pub id: String,
}

#[derive(Serialize, Deserialize)]
pub struct ListQuery {
    #[serde(rename = "include-dev")]
    pub include_dev: Option<bool>,
    #[serde(rename = "include-fallback")]
    pub include_fallback: Option<bool>,
    #[serde(rename = "exclude-full")]
    pub exclude_full: Option<bool>,
}

#[derive(Serialize, Deserialize)]
pub struct ListResponse (pub Vec<GameServer>);


#[derive(Serialize, Deserialize)]
pub struct NewRequest {
    pub game: String,
    pub developer: Option<bool>,
    pub fallback: Option<bool>,
    pub ignore: Option<Vec<String>>,
}

#[derive(Serialize, Deserialize)]
pub struct NewResponse {
    pub id: String,
    #[serde(rename = "api-uri")]
    pub api_uri: String,
    #[serde(rename = "game-uri")]
    pub game_uri: String,
}

#[derive(Serialize, Deserialize)]
pub struct FastTokenAddRequest {
    pub game: String,
    pub lobby: String,
}

const MAX_FAST_TOKEN_SIZE: usize = 4;

impl TryFrom<(Uuid, FastTokenAddRequest)> for crate::db::model::FastToken {
    type Error = ApiError;

    fn try_from((server_id, value): (Uuid, FastTokenAddRequest)) -> Result<Self, Self::Error> {
        let now = chrono::Utc::now();
        let limit = now.checked_sub_signed(
            chrono::Duration::minutes(20)
        )
            .expect("limit traveled back in time")
            .naive_utc();
        let range = "ABCDEFGHIJKLMOPQRSTUVWXYZ0123456789";
        let mut rng = rand::thread_rng();
        let dist = rand::distributions::Uniform::new(0, range.len());
        let mut token = String::with_capacity(MAX_FAST_TOKEN_SIZE);
        loop {
            token.clear();
            for _ in 0..MAX_FAST_TOKEN_SIZE {
                token.push(range.chars().nth(dist.sample(&mut rng))
                    .expect("random out of range")
                );
            }
            if let Err(_) = crate::db::model::FastToken::find_by_token_checked(&token, limit) {
                return crate::db::model::FastToken::create(
                    crate::db::model::FastToken {
                        id: Uuid::new_v4(),
                        token,
                        server_id,
                        game: value.game,
                        lobby: value.lobby,
                        created_at: now.naive_utc(),
                        updated_at: None,
                    }
                );
            }
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct FastTokenAddResponse {
    pub token: String,
}

impl From<crate::db::model::FastToken> for FastTokenAddResponse {
    fn from(value: crate::db::model::FastToken) -> Self {
        FastTokenAddResponse {
            token: value.token,
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct FastTokenFetchResponse {
    pub server: String,
    pub game: String,
    pub lobby: String,
    #[serde(rename = "api-uri")]
    pub api_uri: String,
    #[serde(rename = "game-uri")]
    pub game_uri: Option<String>,
}

impl TryFrom<crate::db::model::FastToken> for FastTokenFetchResponse {
    type Error = ApiError;

    fn try_from(value: crate::db::model::FastToken) -> Result<Self, Self::Error> {
        let server: GameServer = crate::db::model::Server::find_by_id(value.server_id)?
            .try_into()?;
        for game in &server.info.games {
            if game.name == value.game {
                return Ok(FastTokenFetchResponse {
                    server: Uuid::to_simple(value.server_id)
                        .encode_lower(&mut Uuid::encode_buffer())
                        .to_string(),
                    game: value.game,
                    lobby: value.lobby,
                    api_uri: server.info.uri,
                    game_uri: Some(game.uri.clone()),
                });
            }
        }

        Ok(FastTokenFetchResponse {
            server: Uuid::to_simple(value.server_id)
                .encode_lower(&mut Uuid::encode_buffer())
                .to_string(),
            game: value.game,
            lobby: value.lobby,
            api_uri: server.info.uri,
            game_uri: None,
        })
    }
}