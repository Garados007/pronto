// #[macro_use]
// extern crate log;

use std::convert::TryInto;

use actix_web::{ HttpResponse, Responder, get, post, web};
use actix_files::NamedFile;
use serde_json::json;
use uuid::Uuid;
use super::model::*;

fn get_header(req: &web::HttpRequest, name: &str) -> Option<String> {
    Some(req.headers()
        .get(name)?
        .to_str()
        .ok()?
        .to_string()
    )
}

#[get("/")]
async fn redirect() -> impl Responder {
    HttpResponse::TemporaryRedirect()
        .append_header((actix_web::http::header::LOCATION, "/v1"))
        .finish()
}

#[get("/v1")]
async fn index() -> actix_web::Result<NamedFile> {
    // let file = NamedFile::open("./open-api-v1.yml")
    let file = NamedFile::open("./doc.html")
        .map_err(|x| {
            warn!("{}", x);
            actix_web::error::ErrorNotImplemented(x)
        });
    file.map(|x|
        x.set_content_disposition(actix_web::http::header::ContentDisposition {
            disposition: actix_web::http::header::DispositionType::Inline,
            parameters: vec![],
        })
        .set_content_type(mime::TEXT_HTML_UTF_8)
    )
}

#[get("/v1.yml")]
async fn index_yml() -> actix_web::Result<NamedFile> {
    // let file = NamedFile::open("./open-api-v1.yml")
    let file = NamedFile::open("./open-api-v1.yml")
        .map_err(|x| {
            warn!("{}", x);
            actix_web::error::ErrorNotImplemented(x)
        });
    file.map(|x|
        x.set_content_disposition(actix_web::http::header::ContentDisposition {
            disposition: actix_web::http::header::DispositionType::Inline,
            parameters: vec![],
        })
        .set_content_type(mime::TEXT_PLAIN_UTF_8)
    )
}

#[get("/v1.json")]
async fn index_json() -> actix_web::Result<NamedFile> {
    // let file = NamedFile::open("./open-api-v1.yml")
    let file = NamedFile::open("./open-api-v1.json")
        .map_err(|x| {
            warn!("{}", x);
            actix_web::error::ErrorNotImplemented(x)
        });
    file.map(|x|
        x.set_content_disposition(actix_web::http::header::ContentDisposition {
            disposition: actix_web::http::header::DispositionType::Inline,
            parameters: vec![],
        })
        .set_content_type(mime::APPLICATION_JSON)
    )
}

#[post("/v1/update")]
async fn update(req: web::HttpRequest, request: web::Json<GameServerInfo>) -> impl Responder {
    let token = match get_header(&req, "token") {
        Some(token) => token,
        None => return HttpResponse::Forbidden().finish(),
    };
    if !crate::tokens::has_token(token.as_str()) {
        return HttpResponse::Forbidden().finish();
    }
    let mut server = GameServer {
        id:  "".to_string(),
        info: request.into_inner(),
        last_seen: "".to_string(),
        last_seen_sec: 0.0,
    };
    match server.save(&token.as_str()) {
        Ok(()) =>
            HttpResponse::Ok().json(UpdateResponse {
                id: server.id,
            }),
        Err(err) =>
            HttpResponse::InternalServerError().json(json!({
                "error": err,
            })),
    }
    
}

#[get("/v1/list")]
async fn list(query: web::Query<ListQuery>) -> impl Responder {
    let mut result = Vec::new();
    for entry in match crate::db::model::Server::find_by_filter(
        query.include_dev.unwrap_or(false),
        query.include_fallback.unwrap_or(false),
        query.exclude_full.unwrap_or(false)
    ) {
        Ok(x) => x,
        Err(e) => {
            return HttpResponse::InternalServerError().json(json!({
                "error": e,
            }));
        }
    } {
        result.push(match entry.try_into() {
            Ok(x) => x,
            Err(e) => {
                return HttpResponse::InternalServerError().json(json!({
                    "error": e,
                }));
            }
        });
    }

    HttpResponse::Ok().json(ListResponse(result))
}

#[get("/v1/info/{server_id}")]
async fn info(server_id: web::Path<String>) -> impl Responder {
    let id = match Uuid::parse_str(server_id.as_str()) {
        Ok(x) => x,
        Err(_) => {
            return HttpResponse::NotFound().finish();
        },
    };
    let info = match crate::db::model::Server::find_by_id(id) {
        Ok(x) => x,
        Err(_) => {
            return HttpResponse::NotFound().finish();
        },
    };
    let info: Result<GameServer, _> = info.try_into();
    match info {
        Ok(x) => HttpResponse::Ok().json(x),
        Err(e) => HttpResponse::InternalServerError().json(json!({
            "error": e,
        }))
    }
}

