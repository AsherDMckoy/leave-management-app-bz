use super::*;

#[derive(Debug, Clone, Deserialize, Serialize, sqlx::FromRow)]
pub struct Team {
    pub id: i32,
    pub name: String,
    pub team_lead_id: Option<i32>,
}

#[derive(Debug, Clone, Deserialize, Serialize, sqlx::FromRow)]
pub struct TeamMember {
    pub id: i32,
    pub name: String,
    pub job_title: String,
}
