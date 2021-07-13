use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use diesel::prelude::*;
use crate::api_error::ApiError;
use crate::schema::{server, server_game, server_info, fast_token};

#[derive(Serialize, Deserialize, AsChangeset, Queryable, Insertable)]
#[table_name = "server"]
pub struct Server {
    pub id: Uuid,
    pub last_seen: NaiveDateTime,
    pub token: String,
    pub created_at: NaiveDateTime,
    pub updated_at: Option<NaiveDateTime>,
}

impl Server {
    pub fn find_all() -> Result<Vec<Self>, ApiError> {
        let conn = crate::db::connection()?;

        let servers = server::table
            .load::<Server>(&conn)?;

        Ok(servers)
    }

    pub fn find_by_filter(
        include_dev: bool, 
        include_fallback: bool, 
        exclude_full: bool
    ) -> Result<Vec<(Self, ServerInfo)>, ApiError> {
        let conn = crate::db::connection()?;

        let mut result = server::table
            .inner_join(server_info::table)
            .into_boxed();
        
        if !include_dev {
            result = result
                .filter(server_info::developer.eq(false));
        }
        if !include_fallback {
            result = result
                .filter(server_info::fallback.eq(false));
        }
        if !exclude_full {
            result = result
                .filter(server_info::full.eq(false));
        }

        let result = result.load::<(Self, ServerInfo)>(&conn)?;

        Ok(result)
    }

    pub fn find_by_id(id: Uuid) -> Result<Self, ApiError> {
        let conn = crate::db::connection()?;

        let server = server::table
            .filter(server::id.eq(id))
            .first(&conn)?;
        
        Ok(server)
    }

    pub fn find_by_token(token: &str) -> Result<Self, ApiError> {
        let conn = crate::db::connection()?;

        let server = server::table
            .filter(server::token.eq(token))
            .first(&conn)?;
        
        Ok(server)
    }

    pub fn create(server: Server) -> Result<Self, ApiError> {
        let conn = crate::db::connection()?;

        let server = diesel::insert_into(server::table)
            .values(server)
            .get_result(&conn)?;
        
        Ok(server)
    }

    pub fn update(server: Server) -> Result<Self, ApiError> {
        let conn = crate::db::connection()?;

        let server = diesel::update(server::table)
            .filter(server::id.eq(server.id))
            .set(server)
            .get_result(&conn)?;
        
        Ok(server)
    }

    pub fn delete(id: Uuid) -> Result<usize, ApiError> {
        let conn = crate::db::connection()?;

        let res = diesel::delete(
            server::table
                .filter(server::id.eq(id))
        ).execute(&conn)?;

        Ok(res)
    }
}

#[derive(Serialize, Deserialize, AsChangeset, Queryable, Insertable)]
#[table_name = "server_info"]
pub struct ServerInfo {
    pub id: Uuid,
    pub name: String,
    pub uri: String,
    pub developer: bool,
    pub fallback: bool,
    pub full: bool,
    pub maintenance: bool,
    pub max_clients: Option<i32>,
    pub server_id: Uuid,
    pub created_at: NaiveDateTime,
    pub updated_at: Option<NaiveDateTime>,
}

impl ServerInfo {
    pub fn find_all() -> Result<Vec<Self>, ApiError> {
        let conn = crate::db::connection()?;

        let infos = server_info::table
            .load::<ServerInfo>(&conn)?;
        
        Ok(infos)
    }

    pub fn find_by_server(server_id: Uuid) -> Result<Self, ApiError> {
        let conn = crate::db::connection()?;

        let info = server_info::table
            .filter(server_info::server_id.eq(server_id))
            .first(&conn)?;

        Ok(info)
    }

    pub fn find_by_filter(
        incl_developer: bool, 
        incl_fallback: bool, 
        excl_full: bool
    ) -> Result<Vec<Self>, ApiError> {
        let conn = crate::db::connection()?;

        let mut infos = server_info::table
            .filter(server_info::developer.eq(true))
            .into_boxed();
        if !incl_developer {
            infos = infos.filter(server_info::developer.eq(false));
        }
        if !incl_fallback {
            infos = infos.filter(server_info::fallback.eq(false));
        }
        if excl_full {
            infos = infos.filter(server_info::full.eq(false));
        }

        let infos = infos.load::<ServerInfo>(&conn)?;

        Ok(infos)
    }

    pub fn create(value: Self) -> Result<Self, ApiError> {
        let conn = crate::db::connection()?;

        let res = diesel::insert_into(server_info::table)
            .values(value)
            .get_result(&conn)?;
        
        Ok(res)
    }

    pub fn update(value: Self) -> Result<Self, ApiError> {
        let conn = crate::db::connection()?;

        let res = diesel::update(server_info::table)
            .filter(server_info::id.eq(value.id))
            .set(value)
            .get_result(&conn)?;
        
        Ok(res)
    }

