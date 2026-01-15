#[cfg(all(
    any(feature = "http", feature = "gateway"),
    not(any(feature = "rustls_backend", feature = "native_tls_backend"))
))]
compile_error!(
    "You have the `http` or `gateway` feature enabled, either the `rustls_backend` or \
    `native_tls_backend` feature must be selected to let Serenity use `http` or `gateway`.\n\
    - `rustls_backend` uses Rustls, a pure Rust TLS-implemenation.\n\
    - `native_tls_backend` uses SChannel on Windows, Secure Transport on macOS, and OpenSSL on \
    other platforms.\n\
    If you are unsure, go with `rustls_backend`."
);

#[cfg(all(target_arch = "wasm32", feature = "multipart"))]
compile_error!(
    "Multipart file uploads are not supported in WASM builds. \
    Use text/JSON payloads only, or deploy to a native platform for file upload support."
);

#[cfg(all(target_arch = "wasm32", feature = "gateway"))]
compile_error!(
    "Gateway (WebSocket) is not supported in WASM builds. \
    Cloudflare Workers cannot maintain persistent WebSocket connections. \
    Use REST API, Interactions (Slash Commands), or Webhooks instead."
);

#[cfg(all(target_arch = "wasm32", feature = "client"))]
compile_error!(
    "Client feature is not supported in WASM builds. \
    The client uses tokio runtime and arbitrary task spawning, which are not compatible with Cloudflare Workers. \
    Use the HTTP client directly for REST API operations."
);

fn main() {
    println!("cargo:rustc-check-cfg=cfg(tokio_unstable, ignore_serenity_deprecated)");
}
