use super::*;
use notification::create_notification;
// pub const HYPATIA_ID: i32 = 3;

pub async fn index(
    State(pool): State<PgPool>,
    auth: AuthSession,
    headers: axum::http::HeaderMap,
) -> impl IntoResponse {
    let auth_clone = auth.clone();
    let result = auth.user.ok_or(StatusCode::UNAUTHORIZED);
    let (available_leaves, used_leaves, overdue_leaves): (i32, i32, i32) =
        match sqlx::query_as::<_, (i32, i32, i32)>(
            "SELECT available_leaves, used_leaves, overdue_leaves FROM users WHERE id = $1",
        )
        .bind(result.clone().unwrap().id)
        .fetch_one(&pool)
        .await
        {
            Ok(data) => data,
            Err(_) => return StatusCode::INTERNAL_SERVER_ERROR.into_response(),
        };
    let pending_count = match get_pending_requests_count(State(pool.clone()), auth_clone).await {
        Ok(count) => count,
        Err(_) => return StatusCode::INTERNAL_SERVER_ERROR.into_response(),
    };

    match result {
        Ok(user) => {
            let is_htmx_response = headers.contains_key("HX-Request");
            let mut response = if is_htmx_response {
                let template = DashboardTemplate {
                    username: user.name,
                    available_leaves,
                    used_leaves,
                    overdue_leaves,
                };
                HtmlTemplate(template).into_response()
            } else {
                let template = IndexTemplate {
                    username: user.name,
                    role: user.role,
                    pending_count,
                    available_leaves,
                    used_leaves,
                    overdue_leaves,
                };
                HtmlTemplate(template).into_response()
            };
            let headers = response.headers_mut();
            headers.insert("HX-Trigger", "initCalendar".parse().unwrap());
            response
        }
        Err(status) => status.into_response(),
    }
}

/*pub async fn dashboard(State(pool): State<PgPool>, auth: AuthSession) -> impl IntoResponse {
    let result = auth.user.ok_or(StatusCode::UNAUTHORIZED);
    match result {
        Ok(user) => {
            let template = DashboardTemplate {
                username: user.username,
            };
            let mut response = HtmlTemplate(template).into_response();

            let headers = response.headers_mut();
            headers.insert("HX-Redirect", "/".parse().unwrap());
            response
        }
        Err(status) => status.into_response(),
    }
}*/

pub async fn request_leave() -> impl IntoResponse {
    HtmlTemplate(NewLeaveTemplate)
}

pub async fn my_leaves_prev(
    State(pool): State<PgPool>,
    auth: AuthSession,
) -> Result<Html<String>, axum::http::StatusCode> {
    let user = auth.user.ok_or(StatusCode::UNAUTHORIZED)?;
    let leaves_prev = sqlx::query_as::<_, MyLeavePreview>("SELECT start_date, end_date, days, leave_type, status FROM leave_requests WHERE user_id = $1 ORDER BY created_at DESC LIMIT 8")
        .bind(user.id)
        .fetch_all(&pool)
        .await
        .map_err(|e| {
            println!("Database error: {:?}", e);
            axum::http::StatusCode::INTERNAL_SERVER_ERROR
        })?;

    let template = MyLeavesPrevTemplate {
        leaves: leaves_prev,
    };
    let html = template.render().map_err(|e| {
        println!("Template rendering error: {:?}", e);
        axum::http::StatusCode::INTERNAL_SERVER_ERROR
    })?;

    Ok(Html(html))
}

