use custom_error::custom_error;

custom_error! {
///! Custom error for missing env variable or invalid configuration files.
pub MalformedInput
    InvalidName{message:String} = "{message}",
    InvalidEmail{message:String} = "{message}",
}
