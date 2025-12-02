use std::fmt::Display;

use rex::{error::{FontError, LayoutError}, parser::error::ParseError};
use ttf_parser::FaceParsingError;




#[derive(Debug,)]
pub enum AppError {
    ParseError(String),
    IOError(std::io::Error),
    #[cfg(not(target_arch = "wasm32"))]
    CairoError(cairo::Error),
    FontError(FontError),
    LayoutError(LayoutError),
    FaceParsingError(FaceParsingError),
}

impl Display for AppError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        #[allow(unreachable_patterns)]
        let error_tag = match self {
            AppError::FontError(_) |
            AppError::FaceParsingError(_) |
            AppError::LayoutError(LayoutError::Font(_)) => "Font Error",
            AppError::ParseError(_) => "Parse Error",
            AppError::IOError(_) => "IO Error",

            _ => "App-internal Error",
        };

        let error_message = match self {
            AppError::ParseError(e)  => format!("{}", e),
            AppError::IOError(e)     => format!("{}", e),
            #[cfg(not(target_arch = "wasm32"))]
            AppError::CairoError(e)  => format!("{}", e),
            AppError::FaceParsingError(e) => format!("{}", e),
            AppError::FontError(e)   |
            AppError::LayoutError(LayoutError::Font(e)) => format!("{}", e),
        };

        write!(f, "{} : {}", error_tag, error_message)
    }
}

pub type AppResult<A> = Result<A, AppError>;

impl From<std::io::Error> for AppError {
    fn from(err: std::io::Error) -> Self 
    { Self::IOError(err) }
}


#[cfg(not(target_arch = "wasm32"))]
impl From<cairo::Error> for AppError {
    fn from(err: cairo::Error) -> Self 
    { Self::CairoError(err) }
}

impl From<FontError> for AppError {
    fn from(err: FontError) -> Self 
    { Self::FontError(err) }
}

impl From<LayoutError> for AppError {
    fn from(err: LayoutError) -> Self 
    { Self::LayoutError(err) }
}

impl From<FaceParsingError> for AppError {
    fn from(err: FaceParsingError) -> Self 
    { Self::FaceParsingError(err) }
}

impl From<ParseError> for AppError {
    fn from(err: ParseError) -> Self 
    { Self::ParseError(err.to_string()) }
}