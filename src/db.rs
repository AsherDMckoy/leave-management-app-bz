// use crate::models::leave::LeaveRequestInput;
// use crate::models::leave::LeaveType;
// use crate::models::team::Team;
// use crate::models::user::User;
// // use sqlx::PgPool;
// use chrono::NaiveDate;
// use rand::prelude::SliceRandom;
// use std::error::Error;

// pub async fn load_requests(
//     transaction: &mut sqlx::Transaction<'_, sqlx::Postgres>,
// ) -> Result<(), Box<dyn Error>> {
//     let mut ids: Vec<i64> = std::vec::Vec::new();
//     ids.push(2117);
//     ids.push(2118);
//     ids.push(2119);
//     ids.push(2120);
//     ids.push(2121);
//     for _ in 1..=10 {
//         let request = LeaveRequestInput {
//             start_date: NaiveDate::parse_from_str("2024-12-14", "%Y-%m-%d").unwrap(),
//             end_date: NaiveDate::parse_from_str("2024-12-31", "%Y-%m-%d").unwrap(),
//             leave_type: LeaveType::Vacation,
//             comments: Some("My annual leave".to_string()),
//         };
//         let user_id = *ids.choose(&mut rand::thread_rng()).unwrap();
//         let query = "INSERT INTO leave_requests (user_id, start_date, end_date, comments, leave_type) VALUES ($1, $2, $3, $4, $5)";
//         sqlx::query(query)
//             .bind(user_id)
//             .bind(request.start_date)
//             .bind(request.end_date)
//             .bind(request.comments)
//             .bind(&request.leave_type)
//             .execute(&mut **transaction)
//             .await?;
//     }
//     Ok(())
// }

// async fn create(
//     user: &User,
//     transaction: &mut sqlx::Transaction<'_, sqlx::Postgres>,
// ) -> Result<(), Box<dyn Error>> {
//     let query = "INSERT INTO users (id, username, password_hash, email, role, name, team_id) VALUES ($1, $2, $3, $4, $5, $6, $7)";
//     sqlx::query(query)
//         .bind(&user.id)
//         .bind(&user.username)
//         .bind(&user.password)
//         .bind(&user.email)
//         .bind(&user.role)
//         .bind(&user.name)
//         .bind(&user.team_id)
//         .execute(&mut **transaction)
//         .await?;
//     Ok(())
// }
// async fn create_team(
//     team: &Team,
//     transaction: &mut sqlx::Transaction<'_, sqlx::Postgres>,
// ) -> Result<(), Box<dyn Error>> {
//     let query = "INSERT INTO teams (id, name, team_lead_id) VALUES ($1, $2, $3)";
//     sqlx::query(query)
//         .bind(&team.id)
//         .bind(&team.name)
//         .bind(&team.team_lead_id)
//         .execute(&mut **transaction)
//         .await?;
//     Ok(())
// }
// async fn assign_team_lead(
//     team: &Team,
//     user: &User,
//     transaction: &mut sqlx::Transaction<'_, sqlx::Postgres>,
// ) -> Result<(), Box<dyn Error>> {
//     println!("Assigning Team Lead...");
//     let query = "UPDATE teams SET team_lead_id = $1 WHERE name = $2";
//     sqlx::query(query)
//         .bind(&user.id)
//         .bind(&team.name)
//         .execute(&mut **transaction)
//         .await?;
//     Ok(())
// }
