#[macro_use]
extern crate diesel;

use std::{env, io};

use actix_web::{App, HttpServer, middleware, web};
use diesel::SqliteConnection;
use diesel_async::{
    pooled_connection::{AsyncDieselConnectionManager, bb8::Pool},
    sync_connection_wrapper::SyncConnectionWrapper,
};
use dotenvor::dotenv;
use utoipa_actix_web::AppExt;
use utoipa_scalar::{Scalar, Servable};

use crate::handlers::{ScopedHandler, user::UserHandler};

mod database;
mod dto;
mod handlers;
mod models;
mod schema;
mod util;

type DbConnection = SyncConnectionWrapper<SqliteConnection>;
type DbPool = Pool<DbConnection>;
type WebData = web::Data<DbPool>;

#[actix_web::main]
async fn main() -> io::Result<()> {
    unsafe { dotenv() }.ok();
    env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));

    // initialize DB pool outside `HttpServer::new` so that it is shared across all workers
    let pool = initialize_db_pool().await;

    log::info!("starting HTTP server at http://localhost:8080");

    HttpServer::new(move || {
        let (app, api) = App::new()
            .into_utoipa_app()
            // add DB pool handle to app data; enables use of `web::Data<DbPool>` extractor
            .app_data(web::Data::new(pool.clone()))
            .service(UserHandler::get_service())
            .split_for_parts();

        app.service(Scalar::with_url("/docs", api))
            .wrap(middleware::Logger::default())
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}

/// Initialize database connection pool based on `DATABASE_URL` environment variable.
///
/// See more: <https://docs.rs/diesel-async/latest/diesel_async/pooled_connection/index.html#modules>.
async fn initialize_db_pool() -> DbPool {
    let db_url = env::var("DATABASE_URL").expect("Env var `DATABASE_URL` not set");

    let connection_manager = AsyncDieselConnectionManager::<DbConnection>::new(db_url);
    Pool::builder().build(connection_manager).await.unwrap()
}
