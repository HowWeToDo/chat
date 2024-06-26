use axum::{extract::State, http::StatusCode, response::IntoResponse, Json};
use serde::{Deserialize, Serialize};

use crate::{
    error::{AppError, ErrorOutput},
    AppState, CreateUser, SigninUser, User,
};

#[derive(Debug, Serialize, Deserialize)]
pub struct AuthOutput {
    token: String,
}

pub(crate) async fn signup_handler(
    State(state): State<AppState>,
    Json(input): Json<CreateUser>,
) -> Result<impl IntoResponse, AppError> {
    let user = User::create(&input, &state.pool).await?;
    let token = state.ek.sign(user)?;
    let body = Json(AuthOutput { token });
    Ok((StatusCode::CREATED, body))
}

pub(crate) async fn signin_handler(
    State(state): State<AppState>,
    Json(input): Json<SigninUser>,
) -> Result<impl IntoResponse, AppError> {
    let user = User::verify(&input, &state.pool).await?;

    match user {
        Some(user) => {
            let token = state.ek.sign(user)?;
            Ok((StatusCode::OK, Json(AuthOutput { token })).into_response())
        }
        None => {
            let boby = Json(ErrorOutput::new("Invalid email or password"));
            Ok((StatusCode::FORBIDDEN, boby).into_response())
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::AppConfig;

    use super::*;
    use anyhow::Result;
    use http_body_util::BodyExt;

    #[tokio::test]
    async fn signup_should_work() -> Result<()> {
        let config = AppConfig::load()?;
        let (_tdb, state) = AppState::new_for_test(config).await?;
        let input = CreateUser::new("none", "Tyr Chen", "tchen@acme.org", "hunter42");
        let res = signup_handler(State(state), Json(input))
            .await?
            .into_response();
        assert_eq!(res.status(), StatusCode::CREATED);
        let boby = res.into_body().collect().await?.to_bytes();
        let res: AuthOutput = serde_json::from_slice(&boby)?;
        assert_ne!(res.token, "");
        Ok(())
    }

    #[tokio::test]
    async fn signup_duplicate_user_should_409() -> Result<()> {
        let config = AppConfig::load()?;
        let (_tdb, state) = AppState::new_for_test(config).await?;
        let input = CreateUser::new("none", "Tyr Chen", "tchen@acme.org", "hunter42");
        signup_handler(State(state.clone()), Json(input.clone())).await?;
        let res = signup_handler(State(state.clone()), Json(input.clone()))
            .await
            .into_response();
        assert_eq!(res.status(), StatusCode::CONFLICT);
        let boby = res.into_body().collect().await?.to_bytes();
        let res: ErrorOutput = serde_json::from_slice(&boby)?;
        assert_eq!(res.error, "email already exists: tchen@acme.org");
        Ok(())
    }

    #[tokio::test]
    async fn signin_should_work() -> Result<()> {
        let config = AppConfig::load()?;
        let (_tdb, state) = AppState::new_for_test(config).await?;
        let user = CreateUser::new("none", "Alice", "alice@acme.org", "Hunter42");
        User::create(&user, &state.pool).await?;
        let input = SigninUser::new(&user.email, &user.password);
        let res = signin_handler(State(state), Json(input))
            .await?
            .into_response();
        assert_eq!(res.status(), StatusCode::OK);
        let boby = res.into_body().collect().await?.to_bytes();
        let res: AuthOutput = serde_json::from_slice(&boby)?;
        assert_ne!(res.token, "");
        Ok(())
    }

    #[tokio::test]
    async fn signin_with_non_exists_user_should_403() -> Result<()> {
        let config = AppConfig::load()?;
        let (_tdb, state) = AppState::new_for_test(config).await?;
        let email = "alice@acme.org";
        let password = "Hunter42";
        let input = SigninUser::new(email, password);
        let res = signin_handler(State(state), Json(input))
            .await
            .into_response();
        assert_eq!(res.status(), StatusCode::FORBIDDEN);
        let boby = res.into_body().collect().await?.to_bytes();
        let res: ErrorOutput = serde_json::from_slice(&boby)?;
        assert_eq!(res.error, "Invalid email or password");
        Ok(())
    }
}
