pub mod auth;
pub mod fallback;
pub mod leave;
pub mod notification;
pub mod profile;

use crate::models::leave::*;
use crate::models::notification::*;
use crate::models::team::TeamMember;
use crate::models::user::*;
use crate::templates::*;
use askama::Template;
use axum::{
    body::Body,
    extract::{Form, Path, Query, State},
    http::{HeaderMap, StatusCode},
    response::{Html, IntoResponse, Redirect, Response, Result},
    Json,
};
use axum_messages::Messages;
use chrono::{Datelike, Duration, NaiveDate, Weekday};
use serde::Deserialize;
use sqlx::postgres::PgPool;
use std::collections::HashMap;
