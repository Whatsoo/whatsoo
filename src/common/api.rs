use std::borrow::Cow;
use actix_web::http::StatusCode;
use serde::Serialize;
use std::fmt::{self, Debug, Display};
use actix_web::{ HttpResponse, Responder, Error, HttpRequest};
use futures::future::{ready, Ready};

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct ApiResult<T = ()> {
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

// impl<T: Debug + Serialize> ResponseError for ApiResult<T> {
//     fn status_code(&self) -> StatusCode {
//         StatusCode::OK
//     }
//     fn error_response(&self) -> HttpResponse {
//         self.to_resp()
//     }
// }

impl<T: Serialize> Responder for ApiResult<T> {
    type Error = Error;
    type Future = Ready<Result<HttpResponse, Error>>;

    fn respond_to(self, _req: &HttpRequest) -> Self::Future {
        let body = serde_json::to_string(&self).unwrap();

        // Create response and set content type
        ready(Ok(HttpResponse::Ok()
            .content_type("application/json")
            .body(body)))
    }
}
