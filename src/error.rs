use http::StatusCode;
use dom_smoothie::ReadabilityError;

// pub type DateTime = time::OffsetDateTime;
pub type Result<T, E = Error> = std::result::Result<T, E>;

#[derive(Debug, thiserror::Error)]
pub enum JsError {
  #[error("{0}")]
  Message(String),

  #[error("Exception: {0}")]
  Exception(crate::js::Exception),

  #[error("{0}")]
  Error(#[from] rquickjs::Error),
}

#[derive(Debug, thiserror::Error)]
pub enum ConfigError {
  #[error("Bad selector: {0}")]
  BadSelector(String),

  #[error("YAML parse error: {0}")]
  Yaml(#[from] serde_yaml::Error),

  #[error("Regex error: {0}")]
  Regex(#[from] regex::Error),

  #[error("Invalid URL {0}")]
  InvalidUrl(#[from] url::ParseError),

  #[error("IO error: {0}")]
  Io(#[from] std::io::Error),

  #[error("Reqwest client error: {0}")]
  Reqwest(#[from] reqwest::Error),

  #[error("Js runtime initialization error: {0}")]
  Js(#[from] JsError),

  #[error("Client config error - bad header value: {0}")]
  ClientHeader(#[from] reqwest::header::InvalidHeaderValue),

  #[error("Duplicate endpoint: {0}")]
  DuplicateEndpoint(String),

  #[error("Bad source template: {0}")]
  BadSourceTemplate(String),

  #[error("{0}")]
  Message(String),
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
  #[error("IO error: {0}")]
  Io(#[from] std::io::Error),

  #[error("HTTP error: {0}")]
  Http(#[from] http::Error),

  #[error("Axum error: {0}")]
  Axum(#[from] axum::Error),

  #[error("RSS feed error: {0}")]
  Rss(#[from] rss::Error),

  #[error("Atom feed error: {0}")]
  Atom(#[from] atom_syndication::Error),

  #[error("Invalid URL: {0}")]
  InvalidUrl(#[from] url::ParseError),

  #[error("Feed parsing error: {0}")]
  FeedParse(&'static str),

  #[error("Feed merge error: {0}")]
  FeedMerge(&'static str),

  #[error("Reqwest client error: {0}")]
  Reqwest(#[from] reqwest::Error),

  #[error("HTTP status error {0} (url: {1})")]
  HttpStatus(reqwest::StatusCode, url::Url),

  #[error("Js runtime error: {0}")]
  Js(#[from] JsError),

  #[error("Failed to extract webpage: {0}")]
  Readability(#[from] ReadabilityError),

  #[error("Config error: {0}")]
  Config(#[from] ConfigError),

  #[error("Tokio task join error: {0}")]
  Join(#[from] tokio::task::JoinError),

  #[error("Endpoint not found: {0}")]
  EndpointNotFound(String),

  #[error("Unsupported feed format: {0}")]
  UnsupportedFeedFormat(String),

  #[error("Failed fetching source: {0}")]
  FetchSource(Box<Error>),

  #[error("Source URL unspecified for dynamic source")]
  DynamicSourceUnspecified,

  #[error("Source parameter {placeholder} failed to match validation: {validation} (input: {input})")]
  SourceTemplateValidation {
    placeholder: String,
    validation: String,
    input: String,
  },

  #[error("Source template placeholder unspecified: {0}")]
  MissingSourceTemplatePlaceholder(String),

  #[error("Can't infer app base, please refer to https://github.com/shouya/rss-funnel/wiki/App-base")]
  BaseUrlNotInferred,

  #[error("{0}")]
  Message(String),
}

impl Error {
  pub fn into_http(self) -> (StatusCode, String) {
    match self {
      Error::FetchSource(e) => {
        let (status, body) = e.into_http();
        (status, format!("Error fetching source: {body}"))
      }
      Error::HttpStatus(status, url) => {
        let body = format!("Error requesting {url}: {status}");
        (StatusCode::BAD_GATEWAY, body)
      }
      Error::SourceTemplateValidation { .. }
      | Error::MissingSourceTemplatePlaceholder(_) => {
        (StatusCode::BAD_REQUEST, format!("{self}"))
      }
      _ => (StatusCode::INTERNAL_SERVER_ERROR, format!("{self}")),
    }
  }
}
