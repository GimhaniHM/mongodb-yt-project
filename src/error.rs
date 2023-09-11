//seperate error handling file ->->->  for handling error more elegantly

use serde::Serialize;
use thiserror::Error;  //use as a main library to handle the errors
use warp::{http::StatusCode, reply, Rejection, Reply};
use std::convert::Infallible;
use mongodb::bson;


// Define a custom error enum using the thiserror crate
#[derive(Error, Debug)]
pub enum Error{
    #[error("mongodb error:{0}")]
    MongoError(#[from] mongodb::error::Error),
    #[error("error during mongodb query: {0}")]
    MongoQueryError(mongodb::error::Error),
    #[error("could not access field in document: {0}")]
    MongoDataError(#[from] bson::document::ValueAccessError),
    #[error("Invalid id used:{0}")]
    InvalidIDError(String),

}

// Define a struct for JSON error responses
#[derive(Serialize)]
struct ErrorResponse {
    message: String,
}

// Implement the Reject trait for the custom Error enum
impl warp::reject::Reject for Error {}

// Define a function to handle rejections (errors) and convert them into HTTP responses
pub async fn handle_rejection(err: Rejection) -> std::result::Result<Box<dyn Reply>, Infallible> {  //successful result is a boxed trait object  //  indicates that the function never returns an error in the success case.
    let code;
    let message;

    if err.is_not_found() {
        code = StatusCode::NOT_FOUND;
        message = "Not Found";
    } else if let Some(_) = err.find::<warp::filters::body::BodyDeserializeError>() {  //use to find the specific type of error
        code = StatusCode::BAD_REQUEST;
        message = "Invalid Body";
    } else if let Some(e) = err.find::<Error>() {
        match e {
            _ => {
                eprintln!("unhandled application error: {:?}", err);
                code = StatusCode::INTERNAL_SERVER_ERROR;
                message = "Internal Server Error";
            }
        }
    } else if let Some(_) = err.find::<warp::reject::MethodNotAllowed>() {      //some variant of an 'Option' 
        code = StatusCode::METHOD_NOT_ALLOWED;
        message = "Method Not Allowed";
    } else {
        eprintln!("unhandled error: {:?}", err);
        code = StatusCode::INTERNAL_SERVER_ERROR;
        message = "Internal Server Error";
    }

    //cerate ressponse json containing error message
    let json = reply::json(&ErrorResponse {
        message: message.into(),
    });

    //return an HTTP response with the appropriate status code and JSON bod
    Ok(Box::new(reply::with_status(json, code)))
}




