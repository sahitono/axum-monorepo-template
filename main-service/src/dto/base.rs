use serde::Serialize;

#[derive(Serialize)]
pub struct BaseResponse<T = ()> {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<T>, // Data payload for success
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<ErrorDetails>, // Error details for failures
}

#[derive(Serialize)]
pub struct ErrorDetails {
    pub message: String, // Error message
    pub code: Option<u16>, // Optional error code
}

impl<T> BaseResponse<T> {
    pub fn success(data: T) -> Self {
        BaseResponse {
            data: Some(data),
            error: None,
        }
    }

    pub fn error(message: String, code: Option<u16>) -> Self {
        BaseResponse {
            data: None,
            error: Some(ErrorDetails { message, code }),
        }
    }
}
