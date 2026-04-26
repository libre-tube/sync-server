#[macro_use]
extern crate diesel;

use std::{io, sync::LazyLock};

use actix_web::{App, HttpServer, middleware, web};
use diesel_async::pooled_connection::{AsyncDieselConnectionManager, PoolError, bb8::Pool};
use diesel_migrations::{EmbeddedMigrations, MigrationHarness, embed_migrations};
use log::error;
use utoipa::OpenApi;
use utoipa_actix_web::AppExt;
use utoipa_scalar::{Scalar, Servable};

use crate::{
    handlers::{
        ScopedHandler, health::HealthHandler, playlist_bookmarks::PlaylistBookmarksHandler,
        playlists::PlaylistsHandler, subscriptions::SubscriptionsHandler, user::UserHandler,
    },
    openapi::ApiDoc,
};

mod auth;
mod config;
mod database;
mod dto;
mod handlers;
mod models;
mod openapi;
mod schema;
mod validation;
mod youtube;

static CONFIG: LazyLock<config::Config> = LazyLock::new(|| match config::build_config() {
    Ok(c) => c,
    Err(e) => {
        error!("Failed to configure server: {e}");
        std::process::exit(1);
    }
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
    env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));

    // initialize DB pool outside `HttpServer::new` so that it is shared across all workers
    let pool = match initialize_db_pool(&CONFIG.database_url).await {
        Ok(pool) => pool,
        Err(err) => panic!("{}", err),
    };

    // run database migrations (must be done BEFORE the server is started!)
    run_migrations(&pool).await;

    log::info!("starting HTTP server at http://localhost:8080");

    HttpServer::new(move || {
        let (app, generated_api) = App::new()
            .into_utoipa_app()
            // add DB pool handle to app data; enables use of `web::Data<DbPool>` extractor
            .app_data(web::Data::new(pool.clone()))
            .service(
                utoipa_actix_web::scope("/v1")
                    .service(UserHandler::get_service())
                    .service(SubscriptionsHandler::get_service())
                    .service(PlaylistsHandler::get_service())
                    .service(PlaylistBookmarksHandler::get_service()),
            )
            .split_for_parts();

        // add additional meta and security info
        let mut api = ApiDoc::openapi();
        api.merge(generated_api);

        // docs service must be registered before health handler!
        app.service(Scalar::with_url("/docs", api))
            .service(HealthHandler::get_service())
            .wrap(middleware::Logger::default())
    })
    .bind(("0.0.0.0", 8080))?
    .run()
    .await
}

/// Initialize database connection pool based on `DATABASE_URL` environment variable.
///
/// See more: <https://docs.rs/diesel-async/latest/diesel_async/pooled_connection/index.html#modules>.
async fn initialize_db_pool(db_url: &str) -> Result<DbPool, PoolError> {
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
