//! Tools for building a transparent TCP/HTTP proxy.

pub use linkerd2_proxy_api_resolve as api_resolve;
pub use linkerd2_proxy_core as core;
pub use linkerd2_proxy_discover as discover;
pub use linkerd2_proxy_http::{
    self as http,
    // TODO(eliza): port
    // grpc
};
pub use linkerd2_proxy_identity as identity;
pub use linkerd2_proxy_resolve as resolve;
pub use linkerd2_proxy_tap as tap;
pub use linkerd2_proxy_tcp as tcp;

mod server;
mod skip_detect;

pub use self::server::DetectHttp;
pub use self::skip_detect::SkipDetect;
