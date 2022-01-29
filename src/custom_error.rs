use hyper::{header::ToStrError, http::uri::InvalidUri};
use log::error;
use rocket::{
    http::Status,
    response::{self, Responder},
    Request,
};

pub struct CustomError {
    msg: String,
}

impl<'r, 'o: 'r> Responder<'r, 'o> for CustomError {
    fn respond_to(self, _: &'r Request<'_>) -> response::Result<'o> {
        error!("{}", self.msg);
        Err(Status::InternalServerError)
    }
}

impl std::convert::From<anyhow::Error> for CustomError {
    fn from(e: anyhow::Error) -> Self {
        CustomError {
            msg: format!("{}", e),
        }
    }
}

impl std::convert::From<uuid::Error> for CustomError {
    fn from(e: uuid::Error) -> Self {
        CustomError {
            msg: format!("{}", e),
        }
    }
}

impl std::convert::From<ToStrError> for CustomError {
    fn from(e: ToStrError) -> Self {
        CustomError {
            msg: format!("{}", e),
        }
    }
}

impl std::convert::From<InvalidUri> for CustomError {
    fn from(e: InvalidUri) -> Self {
        CustomError {
            msg: format!("{}", e),
        }
    }
}

impl std::convert::From<hyper::Error> for CustomError {
    fn from(e: hyper::Error) -> Self {
        CustomError {
            msg: format!("{}", e),
        }
    }
}
