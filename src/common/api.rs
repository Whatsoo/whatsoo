use std::borrow::Cow;
use std::convert::Infallible;
use std::fmt::{self, Debug, Display};

use axum::body::Full;
use axum::http::{header, HeaderValue, Response, StatusCode};
use axum::response::IntoResponse;
use bytes::Bytes;
use serde::Serialize;

use crate::AppResult;

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct ApiResult<T: Serialize> {
    code: Option<u16>,
    msg: Option<Cow<'static, str>>,
    data: Option<T>,
}

impl<T: Serialize> ApiResult<T> {
    pub fn new() -> Self {
        Self {
            code: None,
            msg: None,
            data: None,
        }
    }

    pub fn ok() -> Self {
        Self {
            code: Some(200u16),
            msg: None,
            data: None,
        }
    }

    pub fn error() -> Self {
        Self {
            code: Some(500u16),
            msg: None,
            data: None,
        }
    }

    pub fn code(mut self, code: StatusCode) -> Self {
        self.code = Some(code.as_u16());
        self
    }

    pub fn msg<S: Into<Cow<'static, str>>>(mut self, msg: S) -> Self {
        self.msg = Some(msg.into());
        self
    }

    pub fn data(mut self, data: T) -> Self {
        self.data = Some(data);
        self
    }
}

impl<T: Debug + Serialize> Display for ApiResult<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl<T: Serialize> Into<AppResult<ApiResult<T>>> for ApiResult<T> {
    fn into(self) -> AppResult<ApiResult<T>> {
        Ok(self)
    }
}

impl<T: Serialize> IntoResponse for ApiResult<T> {
    type Body = Full<Bytes>;
    type BodyError = Infallible;

    fn into_response(self) -> Response<Self::Body> {
        let bytes = match serde_json::to_vec(&self) {
            Ok(res) => res,
            Err(err) => {
                return Response::builder()
                    .status(StatusCode::INTERNAL_SERVER_ERROR)
                    .header(header::CONTENT_TYPE, "text/plain")
                    .body(Full::from(err.to_string()))
                    .unwrap();
            }
        };

        let mut res = Response::new(Full::from(bytes));
        res.headers_mut()
            .insert(header::CONTENT_TYPE, HeaderValue::from_static("application/json"));
        res
    }
}
