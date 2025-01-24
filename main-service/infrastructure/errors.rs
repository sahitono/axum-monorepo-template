#![allow(dead_code)]
use std::borrow::Cow;
use std::collections::HashMap;
use std::fmt::{Debug, Display, Formatter};

use axum::extract::rejection::{JsonRejection, PathRejection, QueryRejection};
use axum::response::Response;
use axum::{http::StatusCode, response::IntoResponse, Json};
use base64::DecodeError;
use sea_orm::{DbErr, TransactionError};
use serde::{Deserialize, Serialize};
use serde_json::json;
use thiserror::Error;
use tracing::debug;
use tracing::log::error;
use validator::{ValidationError, ValidationErrors, ValidationErrorsKind};
use crate::dto::base::BaseResponse;

pub type AppResult<T> = Result<T, AppError>;

pub type ErrorMap = HashMap<Cow<'static, str>, Vec<Cow<'static, str>>>;

#[derive(Debug, Deserialize, Serialize)]
pub struct HttpError {
    pub error: String,
}

impl HttpError {
    #[must_use]
    pub fn new(error: String) -> Self {
        Self { error }
    }
}

#[derive(Debug, Error, Serialize)]
pub struct ValidationMessageErrors {
    pub errors: Vec<ValidationMessageError>,
}
impl ValidationMessageErrors {
    #[must_use]
    pub fn from(errors: Vec<ValidationMessageError>) -> Self {
        Self { errors }
    }
}

#[derive(Debug, Error, Serialize)]
pub struct ValidationMessageError {
    pub field: String,
    pub message: String,
}

impl Display for ValidationMessageError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "Field {} : {}", self.field, self.message)
    }
}

impl Display for ValidationMessageErrors {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "Field error: {}", self.errors.len())
    }
}

