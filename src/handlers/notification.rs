use super::*;
use axum::{extract::State, response::Html};
use sqlx::PgPool;

pub async fn create_notification(
    pool: &PgPool,
    recipient_id: i32,
    leave_request: i32,
    message: String,
) -> Result<(), sqlx::Error> {
    let query = "INSERT INTO notifications (recipient_id, leave_request_id, message, is_read) VALUES($1, $2, $3, false)";
    let _request = sqlx::query(query)
        .bind(recipient_id)
        .bind(leave_request)
        .bind(message)
        .execute(pool)
        .await
        .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR));

    Ok(())
}

pub async fn get_notifications(
    State(pool): State<PgPool>,
    auth: AuthSession,
) -> Result<Html<String>, StatusCode> {
    let user = auth.user.ok_or(StatusCode::UNAUTHORIZED)?;
    
    let query = "SELECT n.id, n.recipient_id, n.message, n.created_at, n.is_read, n.leave_request_id 
                FROM notifications n 
                WHERE n.recipient_id = $1 
                AND n.is_read = false
                ORDER BY n.created_at DESC";

    let notifications = sqlx::query_as::<_, Notification>(query)
        .bind(user.id)
        .fetch_all(&pool)
        .await
        .map_err(|e| {
            eprintln!("Error fetching notifications: {:?}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    let template = NotificationsTemplate {
        notifications,
    };

    let rendered = template.render().map_err(|e| {
        eprintln!("Error rendering notifications: {:?}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    Ok(Html(rendered))
}

pub async fn mark_notification_read(
    State(pool): State<PgPool>,
    Path(notification_id): Path<i32>,
) -> impl IntoResponse {
    let query = "UPDATE notifications SET is_read = true WHERE id = $1";
    let _result = sqlx::query(query)
        .bind(notification_id)
        .execute(&pool)
        .await
        .unwrap_or_default();

    Html("")
}
