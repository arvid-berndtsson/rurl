pub mod args;
pub mod client;
pub mod http;
pub mod http2;
pub mod response;
pub mod utils;

// Re-export main types for easy access
pub use args::Args;
pub use client::{send_request, RequestError};
pub use http::build_http_request;
pub use http2::build_http2_request;
pub use response::process_response;
