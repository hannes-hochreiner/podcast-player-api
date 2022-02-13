use hyper::{header::ToStrError, http::uri::InvalidUri};

pub struct CustomError {
    msg: String,
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
