use rex::error::{LayoutError, FontError};

#[derive(Debug,)]
pub enum AppError {
    ParseError(String),
    IOError(std::io::Error),
    FontError(FontError),
    LayoutError(LayoutError),
}

impl AppError {
    pub fn human_readable(&self) -> String {
        let error_tag = match self {
            AppError::FontError(_) |
            AppError::LayoutError(LayoutError::Font(_)) => "Font Error",
            AppError::ParseError(_) => "Parse Error",

            _ => "App-internal Error",
        };

        let error_message = match self {
            AppError::ParseError(e)  => format!("{}", e),
            AppError::IOError(e)     => format!("{}", e),
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


impl From<FontError> for AppError {
    fn from(err: FontError) -> Self 
    { Self::FontError(err) }
}

impl From<LayoutError> for AppError {
    fn from(err: LayoutError) -> Self 
    { Self::LayoutError(err) }
}