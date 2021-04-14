use custom_error::custom_error;
use reqwest::Error;
use url::ParseError;

custom_error! {
///! Custom error for Email client error.
pub EmailClientError
    InvalidUri {source:ParseError} = "{source}",
    InvalidRequest {source:Error} = "{source}",
    ErrorResponse {
        canonical_reason:String,
        code:String, is_client_error:bool,
        is_server_error:bool
    } = @ { match (is_client_error,is_server_error) {
        (true,true) => "Both client and server failed because: \
        {canonical_reason} with code: {code}",
        (true,false) => "Client failed because: {canonical_reason} with code: {code}",
        (false,true) => "Server failed because: {canonical_reason} with code: {code}",
        (false,false) => "{canonical_reason} with code: {code}",
     }
    }
}
