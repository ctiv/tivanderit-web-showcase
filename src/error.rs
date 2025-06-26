use leptos::prelude::*;
use serde::{Deserialize, Serialize};
use std::str::FromStr;
use thiserror::Error;
use server_fn::codec::JsonEncoding;
use server_fn::error::{FromServerFnError, ServerFnErrorErr};

#[derive(Error, Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ContactFormError {
    #[error("MissingEmail")]
    MissingEmail,
    #[error("InvalidEmailFormat")]
    InvalidEmailFormat,
    #[error("EmailTooLong")]
    EmailTooLong,
    #[error("MessageTooLong")]
    MessageTooLong,
    #[error("MissingMessage")]
    MissingMessage,
    #[error("TermsNotAccepted")]
    TermsNotAccepted,
    #[error("DatabaseError")]
    DatabaseError(String),
}

impl ContactFormError {
    pub fn get_user_message(&self) -> String {
        match self {
            ContactFormError::MissingEmail => "Ange en email-adress.".to_string(),
            ContactFormError::InvalidEmailFormat => "Ange en giltig email-adress.".to_string(),
            ContactFormError::EmailTooLong => "E-postadressen är för lång (max 254 tecken).".to_string(),
            ContactFormError::MessageTooLong => "Meddelandet är för långt (max 5000 tecken).".to_string(),
            ContactFormError::MissingMessage => "Meddelandet får inte vara tomt.".to_string(),
            ContactFormError::TermsNotAccepted => "Du måste acceptera villkoren för att skicka meddelandet.".to_string(),
            ContactFormError::DatabaseError(s) => format!("Ett serverfel uppstod: {}", s),
        }
    }
}

#[cfg(feature = "ssr")]
impl From<sqlx::Error> for ContactFormError {
    fn from(e: sqlx::Error) -> Self {
        ContactFormError::DatabaseError(e.to_string())
    }
}

impl FromServerFnError for ContactFormError {
    type Encoder = JsonEncoding;
    fn from_server_fn_error(e: ServerFnErrorErr) -> Self {
        match e {
            ServerFnErrorErr::ServerError(s) => Self::DatabaseError(s),
            _ => Self::DatabaseError(e.to_string()),
        }
    }
}

impl From<ServerFnError> for ContactFormError {
    fn from(e: ServerFnError) -> Self {
        match e {
            ServerFnError::ServerError(s) => Self::DatabaseError(s),
            _ => Self::DatabaseError(e.to_string()),
        }
    }
}

impl FromStr for ContactFormError {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "MissingEmail" => Ok(ContactFormError::MissingEmail),
            "InvalidEmailFormat" => Ok(ContactFormError::InvalidEmailFormat),
            "EmailTooLong" => Ok(ContactFormError::EmailTooLong),
            "MessageTooLong" => Ok(ContactFormError::MessageTooLong),
            "MissingMessage" => Ok(ContactFormError::MissingMessage),
            "TermsNotAccepted" => Ok(ContactFormError::TermsNotAccepted),
            "DatabaseError" => Ok(ContactFormError::DatabaseError("Okänt databasfel".to_string())),
            _ => Err(format!("Okänt felvariantnamn kunde inte parsas: {}", s)),
        }
    }
}
