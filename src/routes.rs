use axum::{
    routing::{delete, get, post, put},
    Router,
};
use axum_login::login_required;
use sqlx::postgres::PgPool;
use tower_http::services::ServeDir;

use crate::handlers;
use crate::models::user::Backend;

pub fn create_router(pool: PgPool) -> Router {
    Router::new()
        .route("/", get(handlers::leave::index))
        //.route("/dashboard", get(handlers::leave::dashboard))
        .route("/profile", get(handlers::profile::profile))
        .route("/change-password", post(handlers::profile::change_password))
        .route("/add_user", post(handlers::profile::create))
        .route(
            "/notifications",
            get(handlers::notification::get_notifications),
        )
        .route(
            "/notifications/:id/mark-read",
            put(handlers::notification::mark_notification_read),
        )
        .route("/new_leave", get(handlers::leave::request_leave))
        .route("/calendar", get(handlers::leave::calendar))
        .route("/api/calendar", get(handlers::leave::calendar_api))
        .route("/requests", get(handlers::leave::requests))
        .route("/requests/pending", get(handlers::leave::pending_requests))
        .route(
            "/requests/approved",
            get(handlers::leave::approved_requests),
        )
        .route(
            "/requests/rejected",
            get(handlers::leave::rejected_requests),
        )
        .route("/new_user", get(handlers::profile::get_add_user_page))
        .route("/submit_leave", post(handlers::leave::submit_leave))
        .route("/leaveslist", get(handlers::leave::my_leaves_list))
        .route("/my_leaves_prev", get(handlers::leave::my_leaves_prev))
        .route("/team_leaves_prev", get(handlers::leave::team_leaves_prev))
        .route(
            "/leave/:id",
            delete(handlers::leave::delete_leave)
                .get(handlers::leave::get_leave)
                .put(handlers::leave::update_leave),
        )
        .route("/leave/:id/edit", get(handlers::leave::edit_leave))
        .route(
            "/approve_request/:id",
            post(handlers::leave::approve_request),
        )
        .route("/reject_request/:id", post(handlers::leave::reject_request))
        .route_layer(login_required!(Backend, login_url = "/authorize"))
        .route("/authorize", get(handlers::auth::login_form))
        .route("/login", post(handlers::auth::login))
        .route("/logout", get(handlers::auth::logout))
        .nest_service("/assets", ServeDir::new("assets"))
        .with_state(pool)
}