pub async fn team_leaves_prev(
    State(pool): State<PgPool>,
    auth: AuthSession,
    Query(params): Query<HashMap<String, i32>>,
) -> Result<Html<String>, axum::http::StatusCode> {
    let user = auth.user.ok_or(StatusCode::UNAUTHORIZED)?;
    let limit = params.get("q").unwrap_or(&10);
    let leaves_prev = sqlx::query_as::<_, TeamLeavePreview>("SELECT users.name,  leave_requests.start_date,  leave_requests.end_date,
CASE
    WHEN CURRENT_DATE BETWEEN leave_requests.start_date AND leave_requests.end_date THEN
        leave_requests.end_date - CURRENT_DATE + 1
    WHEN leave_requests.start_date > CURRENT_DATE THEN
        leave_requests.start_date - CURRENT_DATE + 1
END as days,
CASE
    WHEN CURRENT_DATE BETWEEN leave_requests.start_date AND leave_requests.end_date THEN 'On leave'
    WHEN leave_requests.start_date > CURRENT_DATE THEN 'Planned'
END as current_status
FROM leave_requests JOIN users ON users.id = leave_requests.user_id JOIN teams ON users.team_id = teams.id WHERE leave_requests.status = 'approved' AND ((CURRENT_DATE BETWEEN start_date AND end_date) OR (start_date > CURRENT_DATE)) AND team_id = $1 AND users.id != $2 ORDER BY leave_requests.created_at DESC LIMIT $3")
        .bind(user.team_id)
        .bind(user.id)
        .bind(limit)
        .fetch_all(&pool)
        .await
        .map_err(|_| axum::http::StatusCode::INTERNAL_SERVER_ERROR)?;

    let template = TeamLeavesPrevTemplate {
        leaves: leaves_prev,
    };
    let html = template
        .render()
        .map_err(|_| axum::http::StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Html(html))
}

fn count_weekdays_in_current_year(start_date: NaiveDate, end_date: NaiveDate) -> i64 {
    let current_year = chrono::Utc::now().year();
    let mut count = 0;

    // Determine the actual range to iterate over (only current year dates)
    let year_start = NaiveDate::from_ymd_opt(current_year, 1, 1).unwrap();
    let year_end = NaiveDate::from_ymd_opt(current_year, 12, 31).unwrap();

    let actual_start = start_date.max(year_start);
    let actual_end = end_date.min(year_end);

    // If the range doesn't overlap with current year, return 0
    if actual_start > actual_end {
        return 0;
    }

    let mut current_date = actual_start;

    while current_date <= actual_end {
        // Check if it's not a weekend
        match current_date.weekday() {
            Weekday::Sat | Weekday::Sun => {
                // Skip weekends
            }
            _ => {
                // It's a weekday, count it
                count += 1;
            }
        }

        current_date += Duration::days(1);
    }

    count
}
pub async fn submit_leave(
    State(pool): State<PgPool>,
    auth: AuthSession,
    Form(form): Form<LeaveRequestInput>,
) -> Result<Response<Body>, StatusCode> {
    let user = auth.user.ok_or(StatusCode::UNAUTHORIZED)?;

    // Validation: Check if user has enough available leaves
    let available_leaves: i32 =
        sqlx::query_scalar("SELECT available_leaves FROM users WHERE id = $1")
            .bind(user.id)
            .fetch_one(&pool)
            .await
            .map_err(|e| {
                eprintln!("Error fetching available leaves: {:?}", e);
                StatusCode::INTERNAL_SERVER_ERROR
            })?;

    if available_leaves < form.days {
        eprintln!(
            "Insufficient available leaves: requested {} days, available {} days",
            form.days, available_leaves
        );
        return Err(StatusCode::BAD_REQUEST);
    }

    let current_year_days = count_weekdays_in_current_year(form.start_date, form.end_date);
    let _update_available_leaves_result = sqlx::query(
        "UPDATE users SET available_leaves = (available_leaves - $1) WHERE users.id = $2",
    )
    .bind(form.days)
    .bind(user.id)
    .execute(&pool)
    .await
    .map_err(|e| {
        eprintln!("Error updating the number of available leaves: {:?}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    let _update_overdue_leaves_result =
        sqlx::query("UPDATE users SET overdue_leaves = (overdue_leaves - $1) WHERE users.id = $2 AND overdue_leaves > 0")
            .bind(current_year_days)
            .bind(user.id)
            .execute(&pool)
            .await
            .map_err(|e| {
                eprintln!("Error submitting leave request: {:?}", e);
                StatusCode::INTERNAL_SERVER_ERROR
            })?;

    let query = "INSERT INTO leave_requests (user_id, start_date, end_date, comments, leave_type, days) VALUES ($1, $2, $3, $4, $5, $6) RETURNING id";

    let request_id = sqlx::query_scalar::<_, i32>(query)
        .bind(user.id)
        .bind(form.start_date)
        .bind(form.end_date)
        .bind(form.comments)
        .bind(form.leave_type)
        .bind(form.days)
        .fetch_one(&pool)
        .await
        .map_err(|e| {
            eprintln!("Error submitting leave request: {:?}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    let team_query = if user.role == Role::Admin {
        "SELECT users.id FROM users WHERE role = 'human_resources'"
    } else {
        "SELECT team_lead_id FROM teams WHERE teams.id = $1"
    };

    let team_lead_id = sqlx::query_scalar::<_, i32>(team_query)
        .bind(user.team_id)
        .fetch_one(&pool)
        .await
        .map_err(|e| {
            eprintln!("Error fetching team lead ID: {:?}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    create_notification(
        &pool,
        team_lead_id,
        request_id,
        format!("New leave request from {}", user.name),
    )
    .await
    .map_err(|e| {
        eprintln!("Error creating notification: {:?}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    let mut headers = HeaderMap::new();
    headers.insert("HX-Refresh", "true".parse().unwrap());
    Ok((StatusCode::OK, headers).into_response())
}

pub async fn delete_leave(
    Path(id): Path<i32>,
    Query(params): Query<HashMap<String, i32>>,
    State(pool): State<PgPool>,
    auth: AuthSession,
) -> impl IntoResponse {
    //delete leave code here
    let user = auth.user.ok_or(StatusCode::UNAUTHORIZED).unwrap();
    let days = params.get("days").unwrap();
    println!("Number of days to be added : {}", &days);

    let _update_avaialble_leaves_result =
        sqlx::query("UPDATE users SET available_leaves = (available_leaves + $1) FROM leave_requests WHERE users.id = leave_requests.user_id AND users.id = $2 AND leave_requests.id = $3 AND leave_requests.start_date >= CURRENT_DATE AND leave_requests.status != 'rejected'")
            .bind(days)
            .bind(user.id)
            .bind(id)
            .execute(&pool)
            .await
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Error: {}", e)));

    let start_date =
        sqlx::query_scalar::<_, NaiveDate>("SELECT start_date FROM leave_requests where id = $1")
            .bind(id)
            .fetch_one(&pool)
            .await
            .map_err(|e| {
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    format!("Error fetching the start date of the leave request: {}", e),
                )
            });
    let end_date =
        sqlx::query_scalar::<_, NaiveDate>("SELECT end_date FROM leave_requests where id = $1")
            .bind(id)
            .fetch_one(&pool)
            .await
            .map_err(|e| {
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    format!("Error fetching the end date of the leave request: {}", e),
                )
            });

    let current_year_days = count_weekdays_in_current_year(start_date.unwrap(), end_date.unwrap());

    let _update_used_leaves_result =
        sqlx::query("UPDATE users SET used_leaves = (used_leaves - $1) WHERE users.id = $2")
            .bind(current_year_days)
            .bind(user.id)
            .execute(&pool)
            .await
            .map_err(|e| {
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    format!("Error updating the used leaves: {}", e),
                )
            });

    let _update_overdue_leaves_result =
        sqlx::query("UPDATE users SET overdue_leaves = (overdue_leaves + $1) WHERE users.id = $2")
            .bind(current_year_days)
            .bind(user.id)
            .execute(&pool)
            .await
            .map_err(|e| {
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    format!("Error updating the used leaves: {}", e),
                )
            });

    let query = "DELETE FROM leave_requests WHERE id = $1 AND user_id = $2";
    let result = sqlx::query(query)
        .bind(id)
        .bind(user.id)
        .execute(&pool)
        .await;

    match result {
        Ok(result) => {
            if result.rows_affected() == 0 {
                // No rows were deleted, meaning the leave request wasn't found
                (StatusCode::NOT_FOUND, "Leave request not found").into_response()
            } else {
                // Successfully deleted
                (StatusCode::OK, "").into_response()
            }
        }
        Err(e) => {
            // Log the error here
            eprintln!("Error deleting leave request: {:?}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Error deleting leave request",
            )
                .into_response()
        }
    }
}

pub async fn edit_leave(
    Path(id): Path<i32>,
    State(pool): State<PgPool>,
) -> Result<Html<String>, axum::http::StatusCode> {
    //edit leave code here
    let leave =
        sqlx::query_as::<_, LeaveRequestOutput>("SELECT id, start_date, end_date, days, leave_type, comments, status FROM leave_requests WHERE id = $1")
            .bind(id)
            .fetch_one(&pool)
            .await
            .map_err(|_| axum::http::StatusCode::NOT_FOUND)?;

    let template = EditLeaveTemplate { leave };
    Ok(Html(template.render().unwrap()))
}

pub async fn update_leave(
    Path(id): Path<i32>,
    State(pool): State<PgPool>,
    Form(form): Form<LeaveRequestInput>,
) -> Result<Html<String>, axum::http::StatusCode> {
    let start_date = form.start_date; // NaiveDate::parse_from_str(&form.start_date, "%Y-%m-%d").unwrap();
    let end_date = form.end_date; //NaiveDate::parse_from_str(&form.end_date, "%Y-%m-%d").unwrap();
    let leave_type = form.leave_type;
    let updated_leave = sqlx::query_as::<_,LeaveRequestOutput>(
        "UPDATE leave_requests SET start_date = $1, end_date = $2, leave_type = $3, comments = $4 WHERE id = $5 RETURNING id, start_date, end_date, days, leave_type, comments, status",
    )
    .bind(start_date)
    .bind(end_date)
    .bind(&leave_type)
    .bind(form.comments)
    .bind(id)
    .fetch_one(&pool)
    .await
        .map_err(|_| axum::http::StatusCode::INTERNAL_SERVER_ERROR)?;

    let template = LeaveRowTemplate {
        leave: updated_leave,
    };
    Ok(Html(template.render().unwrap()))
}

pub async fn get_leave(
    Path(id): Path<i32>,
    State(pool): State<PgPool>,
) -> Result<Html<String>, axum::http::StatusCode> {
    //edit leave code here
    let leave =
        sqlx::query_as::<_, LeaveRequestOutput>("SELECT id, start_date, end_date, days, leave_type, comments, status FROM leave_requests WHERE id = $1")
            .bind(id)
            .fetch_one(&pool)
            .await
            .map_err(|_| axum::http::StatusCode::NOT_FOUND)?;

    let template = LeaveRowTemplate { leave };
    Ok(Html(template.render().unwrap()))
}

async fn get_number_of_requests(
    State(pool): State<PgPool>,
    auth: AuthSession,
) -> Result<Vec<i64>, StatusCode> {
    // Changed to i64 which is compatible with SQL's bigint
    let user = auth.user.ok_or(StatusCode::UNAUTHORIZED)?;

    if user.role == Role::Human_Resources {
        // For HR, count requests from admin users
        let pending = sqlx::query_scalar::<_, i64>(
            "SELECT COUNT(leave_requests.id) FROM leave_requests JOIN users ON users.id = leave_requests.user_id WHERE users.role = 'admin' AND leave_requests.status = 'pending'"
        )
        .fetch_one(&pool)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

        let approved = sqlx::query_scalar::<_, i64>(
            "SELECT COUNT(leave_requests.id) FROM leave_requests JOIN users ON users.id = leave_requests.user_id WHERE users.role = 'admin' AND leave_requests.status = 'approved'"
        )
        .fetch_one(&pool)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

        let rejected = sqlx::query_scalar::<_, i64>(
            "SELECT COUNT(leave_requests.id) FROM leave_requests JOIN users ON users.id = leave_requests.user_id WHERE users.role = 'admin' AND leave_requests.status = 'rejected'"
        )
        .fetch_one(&pool)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

        Ok(vec![pending, approved, rejected])
    } else {
        // Similar changes for team leader queries...
        let pending = sqlx::query_scalar::<_, i64>(
            "SELECT COUNT(leave_requests.id) FROM leave_requests JOIN users ON users.id = leave_requests.user_id WHERE users.team_id = $1 AND leave_requests.status = 'pending' AND users.id != $2"
        )
        .bind(user.team_id)
        .bind(user.id)
        .fetch_one(&pool)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

        let approved = sqlx::query_scalar::<_, i64>(
            "SELECT COUNT(leave_requests.id) FROM leave_requests JOIN users ON users.id = leave_requests.user_id WHERE users.team_id = $1 AND leave_requests.status = 'approved' AND users.id != $2"
        )
        .bind(user.team_id)
        .bind(user.id)
        .fetch_one(&pool)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

        let rejected = sqlx::query_scalar::<_, i64>(
            "SELECT COUNT(leave_requests.id) FROM leave_requests JOIN users ON users.id = leave_requests.user_id WHERE users.team_id = $1 AND leave_requests.status = 'rejected' AND users.id != $2"
        )
        .bind(user.team_id)
        .bind(user.id)
        .fetch_one(&pool)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

        Ok(vec![pending, approved, rejected])
    }
}

pub async fn requests(
    State(pool): State<PgPool>,
    auth: AuthSession,
    headers: axum::http::HeaderMap,
) -> Result<impl IntoResponse, axum::http::StatusCode> {
    let auth_clone = auth.clone();
    let user = auth.user.ok_or(StatusCode::UNAUTHORIZED)?;
    // Check if user has required role
    if user.role != Role::Admin && user.role != Role::Human_Resources {
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
    let requests = if user.role == Role::Human_Resources {
        sqlx::query_as::<_, Request>(
            "SELECT leave_requests.id, users.name, users.job_title, start_date, end_date, days, leave_type, comments, leave_requests.created_at::date AS apply_date FROM leave_requests JOIN users ON users.id = leave_requests.user_id WHERE (users.role = 'admin' OR users.role = 'human_resources') AND leave_requests.status = 'pending' AND users.id != $1",)
            .bind(user.id)
            .fetch_all(&pool)
            .await.map_err(|_| axum::http::StatusCode::INTERNAL_SERVER_ERROR)?
    } else {
        sqlx::query_as::<_, Request>(
        "SELECT leave_requests.id, users.name, users.job_title, start_date, end_date, days, leave_type, comments, leave_requests.created_at::date AS apply_date FROM leave_requests JOIN users ON users.id = leave_requests.user_id WHERE users.team_id = $1 AND users.role = 'officer' AND leave_requests.status = 'pending' AND users.id != $2",)
        .bind(user.team_id).bind(user.id).fetch_all(&pool)
        .await.map_err(|_| axum::http::StatusCode::INTERNAL_SERVER_ERROR)?
    };

    let is_htmx_request = headers.contains_key("HX-Request");
    let counts = get_number_of_requests(State(pool.clone()), auth_clone)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    // Extract counts from vector
    let pending_count = counts[0];
    let approved_count = counts[1];
    let rejected_count = counts[2];

    let html = if is_htmx_request {
        let template = RequestsTemplatePartial {
            requests,
            pending_count,
            approved_count,
            rejected_count,
        };
        template
            .render()
            .map_err(|_| axum::http::StatusCode::INTERNAL_SERVER_ERROR)?
    } else {
        let template = RequestsTemplate {
            role: user.role,
            requests,
            pending_count,
            approved_count,
            rejected_count,
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

pub async fn pending_requests(
    State(pool): State<PgPool>,
    auth: AuthSession,
) -> Result<Html<String>, axum::http::StatusCode> {
    let user = auth.user.ok_or(StatusCode::UNAUTHORIZED)?;
    let requests = sqlx::query_as::<_, Request>(
        "SELECT leave_requests.id, users.name, users.job_title, start_date, end_date, days, leave_type, comments, leave_requests.created_at::date AS apply_date FROM leave_requests JOIN users ON users.id = leave_requests.user_id WHERE users.team_id = $1 AND leave_requests.status = 'pending' AND users.id != $2",)
        .bind(user.team_id).bind(user.id).fetch_all(&pool)
        .await.map_err(|_| axum::http::StatusCode::INTERNAL_SERVER_ERROR)?;

    let template = PendingRequestsTemplate { requests };
    let html = template
        .render()
        .map_err(|_| axum::http::StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(Html(html))
}

pub async fn approved_requests(
    State(pool): State<PgPool>,
    auth: AuthSession,
) -> Result<Html<String>, axum::http::StatusCode> {
    let user = auth.user.ok_or(StatusCode::UNAUTHORIZED)?;
    let requests = sqlx::query_as::<_, Request>(
        "SELECT leave_requests.id, users.name, users.job_title, start_date, end_date, days, leave_type, comments, leave_requests.created_at::date AS apply_date FROM leave_requests JOIN users ON users.id = leave_requests.user_id WHERE users.team_id = $1 AND leave_requests.status = 'approved' AND users.id != $2",)
        .bind(user.team_id).bind(user.id).fetch_all(&pool)
        .await.map_err(|_| axum::http::StatusCode::INTERNAL_SERVER_ERROR)?;

    let template = RequestsListTemplate {
        requests,
        request_status: "Approved".to_string(),
    };
    let html = template
        .render()
        .map_err(|_| axum::http::StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(Html(html))
}

pub async fn rejected_requests(
    State(pool): State<PgPool>,
    auth: AuthSession,
) -> Result<Html<String>, axum::http::StatusCode> {
    let user = auth.user.ok_or(StatusCode::UNAUTHORIZED)?;
    let requests = sqlx::query_as::<_, Request>(
        "SELECT leave_requests.id, users.name, users.job_title, start_date, end_date, days, leave_type, comments, leave_requests.created_at::date AS apply_date FROM leave_requests JOIN users ON users.id = leave_requests.user_id WHERE users.team_id = $1 AND leave_requests.status = 'rejected' AND users.id != $2",)
        .bind(user.team_id).bind(user.id).fetch_all(&pool)
        .await.map_err(|_| axum::http::StatusCode::INTERNAL_SERVER_ERROR)?;

    let template = RequestsListTemplate {
        requests,
        request_status: "Rejected".to_string(),
    };
    let html = template
        .render()
        .map_err(|_| axum::http::StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(Html(html))
}

pub async fn my_leaves_list(
    State(pool): State<PgPool>,
    auth: AuthSession,
    headers: axum::http::HeaderMap,
) -> Result<Html<String>, axum::http::StatusCode> {
    let auth_clone = auth.clone();
    let user = auth.user.ok_or(StatusCode::UNAUTHORIZED)?;
    let leaves = sqlx::query_as::<_, LeaveRequestOutput>(
        "SELECT id, start_date, end_date, days, leave_type, comments, status FROM leave_requests WHERE user_id = $1",
    )
    .bind(user.id) // Replace with actual user ID
    .fetch_all(&pool)
    .await
        .map_err(|_| axum::http::StatusCode::INTERNAL_SERVER_ERROR)?;

    let pending_count = get_pending_requests_count(State(pool.clone()), auth_clone)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    // Render the template
    let is_htmx_request = headers.contains_key("HX-Request");
    let html = if is_htmx_request {
        let template = LeavesListTemplatePartial { leaves };
        template
            .render()
            .map_err(|_| axum::http::StatusCode::INTERNAL_SERVER_ERROR)?
    } else {
        let template = LeavesListTemplate {
            leaves,
            role: user.role,
            pending_count,
        };
        template
            .render()
            .map_err(|_| axum::http::StatusCode::INTERNAL_SERVER_ERROR)?
    };

    Ok(Html(html))
}

fn get_days_of_month(year: i32, month: u32) -> Vec<u32> {
    // Create a NaiveDate for the first day of the specified month
    let first_day = NaiveDate::from_ymd_opt(year, month, 1).unwrap();

    // Calculate the number of days in the month
    let days_in_month = if month == 12 {
        NaiveDate::from_ymd_opt(year + 1, 1, 1) // Next year, January 1st
    } else {
        NaiveDate::from_ymd_opt(year, month + 1, 1) // Next month, 1st
    }
    .unwrap()
    .signed_duration_since(first_day)
    .num_days() as u32;

    // Generate a vector of days (1..=days_in_month)
    (1..=days_in_month).collect()
}

// Custom filter to convert month number to name
fn get_month_name(month: &u32) -> String {
    let months = [
        "January",
        "February",
        "March",
        "April",
        "May",
        "June",
        "July",
        "August",
        "September",
        "October",
        "November",
        "December",
    ];

    if *month >= 1 && *month <= 12 {
        months[(*month - 1) as usize].to_string()
    } else {
        "Invalid month".to_string()
    }
}

pub async fn calendar(
    State(pool): State<PgPool>,
    auth: AuthSession,
    headers: axum::http::HeaderMap,
    Query(params): Query<HashMap<String, String>>,
) -> Result<Html<String>, axum::http::StatusCode> {
    let auth_clone = auth.clone();
    let user = auth.user.ok_or(StatusCode::UNAUTHORIZED)?;

    // Get the month and year from query parameters (default to current month/year)
    let now = chrono::Local::now();
    let month = params
        .get("month")
        .and_then(|m| m.parse::<u32>().ok())
        .unwrap_or(now.month());
    let year = params
        .get("year")
        .and_then(|y| y.parse::<i32>().ok())
        .unwrap_or(now.year());
    let days_of_month = get_days_of_month(year, month);
    let days = days_of_month.len() as u32;

    let team_members =
        sqlx::query_as::<_, TeamMember>("SELECT id, name, job_title FROM users WHERE team_id = $1")
            .bind(user.team_id)
            .fetch_all(&pool)
            .await
            .map_err(|e| {
                println!("Failed to fetch team members: {:?}", e);
                axum::http::StatusCode::INTERNAL_SERVER_ERROR
            })?;

    let team_leaves = sqlx::query_as::<_, Request>("SELECT leave_requests.id, users.name, users.job_title, start_date, end_date, days, leave_type, comments, leave_requests.created_at::date AS apply_date FROM leave_requests JOIN users ON users.id = leave_requests.user_id WHERE leave_requests.status = 'approved' AND users.team_id = $1 AND ((start_date >= $2 AND end_date <= $3) OR (start_date <= $2 AND (end_date  >= $2 AND end_date <= $3)) OR ((start_date >= $2 AND start_date <= $3) AND end_date >= $3) OR (start_date <= $2 AND end_date >= $3))",)
        .bind(user.team_id)
        .bind(NaiveDate::from_ymd_opt(year, month, 1).unwrap()) // Start of the month
        .bind(NaiveDate::from_ymd_opt(year, month, days).unwrap()) //End of the month
        .fetch_all(&pool)
        .await
        .map_err(|_| axum::http::StatusCode::INTERNAL_SERVER_ERROR)?;
    // Organize the data into a more convenient format for rendering the Gantt chart

    let gantt_data = organize_for_gantt(team_leaves, team_members);
    let is_htmx_request = headers.contains_key("HX-Request");
    let pending_count = get_pending_requests_count(State(pool.clone()), auth_clone)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let html = if is_htmx_request {
        let template = CalendarTemplatePartial {
            gantt_data,
            days_of_month,
            current_month: month,
            current_year: year,
            current_month_name: get_month_name(&month),
        };
        template
            .render()
            .map_err(|_| axum::http::StatusCode::INTERNAL_SERVER_ERROR)?
    } else {
        let template = CalendarTemplate {
            role: user.role,
            gantt_data,
            days_of_month,
            current_month: month,
            current_year: year,
            current_month_name: get_month_name(&month),
            pending_count,
        };
        template
            .render()
            .map_err(|_| axum::http::StatusCode::INTERNAL_SERVER_ERROR)?
    };
    Ok(Html(html))
}

pub async fn calendar_api(
    State(pool): State<PgPool>,
    auth: AuthSession,
    // Query(params): Query<HashMap<String, String>>,
) -> Result<Json<Vec<Request>>, StatusCode> {
    let user = auth.user.ok_or(StatusCode::UNAUTHORIZED)?;

    // let now = chrono::Local::now();
    // let month = params
    //     .get("month")
    //     .and_then(|m| m.parse().ok())
    //     .unwrap_or(now.month());
    // let year = params
    //     .get("year")
    //     .and_then(|y| y.parse().ok())
    //     .unwrap_or(now.year());

    // let start_of_month = NaiveDate::from_ymd_opt(year, month, 1).unwrap();
    // let end_of_month = NaiveDate::from_ymd_opt(
    //     year,
    //     month,
    //     chrono::NaiveDate::from_ymd_opt(year, month, 1)
    //         .unwrap()
    //         .with_day(1)
    //         .unwrap()
    //         .day(),
    // )
    // .unwrap();

    let team_leaves = sqlx::query_as::<_, Request>(
        "SELECT leave_requests.id, users.name, users.job_title, start_date, end_date, 
                days, leave_type, comments, 
                leave_requests.created_at::date AS apply_date 
         FROM leave_requests 
         JOIN users ON users.id = leave_requests.user_id 
         WHERE leave_requests.status = 'approved' 
         AND users.id = $1",
    )
    .bind(user.id)
    .fetch_all(&pool)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(team_leaves))
}

// Function to organize data for Gantt chart rendering
fn organize_for_gantt(
    team_leaves: Vec<Request>,
    team_members: Vec<TeamMember>,
) -> Vec<EmployeeGanttData> {
    let mut gantt_data: Vec<EmployeeGanttData> = Vec::new();

    for member in team_members {
        gantt_data.push(EmployeeGanttData {
            name: member.name,
            job_title: member.job_title,
            leaves: vec![],
        });
    }

    // Organize data by employee
    for leave in team_leaves {
        // Precompute the days of the leave
        let leave_days: Vec<u32> = (leave.start_date.day()..=leave.end_date.day()).collect();
        if let Some(employee) = gantt_data.iter_mut().find(|e| e.name == leave.name) {
            employee.leaves.push(LeaveGanttData {
                start_date: leave.start_date,
                end_date: leave.end_date,
                leave_type: leave.leave_type,
                leave_days,
            });
        } else {
            gantt_data.push(EmployeeGanttData {
                name: leave.name,
                job_title: leave.job_title,
                leaves: vec![LeaveGanttData {
                    start_date: leave.start_date,
                    end_date: leave.end_date,
                    leave_type: leave.leave_type,
                    leave_days,
                }],
            });
        }
    }

    gantt_data
}

pub async fn reject_request(
    State(pool): State<PgPool>,
    auth: AuthSession,
    Path(id): Path<i32>,
) -> Result<impl IntoResponse, axum::http::StatusCode> {
    let user = auth.user.ok_or(StatusCode::UNAUTHORIZED)?;
    let query = "UPDATE leave_requests SET status = 'rejected', approved_by = $1 WHERE id = $2;";
    let result = sqlx::query(query)
        .bind(user.id)
        .bind(id)
        .execute(&pool)
        .await;

    let days = sqlx::query_scalar::<_, i32>("SELECT days FROM leave_requests WHERE id = $1")
        .bind(id)
        .fetch_one(&pool)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Error: {}", e)));

    let user_query = "SELECT user_id FROM leave_requests WHERE id = $1";
    let user_request_id = sqlx::query_scalar::<_, i32>(user_query)
        .bind(id)
        .fetch_one(&pool)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Error: {}", e)));

    // println!("Team lead id is: {}", request_id.clone().unwrap());

    let _update_leaves =
        sqlx::query("UPDATE users SET available_leaves = available_leaves + $1 WHERE id = $2")
            .bind(days.unwrap())
            .bind(user_request_id.clone().unwrap())
            .execute(&pool)
            .await
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Error: {}", e)));

    create_notification(
        &pool,
        user_request_id.unwrap(),
        id,
        format!("Leave request: {} rejected by {}", id, user.name),
    )
    .await
    .unwrap_or_default();

    match result {
        Ok(result) => {
            if result.rows_affected() == 0 {
                // No rows were deleted, meaning the leave request wasn't found
                Err(StatusCode::NOT_FOUND)
            } else {
                // Successfully deleted
                Ok(StatusCode::OK)
            }
        }
        Err(e) => {
            // Log the error here
            eprintln!("Error deleting leave request: {:?}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

pub async fn approve_request(
    State(pool): State<PgPool>,
    auth: AuthSession,
    Path(id): Path<i32>,
) -> Result<impl IntoResponse, axum::http::StatusCode> {
    let user = auth.user.ok_or(StatusCode::UNAUTHORIZED)?;
    let query = "UPDATE leave_requests SET status = 'approved', approved_by = $1 WHERE id = $2;";
    let result = sqlx::query(query)
        .bind(user.id)
        .bind(id)
        .execute(&pool)
        .await;

    let start_date =
        sqlx::query_scalar::<_, NaiveDate>("SELECT start_date FROM leave_requests where id = $1")
            .bind(id)
            .fetch_one(&pool)
            .await
            .map_err(|e| {
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    format!("Error fetching the start date of the leave request: {}", e),
                )
            });
    let end_date =
        sqlx::query_scalar::<_, NaiveDate>("SELECT end_date FROM leave_requests where id = $1")
            .bind(id)
            .fetch_one(&pool)
            .await
            .map_err(|e| {
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    format!("Error fetching the end date of the leave request: {}", e),
                )
            });

    let current_year_days = count_weekdays_in_current_year(start_date.unwrap(), end_date.unwrap());

    let user_query = "SELECT user_id FROM leave_requests WHERE id = $1";
    let user_request_id = sqlx::query_scalar::<_, i32>(user_query)
        .bind(id)
        .fetch_one(&pool)
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Error fetching user information: {}", e),
            )
        });

    // println!("Team lead id is: {}", request_id.clone().unwrap());

    let _result =
        sqlx::query("UPDATE users SET used_leaves = (used_leaves + $1) WHERE users.id = $2")
            .bind(current_year_days)
            .bind(user_request_id.clone().unwrap())
            .execute(&pool)
            .await
            .map_err(|e| {
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    format!("Error updating the used leaves: {}", e),
                )
            });

    create_notification(
        &pool,
        user_request_id.unwrap(),
        id,
        format!("Leave request: {} approved by {}", id, user.name),
    )
    .await
    .unwrap_or_default();

    match result {
        Ok(result) => {
            if result.rows_affected() == 0 {
                // No rows were updated, meaning the leave request wasn't found
                Err(StatusCode::NOT_FOUND)
            } else {
                // Successfully deleted
                Ok(StatusCode::OK)
            }
        }
        Err(e) => {
            // Log the error here
            eprintln!("Error approving leave request: {:?}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

pub async fn get_pending_requests_count(
    State(pool): State<PgPool>,
    auth: AuthSession,
) -> Result<i64, StatusCode> {
    let user = auth.user.ok_or(StatusCode::UNAUTHORIZED)?;

    if user.role == Role::Officer {
        return Err(StatusCode::UNAUTHORIZED);
    }

    let requests = if user.role == Role::Human_Resources {
        sqlx::query_scalar::<_, i64>(
            "SELECT COUNT(leave_requests.id) FROM leave_requests JOIN users ON users.id = leave_requests.user_id WHERE users.role = 'admin' AND leave_requests.status = 'pending'",)
            .fetch_one(&pool)
            .await.map_err(|_| axum::http::StatusCode::FORBIDDEN)?
    } else {
        sqlx::query_scalar::<_, i64>(
            "SELECT COUNT(leave_requests.id) FROM leave_requests JOIN users ON users.id = leave_requests.user_id WHERE users.team_id = $1 AND leave_requests.status = 'pending' AND users.id != $2",)
            .bind(user.team_id).bind(user.id).fetch_one(&pool)
            .await.map_err(|_| axum::http::StatusCode::FORBIDDEN)?
    };

    Ok(requests)
}
