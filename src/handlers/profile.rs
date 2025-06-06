use super::*;
use crate::handlers::leave::get_pending_requests_count;
use crate::models::team::*;
use crate::templates::ProfileTemplate;
use argon2::{
    password_hash::{rand_core::OsRng, PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
    Argon2,
};
use axum::{extract::State, http::StatusCode, Form};
use rand::Rng;
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use tower_sessions::Session;

#[derive(Deserialize)]
pub struct ChangePasswordRequest {
    current_password: String,
    new_password: String,
    confirm_password: String,
    csrf_token: String,
}

#[derive(Serialize, sqlx::FromRow)]
pub struct ChangePasswordResponse {
    success: bool,
    error: Option<String>,
}

pub async fn profile(
    State(pool): State<PgPool>,
    auth: AuthSession,
    headers: axum::http::HeaderMap,
    session: Session,
) -> Result<impl IntoResponse, StatusCode> {
    let auth_clone = auth.clone();
    let user = auth.user.ok_or(StatusCode::UNAUTHORIZED)?;

    // Generate and store CSRF token
    let csrf_token: String = rand::thread_rng()
        .sample_iter(&rand::distributions::Alphanumeric)
        .take(32)
        .map(char::from)
        .collect();

    session
        .insert("csrf_token", &csrf_token)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    // Get team name
    let team_name = sqlx::query_scalar::<_, String>("SELECT name FROM teams WHERE id = $1")
        .bind(user.team_id)
        .fetch_optional(&pool)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .unwrap_or_else(|| "No Team".to_string());

    // Get supervisor name based on role
    let supervisor_name = match user.role {
        crate::models::user::Role::Admin => "HR Department".to_string(),
        crate::models::user::Role::Human_Resources => "No Supervisor".to_string(),
        crate::models::user::Role::Officer => sqlx::query_scalar::<_, String>(
            "SELECT u.name FROM users u 
                JOIN teams t ON u.id = t.team_lead_id 
                WHERE t.id = $1",
        )
        .bind(user.team_id)
        .fetch_optional(&pool)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .unwrap_or_else(|| "No Supervisor".to_string()),
        crate::models::user::Role::Team_Lead => "HR Department".to_string(),
    };

    let job_title = sqlx::query_scalar::<_, String>("SELECT job_title FROM users WHERE id = $1")
        .bind(user.id)
        .fetch_optional(&pool)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .unwrap_or_else(|| "No Job Title".to_string());

    let pending_count = get_pending_requests_count(State(pool.clone()), auth_clone)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let is_htmx_request = headers.contains_key("HX-Request");

    let html = if is_htmx_request {
        let template = ProfileTemplatePartial {
            user,
            team_name,
            supervisor_name,
            job_title,
            csrf_token,
        };
        template
            .render()
            .map_err(|_| axum::http::StatusCode::INTERNAL_SERVER_ERROR)?
    } else {
        let template = ProfileTemplate {
            user,
            team_name,
            supervisor_name,
            job_title,
            pending_count,
            csrf_token,
        };
        template
            .render()
            .map_err(|_| axum::http::StatusCode::INTERNAL_SERVER_ERROR)?
    };

    Ok(Html(html))
}

pub async fn change_password(
    State(pool): State<PgPool>,
    mut auth: AuthSession, // Changed to mutable
    session: Session,
    Form(payload): Form<ChangePasswordRequest>,
) -> Result<impl IntoResponse, StatusCode> {
    let user = auth.user.as_ref().ok_or(StatusCode::UNAUTHORIZED)?;

    // Verify CSRF token
    let stored_token: Option<String> = session
        .get("csrf_token")
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    if stored_token != Some(payload.csrf_token) {
        return Ok((
            StatusCode::OK,
            Html(r#"<div class="alert alert-danger">Invalid CSRF token</div>"#),
        ));
    }

    // Validate new password matches confirmation
    if payload.new_password != payload.confirm_password {
        return Ok((
            StatusCode::OK,
            Html(r#"<div class="alert alert-danger">New passwords do not match</div>"#),
        ));
    }

    // Validate new password strength
    if payload.new_password.len() < 8 {
        eprintln!("Password is too short!");
        return Ok((
            StatusCode::OK,
            Html(
                r#"<div class="alert alert-danger">Password must be at least 8 characters long</div>"#,
            ),
        ));
    }

    // Verify current password
    match verify_password(&payload.current_password, &user.password) {
        Ok(true) => (),
        Ok(false) => {
            return Ok((
                StatusCode::OK,
                Html(r#"<div class="alert alert-danger">Current password is incorrect</div>"#),
            ));
        }
        Err(_) => return Err(StatusCode::INTERNAL_SERVER_ERROR),
    }

    // Hash new password
    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();

    let new_password_hash = argon2
        .hash_password(payload.new_password.as_bytes(), &salt)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .to_string();

    // Update password in database
    sqlx::query("UPDATE users SET password_hash = $1 WHERE id = $2")
        .bind(&new_password_hash)
        .bind(user.id)
        .execute(&pool)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    // Invalidate all existing sessions by updating the auth hash
    auth.logout()
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    // Return success message
    Ok((
        StatusCode::OK,
        Html(
            r#"<div class="alert alert-success">Password changed successfully. Please log in again.</div>"#,
        ),
    ))
}
// pub async fn change_password(
//     State(pool): State<PgPool>,
//     mut auth: AuthSession,
//     session: Session,
//     Form(payload): Form<ChangePasswordRequest>,
// ) -> Result<impl IntoResponse, StatusCode> {
//     let user = auth.user.ok_or(StatusCode::UNAUTHORIZED)?;

//     // Verify CSRF token
//     let stored_token: Option<String> = session
//         .get("csrf_token")
//         .await
//         .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

//     if stored_token != Some(payload.csrf_token) {
//         return Ok(Html(
//             r#"<div class="alert alert-danger">Invalid CSRF token</div>"#,
//         ));
//     }

//     // Validate new password matches confirmation
//     if payload.new_password != payload.confirm_password {
//         return Ok(Html(
//             r#"<div class="alert alert-danger">New passwords do not match</div>"#,
//         ));
//     }

//     // Verify current password
//     let is_valid = verify_password(&payload.current_password, &user.password)
//         .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

//     if !is_valid {
//         return Ok(Html(
//             r#"<div class="alert alert-danger">Current password is incorrect</div>"#,
//         ));
//     }

//     // Hash new password
//     let salt = SaltString::generate(&mut OsRng);
//     let argon2 = Argon2::default();

//     let new_password_hash = argon2
//         .hash_password(payload.new_password.as_bytes(), &salt)
//         .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
//         .to_string();

//     // Update password in database
//     sqlx::query("UPDATE users SET password_hash = $1 WHERE id = $2")
//         .bind(&new_password_hash)
//         .bind(user.id)
//         .execute(&pool)
//         .await
//         .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

//     // Return success message
//     Ok(Html(
//         r#"<div class="alert alert-success">Password changed successfully</div>"#,
//     ))
// }

pub fn verify_password(password: &str, hash: &str) -> Result<bool, String> {
    let parsed_hash =
        PasswordHash::new(hash).map_err(|e| format!("Failed to parse password hash: {}", e))?;
    Ok(Argon2::default()
        .verify_password(password.as_bytes(), &parsed_hash)
        .is_ok())
}

pub async fn get_add_user_page(
    State(pool): State<PgPool>,
    auth: AuthSession,
    headers: axum::http::HeaderMap,
    session: Session,
) -> Result<impl IntoResponse, axum::http::StatusCode> {
    let auth_clone = auth.clone();
    let user = auth.user.ok_or(StatusCode::UNAUTHORIZED)?;
    if user.role != Role::Human_Resources {
        let response = if headers.contains_key("HX-Request") {
            // For HTMX requests
            Response::builder()
                .status(StatusCode::OK)
                .header("HX-Redirect", "/")
                .body(String::new())
                .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        } else {
            // For regular requests
            Response::builder()
                .status(StatusCode::SEE_OTHER)
                .header("Location", "/")
                .body(String::from("<html><body>Redirecting...</body></html>"))
                .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        };
        return Ok(response);
    }

    let teams = sqlx::query_as::<_, Team>("SELECT id, name, team_lead_id FROM teams")
        .fetch_all(&pool)
        .await
        .map_err(|_| axum::http::StatusCode::INTERNAL_SERVER_ERROR)?;

    // Generate and store CSRF token
    let csrf_token: String = rand::thread_rng()
        .sample_iter(&rand::distributions::Alphanumeric)
        .take(32)
        .map(char::from)
        .collect();

    session
        .insert("csrf_token", &csrf_token)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let is_htmx_request = headers.contains_key("HX-Request");
    let pending_count = get_pending_requests_count(State(pool.clone()), auth_clone)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let html = if is_htmx_request {
        let template = NewUserTemplatePartial { csrf_token, teams };
        template
            .render()
            .map_err(|_| axum::http::StatusCode::INTERNAL_SERVER_ERROR)?
    } else {
        let template = NewUserTemplate {
            role: user.role,
            pending_count,
            csrf_token,
            teams,
        };
        template
            .render()
            .map_err(|_| axum::http::StatusCode::INTERNAL_SERVER_ERROR)?
    };

    // Create response with HTML and HX-Trigger header
    let response = axum::response::Response::builder()
        .header("Content-Type", "text/html")
        .header("HX-Trigger", r#"initRequestsCalendar"#)
        .body(html)
        .map_err(|_| axum::http::StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(response)
}

pub async fn create(
    State(pool): State<PgPool>,
    auth: AuthSession,
    session: Session,
    Form(form): Form<UserForm>,
) -> Result<impl IntoResponse, axum::http::StatusCode> {
    // Check authentication and authorization
    let user = auth.user.ok_or(axum::http::StatusCode::UNAUTHORIZED)?;

    // Only allow Human_Resources to access this endpoint
    if user.role != Role::Human_Resources {
        return Ok((
            axum::http::StatusCode::FORBIDDEN,
            Html(r#"Only HR personnel can create new employees"#),
        ));
    }

    // Verify CSRF token
    let stored_token: Option<String> = session
        .get("csrf_token")
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    if stored_token != Some(form.csrf_token) {
        return Ok((
            axum::http::StatusCode::BAD_REQUEST,
            Html(r#"<div class="alert alert-danger">Invalid CSRF token</div>"#),
        ));
    }

    // Get team_id from the name
    let team_id = sqlx::query_scalar::<_, i32>("SELECT id FROM teams WHERE name = $1")
        .bind(&form.team_name)
        .fetch_one(&pool)
        .await
        .map_err(|e| {
            tracing::error!("Failed to fetch team_id: {:?}", e);

            // Return 404 if team doesn't exist, 500 for other errors
            if let sqlx::Error::RowNotFound = e {
                StatusCode::NOT_FOUND
            } else {
                StatusCode::INTERNAL_SERVER_ERROR
            }
        })?;

    println!("made it to salting the password");

    // Hash the password before storing
    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();

    let password_hash = argon2
        .hash_password(form.password.as_bytes(), &salt)
        .map_err(|_| axum::http::StatusCode::INTERNAL_SERVER_ERROR)?
        .to_string();

    // Insert the new employee into database
    let result = sqlx::query_scalar::<_, i32>(
        "
        INSERT INTO users (username, password_hash, role, name, job_title, team_id, email)
        VALUES ($1, $2, $3, $4, $5, $6, $7)
        RETURNING id
        ",
    )
    .bind(form.username)
    .bind(password_hash)
    .bind(form.role)
    .bind(form.name)
    .bind(form.job_title)
    .bind(team_id)
    .bind(form.email)
    .fetch_one(&pool)
    .await;

    match result {
        Ok(_record) => {
            // Success response
            Ok((
                axum::http::StatusCode::OK,
                Html(r#"User created successfully"#),
            ))
        }
        Err(e) => {
            // Handle unique constraint violations
            tracing::error!("Failed to create new user: {:?}", e);

            if let sqlx::Error::Database(db_err) = &e {
                if db_err.constraint() == Some("users_username_key") {
                    return Ok((
                        axum::http::StatusCode::CONFLICT,
                        Html(r#"Username already exists"#),
                    ));
                }
            }
            // Generic error response
            Err(axum::http::StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}
