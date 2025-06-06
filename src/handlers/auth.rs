use super::*;
//const AUTH_TOKEN: &str = "auth-token";

// This allows us to extract the "next" field from the query string. We use this
// to redirect after log in.
#[derive(Debug, Deserialize)]
pub struct NextUrl {
    next: Option<String>,
}

pub async fn login(
    mut auth_session: AuthSession,
    messages: Messages,
    Form(creds): Form<Credentials>,
) -> impl IntoResponse {
    match auth_session.authenticate(creds.clone()).await {
        Ok(Some(user)) => match auth_session.login(&user).await {
            Ok(_) => {
                messages.success(format!("Successfully logged in as {}", user.username));
                match creds.next {
                    Some(next) => Redirect::to(&next),
                    None => Redirect::to("/"),
                }
                .into_response()
            }
            Err(e) => {
                eprintln!("Login error: {:?}", e);
                StatusCode::INTERNAL_SERVER_ERROR.into_response()
            }
        },
        Ok(None) => {
            messages.error("Invalid credentials");
            let login_url = match creds.next {
                Some(next) => format!("/authorize?next={}", next),
                None => "/authorize".to_string(),
            };
            Redirect::to(&login_url).into_response()
        }
        Err(e) => {
            eprintln!("Authentication error: {:?}", e);
            StatusCode::INTERNAL_SERVER_ERROR.into_response()
        }
    }
    // if form.username == "ada" && form.password == "lovelace" {
    //     // Successful login, redirect to the dashboard or another page
    //     cookies.add(Cookie::new(AUTH_TOKEN, "user-1.exp.sign"));
    //     let mut headers = HeaderMap::new();
    //     headers.insert("HX-Redirect", "/".parse().unwrap());
    //     headers.into_response()
    // } else {
    //     // Failed login, redirect back to the login page or show an error
    //     let mut headers = HeaderMap::new();
    //     headers.insert("HX-Redirect", "/authorize".parse().unwrap());
    //     headers.into_response()
    // }
}
pub async fn logout(mut auth_session: AuthSession) -> impl IntoResponse {
    match auth_session.logout().await {
        Ok(_) => {
            let mut headers = HeaderMap::new();
            headers.insert("HX-Redirect", "/authorize".parse().unwrap());
            headers.into_response()
        } // Redirect::to("/authorize").into_response(),
        Err(_) => StatusCode::INTERNAL_SERVER_ERROR.into_response(),
    }
    // let mut headers = HeaderMap::new();
    // headers.insert("HX-Redirect", "/authorize".parse().unwrap());
    // headers.into_response()
}

pub async fn login_form(
    messages: Messages,
    Query(NextUrl { next }): Query<NextUrl>,
) -> impl IntoResponse {
    let template = LoginTemplate {
        messages: messages.into_iter().collect(),
        next,
    };
    HtmlTemplate(template)
}