#[derive(Error, Debug)]
pub enum AppError {
    #[error("{0}")]
    NotFound(String),
    #[error("{0}")]
    BadRequest(String),
    #[error("authentication is required to access this resource")]
    Unauthorized,
    #[error("user does not have privilege to access this resource")]
    Forbidden(String),
    #[error("unexpected error has occurred")]
    InternalServerError,
    #[error("{0}")]
    InternalServerErrorWithContext(String),
    #[error("{0}")]
    Conflict(String),
    #[error("{0}")]
    PreconditionFailed(String),
    #[error(transparent)]
    AxumJsonRejection(#[from] JsonRejection),
    #[error(transparent)]
    AxumQueryRejection(#[from] QueryRejection),
    #[error(transparent)]
    AxumPathRejection(#[from] PathRejection),
    #[error(transparent)]
    ValidationErrors(#[from] ValidationErrors),
    #[error(transparent)]
    ValidationError(#[from] ValidationError),
    #[error("unprocessable request has occurred")]
    UnprocessableEntity { errors: ErrorMap },
    #[error(transparent)]
    SerdeJsonError(#[from] serde_json::Error),
    #[error(transparent)]
    AnyhowError(#[from] anyhow::Error),
    #[error("{0}")]
    ConfigError(#[from] std::io::Error),
    #[error("{0}")]
    DbError(#[from] DbErr),
    #[error("{0}")]
    TransactionError(#[from] TransactionError<DbErr>),
    #[error("{0}")]
    DecodeError(#[from] DecodeError),
    #[error("Participant quota exceeded")]
    ParticipantQuotaExceeded,
    #[error("Participant already exists")]
    ParticipantAlreadyExists,

    #[error(transparent)]
    ValidationMessageError(#[from] ValidationMessageError),
    #[error(transparent)]
    ValidationMessageErrors(#[from] ValidationMessageErrors),

    #[error("{0}")]
    SqlxDbError(#[from] sea_orm::sqlx::Error),
    
    #[error("Mismatch project version id")]
    ProjectVersionIdMismatch,
    #[error("Failed to parse variable")]
    FailedParsingVariable,
    #[error(transparent)]
    UuidParseError(#[from] uuid::Error),
}

impl AppError {
    #[must_use]
    /// Maps `validator`'s `ValidationrErrors` to a simple map of property name/error messages structure.
    pub fn unprocessable_entity(errors: ValidationErrors) -> Response {
        let mut validation_errors = ErrorMap::new();

        for (field_property, error_kind) in errors.into_errors() {
            if let ValidationErrorsKind::Field(field_meta) = error_kind.clone() {
                for error in field_meta {
                    validation_errors
                        .entry(Cow::from(field_property))
                        .or_default()
                        .push(error.message.unwrap_or_else(|| {
                            let params: Vec<Cow<'static, str>> = error
                                .params
                                .iter()
                                .filter(|(key, _value)| *key != "value")
                                .map(|(key, value)| Cow::from(format!("{key} value is {value}")))
                                .collect();

                            if params.is_empty() {
                                Cow::from(format!("{field_property} is required"))
                            } else {
                                Cow::from(params.join(", "))
                            }
                        }));
                }
            }
        }

        let body = Json(json!({
            "errors": validation_errors,
        }));

        (StatusCode::BAD_REQUEST, body).into_response()
    }

    #[must_use]
    /// parse validation to list of field and error
    pub fn parse_validation_errors(errors: ValidationMessageErrors) -> Response {
        let body = Json(json!({
            "errors": errors.errors,
        }));

        (StatusCode::BAD_REQUEST, body).into_response()
    }

    #[must_use]
    /// parse validation to list of field and error
    pub fn parse_validation_error(error: ValidationMessageError) -> Response {
        let body = Json(json!({
            "errors": vec![error],
        }));

        (StatusCode::BAD_REQUEST, body).into_response()
    }
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        debug!("{:#?}", self);
        if let Self::ValidationErrors(e) = self {
            return Self::unprocessable_entity(e);
        }

        if let Self::ValidationMessageErrors(e) = self {
            return Self::parse_validation_errors(e);
        }

        if let Self::ValidationMessageError(e) = self {
            return Self::parse_validation_error(e);
        }


        let (status, error_message) = match self {
            Self::InternalServerErrorWithContext(err) => (StatusCode::INTERNAL_SERVER_ERROR, err),
            Self::NotFound(err) => (StatusCode::NOT_FOUND, err),
            Self::Conflict(err) => (StatusCode::CONFLICT, err),
            Self::PreconditionFailed(err) => (StatusCode::PRECONDITION_FAILED, err),
            Self::BadRequest(err) => (StatusCode::BAD_REQUEST, err),
            Self::Unauthorized => (StatusCode::UNAUTHORIZED, Self::Unauthorized.to_string()),
            Self::Forbidden(err) => (StatusCode::FORBIDDEN, err),
            Self::AxumJsonRejection(err) => (StatusCode::BAD_REQUEST, err.body_text()),
            Self::ParticipantAlreadyExists => (StatusCode::BAD_REQUEST, Self::ParticipantAlreadyExists.to_string()),
            Self::ParticipantQuotaExceeded => (StatusCode::BAD_REQUEST, Self::ParticipantQuotaExceeded.to_string()),
            Self::ProjectVersionIdMismatch => (StatusCode::BAD_REQUEST, Self::ProjectVersionIdMismatch.to_string()),
            Self::DbError(err) => (StatusCode::INTERNAL_SERVER_ERROR, err.to_string()),
            Self::FailedParsingVariable => (StatusCode::BAD_REQUEST, Self::FailedParsingVariable.to_string()),
            Self::UuidParseError(err) => (StatusCode::INTERNAL_SERVER_ERROR, err.to_string()),
            _ => (
                StatusCode::INTERNAL_SERVER_ERROR,
                Self::InternalServerError.to_string(),
            ),
        };

        let body = Json(BaseResponse::<()>::error(error_message, Some(status.as_u16())));

        (status, body).into_response()
    }
}
