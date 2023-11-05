//! クレートエラーの定義
use thiserror::Error as DeriveError;

#[derive(DeriveError, Debug)]
pub enum Error {
    #[error("alsa error")]
    Alsa(#[from] alsa::Error),
}

pub type Result<T> = ::std::result::Result<T, Error>;
