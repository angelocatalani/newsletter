use custom_error::custom_error;

custom_error! {
///! Custom error for missing env variable or invalid configuration files.
pub MalformedInput
    InvalidName{name:String} = "{name}",
    InvalidEmail{email:String} = "{email}",
}
