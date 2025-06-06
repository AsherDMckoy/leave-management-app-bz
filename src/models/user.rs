use super::*;
use argon2::{
    password_hash::{PasswordHash, PasswordVerifier},
    Argon2,
};

use std::fmt;
use std::str::FromStr;

#[derive(Debug, Clone, Deserialize, Serialize, sqlx::Type, PartialEq)]
#[sqlx(type_name = "role")]
#[sqlx(rename_all = "lowercase")]
pub enum Role {
    Admin,
    Team_Lead,
    Officer,
    Human_Resources,
}

#[derive(Clone, Deserialize, Serialize, sqlx::FromRow)]
pub struct User {
    pub id: i32,
    pub username: String,
    pub password: String,
    pub email: String,
    pub role: Role,
    pub name: String,
    pub team_id: i32,
}

#[derive(Clone, Deserialize, Serialize, sqlx::FromRow)]
pub struct UserForm {
    pub csrf_token: String,
    pub name: String,
    pub email: String,
    pub username: String,
    pub password: String,
    pub team_name: String,
    pub role: Role,
    pub job_title: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Credentials {
    pub username: String,
    pub password: String,
    pub next: Option<String>,
}

// Here we've implemented `Debug` manually to avoid accidentally logging the
// password hash.
impl std::fmt::Debug for User {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("User")
            .field("id", &self.id)
            .field("username", &self.username)
            .field("password", &"[redacted]")
            .field("email", &self.email)
            .field("role", &self.role)
            .field("name", &self.name)
            .field("team_id", &"[redacted]")
            .finish()
    }
}

impl fmt::Display for Role {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Role::Admin => write!(f, "Admin"),
            Role::Team_Lead => write!(f, "Team Lead"),
            Role::Officer => write!(f, "Officer"),
            Role::Human_Resources => write!(f, "Human Resources"),
        }
    }
}

impl FromStr for Role {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "admin" => Ok(Role::Admin),
            "team_lead" => Ok(Role::Team_Lead),
            "officer" => Ok(Role::Officer),
            "human_resources" => Ok(Role::Human_Resources),
            _ => Err(format!("Unknown role: {}", s)),
        }
    }
}

impl PartialEq<str> for Role {
    fn eq(&self, other: &str) -> bool {
        self.to_string().eq_ignore_ascii_case(other)
    }
}

impl PartialEq<&str> for Role {
    fn eq(&self, other: &&str) -> bool {
        self.to_string().eq_ignore_ascii_case(other)
    }
}

impl PartialEq<String> for Role {
    fn eq(&self, other: &String) -> bool {
        self == other.as_str()
    }
}

impl PartialEq<Role> for str {
    fn eq(&self, other: &Role) -> bool {
        other == self
    }
}

impl PartialEq<Role> for String {
    fn eq(&self, other: &Role) -> bool {
        other == self
    }
}

impl AuthUser for User {
    type Id = i32;

    fn id(&self) -> Self::Id {
        self.id
    }

    fn session_auth_hash(&self) -> &[u8] {
        self.password.as_bytes() // We use the password hash as the auth
                                 // hash--what this means
                                 // is when the user changes their password the
                                 // auth session becomes invalid.
    }
}

#[derive(Debug, Clone)]
pub struct Backend {
    db: PgPool,
}

impl Backend {
    pub fn new(db: PgPool) -> Self {
        Self { db }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error(transparent)]
    Sqlx(#[from] sqlx::Error),

    #[error(transparent)]
    TaskJoin(#[from] task::JoinError),
}

#[async_trait]
impl AuthnBackend for Backend {
    type User = User;
    type Credentials = Credentials;
    type Error = Error;

    async fn authenticate(
        &self,
        creds: Self::Credentials,
    ) -> Result<Option<Self::User>, Self::Error> {
        let user: Option<Self::User> = sqlx::query_as("SELECT id, username, password_hash AS password, email, role, name, team_id FROM users WHERE username = $1")
            .bind(&creds.username)
            .fetch_optional(&self.db)
            .await?;

        // Verifying the password is blocking and potentially slow, so we'll do so via
        // `spawn_blocking`.
        task::spawn_blocking(move || match user {
            Some(user) => {
                let parsed_hash = PasswordHash::new(&user.password)
                    .map_err(|e| Error::Sqlx(sqlx::Error::Protocol(e.to_string())))?;

                if Argon2::default()
                    .verify_password(creds.password.as_bytes(), &parsed_hash)
                    .is_ok()
                {
                    Ok(Some(user))
                } else {
                    Ok(None)
                }
            }
            None => Ok(None),
        })
        .await?
    }

    async fn get_user(&self, user_id: &UserId<Self>) -> Result<Option<Self::User>, Self::Error> {
        let user = sqlx::query_as("SELECT id, username, password_hash AS password, email, role, name, team_id FROM users WHERE id = $1")
            .bind(user_id)
            .fetch_optional(&self.db)
            .await?;

        Ok(user)
    }
}

// We use a type alias for convenience.
//
// Note that we've supplied our concrete backend here.
pub type AuthSession = axum_login::AuthSession<Backend>;
