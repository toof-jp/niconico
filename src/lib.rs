//! NicoNico login functionality module
//!
//! This module provides functionality to authenticate with the NicoNico video platform
//! and obtain a user session token.
//!
//! # Examples
//!
//! ```
//! use niconico::{login, Credentials};
//! use secrecy::ExposeSecret;
//! 
//! #[tokio::main]
//! async fn main() {
//!     dotenvy::dotenv().ok();
//!     let credentials = envy::from_env::<Credentials>().unwrap();
//! 
//!     let user_session = login(credentials).await.unwrap();
//! 
//!     println!("{:?}", user_session.user_session.expose_secret());
//! }
//! ```

use std::collections::HashMap;

use reqwest::header;
use secrecy::{ExposeSecret, SecretString};
use serde::Deserialize;
use thiserror::Error;

/// Credentials required for NicoNico login
#[derive(Debug, Deserialize)]
pub struct Credentials {
    /// Email address or telephone number associated with the account
    pub mail_tel: String,
    /// Account password, stored securely using SecretString
    pub password: SecretString,
}

/// Represents a successful login session
#[derive(Debug)]
pub struct UserSession {
    /// Session token received from NicoNico's authentication service
    pub user_session: SecretString,
}

/// Possible errors that can occur during the login process
#[derive(Debug, Error)]
pub enum LoginError {
    /// Error occurred while creating the HTTP client
    #[error("Failed to create HTTP client: {0}")]
    ClientError(#[from] reqwest::Error),

    /// Error occurred while parsing HTTP headers
    #[error("Failed to parse cookie header: {0}")]
    HeaderParseError(#[from] reqwest::header::ToStrError),

    /// Required user session cookie was not found in the response
    #[error("User session cookie not found in response")]
    UserSessionNotFound,

    /// Network-related errors during the login request
    #[error("Network error occurred: {0}")]
    NetworkError(String),
}

/// Type alias for the Result of a login attempt
pub type LoginResult = Result<UserSession, LoginError>;

/// Attempts to log in to NicoNico using the provided credentials
///
/// # Arguments
///
/// * `credentials` - The user credentials to use for login
///
/// # Returns
///
/// Returns a `LoginResult` which is either:
/// * `Ok(UserSession)` containing the session token on successful login
/// * `Err(LoginError)` containing the specific error that occurred
///
/// # Examples
///
/// ```
/// let credentials = Credentials {
///     mail_tel: "user@example.com".to_string(),
///     password: "password123".into(),
/// };
///
/// match login(credentials).await {
///     Ok(session) => println!("Login successful!"),
///     Err(e) => eprintln!("Login failed: {}", e),
/// }
/// ```
pub async fn login(credentials: Credentials) -> LoginResult {
    let login_url = "https://account.nicovideo.jp/login/redirector";
    let user_agent = "toof-jp/niconico";

    let mut params = HashMap::new();
    params.insert("password", credentials.password.expose_secret().to_string());
    params.insert("mail_tel", credentials.mail_tel);

    let res = reqwest::Client::builder()
        .redirect(reqwest::redirect::Policy::none())
        .user_agent(user_agent)
        .build()
        .map_err(LoginError::ClientError)?
        .post(login_url)
        .form(&params)
        .send()
        .await
        .map_err(|e| LoginError::NetworkError(e.to_string()))?;

    parse_response_header(res.headers())
}

/// Parses the response headers to extract the user session token
///
/// # Arguments
///
/// * `response_header` - HTTP response headers containing the Set-Cookie header
///
/// # Returns
///
/// Returns a `LoginResult` containing either:
/// * `Ok(UserSession)` with the parsed session token
/// * `Err(LoginError)` if the session token couldn't be found or parsed
///
/// # Note
///
/// The function specifically looks for cookies that start with "user_session=user_session_"
/// as these contain the authentication token.
fn parse_response_header(response_header: &header::HeaderMap) -> LoginResult {
    // There are multiple Set-Cookie headers with the cookie_name 'user_session`
    for header_value in response_header.get_all(header::SET_COOKIE) {
        let cookie_str = header_value.to_str()?;
        if cookie_str.find("user_session=user_session_") == Some(0) {
            return Ok(UserSession {
                user_session: cookie_str.into(),
            });
        }
    }

    Err(LoginError::UserSessionNotFound)
}

#[cfg(test)]
mod tests {
    use super::*;
    use reqwest::header::{HeaderMap, HeaderValue};

    /// Tests successful parsing of a valid user session cookie
    #[test]
    fn test_parse_response_header_success() {
        let mut headers = HeaderMap::new();
        headers.append(
            header::SET_COOKIE,
            HeaderValue::from_static("user_session=user_session_123"),
        );

        let result = parse_response_header(&headers);
        assert!(result.is_ok());
    }

    /// Tests error handling when user session cookie is not present
    #[test]
    fn test_parse_response_header_not_found() {
        let mut headers = HeaderMap::new();
        headers.append(
            header::SET_COOKIE,
            HeaderValue::from_static("different_cookie=value"),
        );

        let result = parse_response_header(&headers);
        assert!(matches!(result, Err(LoginError::UserSessionNotFound)));
    }

    /// Tests error handling for invalid header values
    #[test]
    fn test_parse_response_header_invalid() {
        let mut headers = HeaderMap::new();
        headers.append(
            header::SET_COOKIE,
            HeaderValue::from_bytes(&[0xff]).unwrap(),
        );

        let result = parse_response_header(&headers);
        assert!(matches!(result, Err(LoginError::HeaderParseError(_))));
    }
}