    pub fn delete(id: Uuid) -> Result<usize, ApiError> {
        let conn = crate::db::connection()?;

        let res = diesel::delete(
            server_info::table
                .filter(server_info::id.eq(id))
        ).execute(&conn)?;

        Ok(res)
    }
}

#[derive(Serialize, Deserialize, AsChangeset, Queryable, Insertable)]
#[table_name = "server_game"]
pub struct ServerGame {
    pub id: Uuid,
    pub name: String,
    pub uri: String,
    pub rooms: i32,
    pub max_rooms: Option<i32>,
    pub clients: i32,
    pub game_info_id: Uuid,
    pub created_at: NaiveDateTime,
    pub updated_at: Option<NaiveDateTime>,
}

impl ServerGame {
    pub fn find_all() -> Result<Vec<Self>, ApiError> {
        let conn = crate::db::connection()?;

        let servers = server_game::table
            .load::<ServerGame>(&conn)?;

        Ok(servers)
    }

    pub fn find_by_id(id: Uuid) -> Result<Self, ApiError> {
        let conn = crate::db::connection()?;

        let game = server_game::table
            .filter(server_game::id.eq(id))
            .first(&conn)?;
        
        Ok(game)
    }

    pub fn find_by_info(server_info_id: Uuid) -> Result<Vec<Self>, ApiError> {
        let conn = crate::db::connection()?;

        let games = server_game::table
            .filter(server_game::game_info_id.eq(server_info_id))
            .load::<ServerGame>(&conn)?;
        
        Ok(games)
    }

    pub fn create(game: Self) -> Result<Self, ApiError> {
        let conn = crate::db::connection()?;

        let server = diesel::insert_into(server_game::table)
            .values(game)
            .get_result(&conn)?;

        Ok(server)
    }

    pub fn update(game: Self) -> Result<Self, ApiError> {
        let conn = crate::db::connection()?;

        let game = diesel::update(server_game::table)
            .filter(server_game::id.eq(game.id))
            .set(game)
            .get_result(&conn)?;

        Ok(game)
    }

    pub fn delete(id: Uuid) -> Result<usize, ApiError> {
        let conn = crate::db::connection()?;

        let res = diesel::delete(
            server_game::table
                .filter(server_game::id.eq(id))
        ).execute(&conn)?;

        Ok(res)
    }

    pub fn delete_by_info(game_info_id: Uuid) -> Result<usize, ApiError> {
        let conn = crate::db::connection()?;

        let res = diesel::delete(
            server_game::table
                .filter(server_game::game_info_id.eq(game_info_id))
        ).execute(&conn)?;

        Ok(res)
    }
}

#[derive(Serialize, Deserialize, AsChangeset, Queryable, Insertable)]
#[table_name = "fast_token"]
pub struct FastToken {
    pub id: Uuid,
    pub token: String,
    pub server_id: Uuid,
    pub game: String,
    pub lobby: String,
    pub created_at: NaiveDateTime,
    pub updated_at: Option<NaiveDateTime>,
}

impl FastToken {
    pub fn find_all() -> Result<Vec<Self>, ApiError> {
        let conn = crate::db::connection()?;

        let result = fast_token::table
            .load::<FastToken>(&conn)?;
        
        Ok(result)
    }

    pub fn find_by_id(id: Uuid) -> Result<Self, ApiError> {
        let conn = crate::db::connection()?;

        let result = fast_token::table
            .filter(fast_token::id.eq(id))
            .first(&conn)?;

        Ok(result)
    }
    
    pub fn find_by_token(token: &String) -> Result<Self, ApiError> {
        let conn = crate::db::connection()?;
        
        let result = fast_token::table
        .filter(fast_token::token.eq(token))
        .first(&conn)?;
        
        Ok(result)
    }
    
    pub fn find_by_token_checked(token: &String, limit: NaiveDateTime) -> Result<Self, ApiError> {
        let conn = crate::db::connection()?;

        let result = fast_token::table
            .filter(fast_token::token.eq(token))
            .filter(fast_token::created_at.gt(limit))
            .first(&conn)?;

        Ok(result)
    }
    
    pub fn create(entry: Self) -> Result<Self, ApiError> {
        let conn = crate::db::connection()?;

        let result = diesel::insert_into(fast_token::table)
            .values(entry)
            .get_result(&conn)?;
        
        Ok(result)
    }

    pub fn update(entry: Self) -> Result<Self, ApiError> {
        let conn = crate::db::connection()?;

        let result = diesel::update(fast_token::table)
            .filter(fast_token::id.eq(entry.id))
            .set(entry)
            .get_result(&conn)?;

        Ok(result)
    }

    pub fn delete(id: Uuid) -> Result<usize, ApiError> {
        let conn = crate::db::connection()?;

        let res = diesel::delete(
            fast_token::table
                .filter(fast_token::id.eq(id))
        ).execute(&conn)?;

        Ok(res)
    }
}