async fn find_server(game: &str, dev: bool, fallback: bool, ignore: &Vec<String>) -> Option<GameServer> {
    // search for entries
    for entry in match crate::db::model::Server
        ::find_by_filter(dev, fallback, true) 
    {
        Ok(x) => x,
        Err(_) => {
            return None;
        },
    }
    {
        let entry: GameServer = match entry.try_into() {
            Ok(x) => x,
            Err(_) => continue,
        };
        // check if server is ignored
        if let Err(_) = ignore.binary_search(&entry.id) {
            continue;
        }
        // check if entry flags matches filter
        if entry.info.developer != dev 
            || entry.info.fallback != fallback
            || entry.info.maintenance
            || entry.info.full 
        {
            continue;
        }
        // check if server is online
        if entry.last_seen_sec >= 60.0 {
            continue;
        }
        // check if entry has game supported
        for game_entry in &entry.info.games {
            if game_entry.name == game {
                return Some(entry);
            }
        }
    }
    None
}

async fn find_server_for_request(request: &NewRequest) -> Option<GameServer> {
    let game = request.game.as_str();
    let developer = request.developer.unwrap_or(false);
    let fallback = request.fallback.unwrap_or(true);
    let empty = vec![];
    let ignore = match &request.ignore {
        Some(x) => x,
        None => &empty,
    };
    if developer {
        if let Some(result) = find_server(game, true, false, ignore).await {
            return Some(result);
        }
        if !fallback {
            return None;
        }
        if let Some(result) = find_server(game, true, true, ignore).await {
            return Some(result);
        }
    }

    if let Some(result) = find_server(game, false, false, ignore).await {
        return Some(result);
    }
    if !fallback {
        return None;
    }
    find_server(game, false, true, ignore).await
}

async fn new(mut request: NewRequest) -> impl Responder {
    if let Some(mut ignore) = request.ignore {
        ignore.sort();
        request = NewRequest {
            developer: request.developer,
            fallback: request.fallback,
            game: request.game,
            ignore: Some(ignore),
        };
    }
    match find_server_for_request(&request).await {
        Some(result) => {
            let game_name = &request.game;
            HttpResponse::Ok().json(NewResponse {
                id: result.id,
                api_uri: result.info.uri,
                game_uri: result.info.games.iter()
                    .filter_map(|game| {
                        if &game.name == game_name {
                            Some(game.uri.clone())
                        } else {
                            None
                        }
                    })
                    .next()
                    .unwrap(),
            })
        },
        None => HttpResponse::NotFound().finish(),
    }
}

#[get("/v1/new")]
async fn new_get(query: web::Query<NewRequest>) -> impl Responder {
    new(query.0).await
}

#[post("/v1/new")]
async fn new_post(request: web::Json<NewRequest>) -> impl Responder {
    new(request.0).await
}

#[post("/v1/token")]
async fn token_post(req: web::HttpRequest, request: web::Json<FastTokenAddRequest>) -> impl Responder {
    let token = match get_header(&req, "token") {
        Some(token) => token,
        None => return HttpResponse::Forbidden().finish(),
    };
    let server = match crate::db::model::Server::find_by_token(&token.as_str()) {
        Ok(x) => x,
        Err(_) => return HttpResponse::Forbidden().finish(),
    };
    let result: Result<crate::db::model::FastToken, _> = (server.id, request.into_inner()).try_into();
    match result {
        Ok(res) => HttpResponse::Ok().json(Into::<FastTokenAddResponse>::into(res)),
        Err(e) =>
            HttpResponse::InternalServerError().json(json!({
                "error": e,
            })),
    }
}

#[get("/v1/token/{token}")]
async fn token_get(token: web::Path<String>) -> impl Responder {let now = chrono::Utc::now();
    let limit = now.checked_sub_signed(
        chrono::Duration::minutes(20)
    )
        .expect("limit traveled back in time")
        .naive_utc();
    let result = crate::db::model::FastToken::find_by_token_checked(
        &token.into_inner().to_uppercase(), 
        limit
    );
    match result {
        Ok(x) => {
            let x: Result<FastTokenFetchResponse, _> = x.try_into();
            match x {
                Ok(x) => HttpResponse::Ok().json(x),
                Err(e) => HttpResponse::InternalServerError().json(json!({
                    "error": e,
                })),
            }
        },
        Err(_) => HttpResponse::NotFound().finish(),
    }
}

pub fn init_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(redirect);
    cfg.service(index);
    cfg.service(index_yml);
    cfg.service(index_json);
    cfg.service(update);
    cfg.service(list);
    cfg.service(info);
    cfg.service(new_get);
    cfg.service(new_post);
    cfg.service(token_post);
    cfg.service(token_get);
}