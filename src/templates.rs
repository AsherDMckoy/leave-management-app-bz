use crate::models::leave::{
    EmployeeGanttData, LeaveRequestOutput, MyLeavePreview, Request, TeamLeavePreview,
};
use crate::models::notification::Notification;
use crate::models::team;
use crate::models::user::{Role, User};
use askama::Template;
use axum::http::StatusCode;
use axum::response::{Html, IntoResponse, Response};
use axum_messages::Message;

pub struct HtmlTemplate<T>(pub T);
impl<T> IntoResponse for HtmlTemplate<T>
where
    T: Template,
{
    fn into_response(self) -> Response {
        match self.0.render() {
            Ok(html) => Html(html).into_response(),
            Err(err) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Failed to render template. Error: {err}"),
            )
                .into_response(),
        }
    }
}

#[derive(Template)]
#[template(path = "index.html")]
pub struct IndexTemplate {
    pub username: String,
    pub role: Role,
    pub pending_count: i64,
    pub available_leaves: i32,
    pub used_leaves: i32,
    pub overdue_leaves: i32,
}

#[derive(Template)]
#[template(path = "notifications.html")]
pub struct NotificationsTemplate {
    pub notifications: Vec<Notification>,
}

#[derive(Template)]
#[template(path = "dashboard.html")]
pub struct DashboardTemplate {
    pub username: String,
    pub available_leaves: i32,
    pub used_leaves: i32,
    pub overdue_leaves: i32,
}

#[derive(Template)]
#[template(path = "calendar.html")]
pub struct CalendarTemplate {
    pub role: Role,
    pub gantt_data: Vec<EmployeeGanttData>,
    pub days_of_month: Vec<u32>,
    pub current_month: u32,
    pub current_year: i32,
    pub current_month_name: String,
    pub pending_count: i64,
}

#[derive(Template)]
#[template(path = "calendar_partial.html")]
pub struct CalendarTemplatePartial {
    pub gantt_data: Vec<EmployeeGanttData>,
    pub days_of_month: Vec<u32>,
    pub current_month: u32,
    pub current_year: i32,
    pub current_month_name: String,
}

#[derive(Template)]
#[template(path = "login.html")]
pub struct LoginTemplate {
    pub messages: Vec<Message>,
    pub next: Option<String>,
}

#[derive(Template)]
#[template(path = "requests.html")]
pub struct RequestsTemplate {
    pub role: Role,
    pub requests: Vec<Request>,
    pub pending_count: i64,
    pub approved_count: i64,
    pub rejected_count: i64,
}

#[derive(Template)]
#[template(path = "requests_partial.html")]
pub struct RequestsTemplatePartial {
    pub requests: Vec<Request>,
    pub pending_count: i64,
    pub approved_count: i64,
    pub rejected_count: i64,
}

#[derive(Template)]
#[template(path = "new_user.html")]
pub struct NewUserTemplate {
    pub role: Role,
    pub pending_count: i64,
    pub csrf_token: String,
    pub teams: Vec<team::Team>,
}

#[derive(Template)]
#[template(path = "new_user_partial.html")]
pub struct NewUserTemplatePartial {
    pub csrf_token: String,
    pub teams: Vec<team::Team>,
}

#[derive(Template)]
#[template(path = "pending_requests.html")]
pub struct PendingRequestsTemplate {
    pub requests: Vec<Request>,
}

#[derive(Template)]
#[template(path = "requests_list.html")]
pub struct RequestsListTemplate {
    pub requests: Vec<Request>,
    pub request_status: String,
}

#[derive(Template)]
#[template(path = "leaves_list.html")]
pub struct LeavesListTemplate {
    pub leaves: Vec<LeaveRequestOutput>,
    pub role: Role,
    pub pending_count: i64,
}

#[derive(Template)]
#[template(path = "leaves_list_partial.html")]
pub struct LeavesListTemplatePartial {
    pub leaves: Vec<LeaveRequestOutput>,
}

#[derive(Template)]
#[template(path = "new_leave.html")]
pub struct NewLeaveTemplate;

#[derive(Template)]
#[template(path = "edit_leave.html")]
pub struct EditLeaveTemplate {
    pub leave: LeaveRequestOutput,
}

#[derive(Template)]
#[template(path = "leave_row.html")]
pub struct LeaveRowTemplate {
    pub leave: LeaveRequestOutput,
}

#[derive(Template)]
#[template(path = "my_leaves_prev.html")]
pub struct MyLeavesPrevTemplate {
    pub leaves: Vec<MyLeavePreview>,
}

#[derive(Template)]
#[template(path = "team_leaves_prev.html")]
pub struct TeamLeavesPrevTemplate {
    pub leaves: Vec<TeamLeavePreview>,
}

#[derive(Template)]
#[template(path = "profile.html")]
pub struct ProfileTemplate {
    pub user: User,
    pub team_name: String,
    pub supervisor_name: String,
    pub job_title: String,
    pub pending_count: i64,
    pub csrf_token: String,
}

#[derive(Template)]
#[template(path = "profile_partial.html")]
pub struct ProfileTemplatePartial {
    pub user: User,
    pub team_name: String,
    pub supervisor_name: String,
    pub job_title: String,
    pub csrf_token: String,
}
