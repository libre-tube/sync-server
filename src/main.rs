#[macro_use]
extern crate diesel;

use std::{env, io, sync::LazyLock};

use actix_web::{App, HttpServer, middleware, web};
use diesel_async::pooled_connection::{AsyncDieselConnectionManager, PoolError, bb8::Pool};
use diesel_migrations::{EmbeddedMigrations, MigrationHarness, embed_migrations};
use dotenvor::dotenv;
use utoipa::openapi::LicenseBuilder;
use utoipa_actix_web::AppExt;
use utoipa_scalar::{Scalar, Servable};

use crate::handlers::{
    ScopedHandler, health::HealthHandler, playlists::PlaylistsHandler,
    subscriptions::SubscriptionsHandler, user::UserHandler,
};

mod auth;
mod database;
mod dto;
mod handlers;
mod models;
mod schema;

static SECRET_KEY: LazyLock<String> = LazyLock::new(|| {
    env::var("SECRET_KEY").expect("Please set the `SECRET_KEY` env variable to a random value!")
});

pub const MIGRATIONS: EmbeddedMigrations = embed_migrations!("./migrations/");

#[cfg(all(feature = "sqlite", feature = "postgres"))]
compile_error!("Sqlite and Postgres are mutually exclusive and cannot be enabled together");

#[cfg(feature = "sqlite")]
type DbConnection =
    diesel_async::sync_connection_wrapper::SyncConnectionWrapper<diesel::SqliteConnection>;
#[cfg(feature = "postgres")]
type DbConnection = diesel_async::AsyncPgConnection;

type DbPool = Pool<DbConnection>;
type WebData = web::Data<DbPool>;

#[actix_web::main]
async fn main() -> io::Result<()> {
    // load env from .env file
    unsafe { dotenv() }.ok();

    env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));

    // initialize DB pool outside `HttpServer::new` so that it is shared across all workers
    let pool = match initialize_db_pool().await {
        Ok(pool) => pool,
        Err(err) => panic!("{}", err),
    };

    // run database migrations (must be done BEFORE the server is started!)
    run_migrations(&pool).await;

    log::info!("starting HTTP server at http://localhost:8080");

    HttpServer::new(move || {
        let (app, mut api) = App::new()
            .into_utoipa_app()
            // add DB pool handle to app data; enables use of `web::Data<DbPool>` extractor
            .app_data(web::Data::new(pool.clone()))
            .service(HealthHandler::get_service())
            .service(UserHandler::get_service())
            .service(SubscriptionsHandler::get_service())
            .service(PlaylistsHandler::get_service())
            .split_for_parts();

        // update displayed metadata in OpenAPI docs
        api.info.title = String::from(env!("CARGO_PKG_NAME"));
        api.info.version = String::from(env!("CARGO_PKG_VERSION"));
        api.info.description = Some(String::from(env!("CARGO_PKG_DESCRIPTION")));
        api.info.license = Some(
            LicenseBuilder::new()
                .identifier(Some(env!("CARGO_PKG_LICENSE")))
                .name(env!("CARGO_PKG_LICENSE"))
                .build(),
        );
        api.info.contact = None;

        app.service(Scalar::with_url("/docs", api))
            .wrap(middleware::Logger::default())
    })
    .bind(("0.0.0.0", 8080))?
    .run()
    .await
}

/// Initialize database connection pool based on `DATABASE_URL` environment variable.
///
/// See more: <https://docs.rs/diesel-async/latest/diesel_async/pooled_connection/index.html#modules>.
async fn initialize_db_pool() -> Result<DbPool, PoolError> {
    let db_url = env::var("DATABASE_URL").expect("Env var `DATABASE_URL` not set");

    let connection_manager = AsyncDieselConnectionManager::<DbConnection>::new(db_url);
    Pool::builder().build(connection_manager).await
}

async fn run_migrations(pool: &DbPool) {
    // https://github.com/diesel-rs/diesel_async/discussions/268
    let conn = pool.get_owned().await.unwrap();

    #[cfg(feature = "sqlite")]
    {
        let mut conn = conn;
        conn.spawn_blocking(|conn| {
            // we panic if migrations fail, because otherwise the app wouldn't work anyways
            conn.run_pending_migrations(MIGRATIONS).unwrap();
            Ok(())
        })
        .await
        .unwrap();
    }

    #[cfg(feature = "postgres")]
    {
        // must be spawned blocking, otherwise this would raise 'can call blocking only when running on the multi-threaded runtime': see https://github.com/rwf2/Rocket/pull/2648
        actix_web::rt::task::spawn_blocking(move || {
            let mut harness = diesel_async::AsyncMigrationHarness::new(conn);
            harness.run_pending_migrations(MIGRATIONS).unwrap();
        })
        .await
        .unwrap();
    }
}
