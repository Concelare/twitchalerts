use std::fmt::{Debug, Display, Formatter};

/// Error Struct For Twitch Alerts
///
/// # Parameters
/// * `code` - Error Code Identifier
/// * `description` - Description of the error
///
/// # Error Codes
/// ### Reqwest Codes
/// * `1` - An error occurred due to timing out...
/// * `2` - An error occurred when trying to connect...
/// * `3` - Status returned as an Error...
/// * `4` - An error occurred due to an attempted redirect...
/// * `5` - An error occurred due to the request...
/// * `6` - An error occurred with the request or response body...
/// * `7` - An error occurred with the type builder...
/// * `8` - An unknown error occurred with the request
///
/// ### Tokio Codes
/// * `9` - A Tokio error occurred which resulted in a check being cancelled...
/// * `10` - An error occurred causing the Tokio task to panic...
/// * `11` - An unknown Tokio Error Occurred...
/// # Example
/// ```
///
/// use twitchalerts::error::Error;
///
/// pub async fn main() -> Result<(), Error> {
///     let err: Error = Error::new("An error occurred due to an attempted redirect...", 4);
///     Err(err)
/// }
/// ```
pub struct Error {
    pub code: u16,
    pub description: String,
}

impl Debug for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.description)
    }
}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.description)
    }
}

impl std::error::Error for Error {
    fn description(&self) -> &str {
        &self.description
    }
}

impl Error {
    pub fn new(description: &str, code: u16) -> Error {
        Error {
            code: code.clone(),
            description: description.to_string(),
        }
    }
}
