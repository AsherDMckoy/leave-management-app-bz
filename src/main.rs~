use axum_login::{
    tower_sessions::{ExpiredDeletion, Expiry, SessionManagerLayer},
    AuthManagerLayerBuilder,
};
use axum_messages::MessagesManagerLayer;
use sqlx::postgres::PgPool;
use std::error::Error;
use time::Duration;
use tokio::{signal, task::AbortHandle};
use tower_sessions::cookie::Key;
use tower_sessions_sqlx_store::PostgresStore;
use tracing::info;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
//pub use::error::{Error, Result};
//mod error;

mod db;
mod handlers;
mod models;
mod routes;
mod templates;
use crate::models::user::Backend;
use crate::routes::create_router;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    //    //
    dotenv::dotenv().ok();
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "hrm_app_axum_htmx=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    let conn_url =
        std::env::var("DATABASE_URL").expect("Env var DATABASE_URL is required for this example.");
    let pool = PgPool::connect(&conn_url).await?;
    // let mut tx = pool.begin().await?;
    // db::load_requests(&mut tx).await?;
    // tx.commit().await?;
    let session_store = PostgresStore::new(pool.clone());

    let deletion_task = tokio::task::spawn(
        session_store
            .clone()
            .continuously_delete_expired(tokio::time::Duration::from_secs(60)),
    );

    let key = Key::generate();

    let session_layer = SessionManagerLayer::new(session_store)
        .with_secure(false)
        .with_expiry(Expiry::OnInactivity(Duration::days(1)))
        .with_signed(key);

    // Auth service.
    //
    // This combines the session layer with our backend to establish the auth
    // service which will provide the auth session as a request extension.
    let backend = Backend::new(pool.clone());
    let auth_layer = AuthManagerLayerBuilder::new(backend, session_layer).build();

    info!("Initializing Server...");
    let app = create_router(pool)
        .layer(MessagesManagerLayer)
        .layer(auth_layer)
        .fallback(handlers::fallback::fallback);

    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000")
        .await
        .unwrap();
    println!("Listening on 127.0.0.1:3000");
    tracing::debug!("listening on {}", listener.local_addr().unwrap());
    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal(deletion_task.abort_handle()))
        .await
        .unwrap();
    Ok(())
}

async fn shutdown_signal(deletion_task_abort_handle: AbortHandle) {
    let ctrl_c = async {
        signal::ctrl_c()
            .await
            .expect("failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("failed to install signal handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => { deletion_task_abort_handle.abort() },
        _ = terminate => { deletion_task_abort_handle.abort() },
    }
}
