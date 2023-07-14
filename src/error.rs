use rex::error::{LayoutError, FontError, ParseError};
use ttf_parser::FaceParsingError;

#[derive(Debug,)]
pub enum AppError {
    ParseError(String),
    IOError(std::io::Error),
    CairoError(cairo::Error),
    FontError(FontError),
    LayoutError(LayoutError),
    FaceParsingError(FaceParsingError),
}

impl AppError {
    pub fn human_readable(&self) -> String {
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
            AppError::CairoError(e)  => format!("{}", e),
            AppError::FaceParsingError(e) => format!("{}", e),
            AppError::FontError(e)   |
            AppError::LayoutError(LayoutError::Font(e)) => format!("{}", e),
        };

        format!("{} : {}", error_tag, error_message)
    }
}

pub type AppResult<A> = Result<A, AppError>;

impl From<std::io::Error> for AppError {
    fn from(err: std::io::Error) -> Self 
    { Self::IOError(err) }
}

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

impl<'a> From<ParseError<'a>> for AppError {
    fn from(err: ParseError) -> Self 
    { Self::ParseError(err.to_string()) }
}