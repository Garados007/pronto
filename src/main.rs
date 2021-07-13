#[macro_use]
extern crate log;
#[macro_use]
extern crate diesel;
#[macro_use]
extern crate diesel_migrations;

use actix_web::{App, HttpServer };
use listenfd::ListenFd;
use std::env;

mod api_error;
mod db;
mod schema;
mod v1;
mod tokens;

#[actix_rt::main]
async fn main() -> std::io::Result<()> {
    println!("init pronto server ...");
    dotenv::dotenv()
        .ok();
    env_logger::init();
    db::init();
    tokens::init();

    let mut listenfd = ListenFd::from_env();
    let mut server = HttpServer::new(|| {
        let cors = actix_cors::Cors::default()
            .allow_any_origin()
            .allow_any_method()
            .allow_any_header()
            .max_age(3600);

        App::new()
            .wrap(cors)
            .configure(v1::init_routes)
    });
    
    server = match listenfd.take_tcp_listener(0)? {
        Some(listener) => server.listen(listener)?,
        None => {
            let host = env::var("HOST").expect("HOST not set");
            let port = env::var("PORT").expect("PORT not set");

            server.bind(format!("{}:{}", host, port))?
        },
    };

    info!("starting server");
    server.run().await
}
