use crate::dto::auth::Claims;
use crate::infrastructure::errors::{AppError, AppResult};
use crate::infrastructure::state::AppState;
use crate::repository::user::UserRepository;
use axum::body::Body;
use axum::extract::{Request, State};
use axum::http::header::AUTHORIZATION;
use axum::middleware::Next;
use axum::response::Response;
use jsonwebtoken::{decode, DecodingKey, TokenData, Validation};

pub async fn authentication_middleware(
    State(state): State<AppState>,
    mut req: Request,
    next: Next,
) -> AppResult<Response<Body>> {
    let auth_header = req.headers_mut().get(AUTHORIZATION);
    let auth_header = match auth_header {
        Some(header) => header
            .to_str()
            .map_err(|_| AppError::Forbidden("Empty header is not allowed".to_string()))?,
        None => {
            return Err(AppError::Forbidden(
                "Please add the JWT token to the header".to_string(),
            ))
        }
    };
    let mut header = auth_header.split_whitespace();
    let (_bearer, token) = (header.next(), header.next());
    let token_data = match decode_jwt(
        token.ok_or(AppError::Forbidden(String::from("Missing token")))?,
        &state.config.jwt_secret,
    ) {
        Ok(data) => data,
        Err(_) => return Err(AppError::Unauthorized),
    };
    // Fetch the user details from the database
    let current_user = UserRepository::new(state.db)
        .get_by_id(&token_data.claims.sub)
        .await
        .map_err(|_| AppError::Forbidden("User not found".to_string()))?;
    req.extensions_mut().insert(current_user);
    Ok(next.run(req).await)
}

pub fn decode_jwt(jwt_token: &str, secret: &str) -> AppResult<TokenData<Claims>> {
    let result = decode(
        jwt_token,
        &DecodingKey::from_secret(secret.as_ref()),
        &Validation::default(),
    )
    .map_err(|e| AppError::InternalServerErrorWithContext(e.to_string()));
    result
}
