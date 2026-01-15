fn main() {
    let is_wasm = std::env::var("CARGO_CFG_TARGET_ARCH").as_deref() == Ok("wasm32");
    let _features = std::env::var("CARGO_FEATURES").unwrap_or_default();
    let has_http = std::env::var("CARGO_FEATURE_HTTP").is_ok();
    let has_gateway = std::env::var("CARGO_FEATURE_GATEWAY").is_ok();
    let has_rustls_backend = std::env::var("CARGO_FEATURE_RUSTLS_BACKEND").is_ok();
    let has_native_tls_backend = std::env::var("CARGO_FEATURE_NATIVE_TLS_BACKEND").is_ok();
    let has_reqwest_wasm = std::env::var("CARGO_FEATURE_REQWEST_WASM").is_ok();
    let has_multipart = std::env::var("CARGO_FEATURE_MULTIPART").is_ok();
    let has_client = std::env::var("CARGO_FEATURE_CLIENT").is_ok();

    // Check for required TLS backend when http or gateway features are enabled
    if (has_http || has_gateway)
        && !has_rustls_backend
        && !has_native_tls_backend
        && !(is_wasm && has_reqwest_wasm)
    {
        eprintln!(
            "You have the `http` or `gateway` feature enabled, either the `rustls_backend` or \
            `native_tls_backend` feature must be selected to let Serenity use `http` or `gateway`.\n\
            - `rustls_backend` uses Rustls, a pure Rust TLS-implementation.\n\
            - `native_tls_backend` uses SChannel on Windows, Secure Transport on macOS, and OpenSSL on \
            other platforms.\n\
            - For WASM targets, use `reqwest-wasm` feature.\n\
            If you are unsure, go with `rustls_backend`."
        );
        std::process::exit(1);
    }

    // WASM-specific checks
    if is_wasm {
        if has_multipart {
            eprintln!(
                "Multipart file uploads are not supported in WASM builds. \
                Use text/JSON payloads only, or deploy to a native platform for file upload support."
            );
            std::process::exit(1);
        }

        if has_gateway {
            eprintln!(
                "Gateway (WebSocket) is not supported in WASM builds. \
                Cloudflare Workers cannot maintain persistent WebSocket connections. \
                Use REST API, Interactions (Slash Commands), or Webhooks instead."
            );
            std::process::exit(1);
        }

        if has_client {
            eprintln!(
                "Client feature is not supported in WASM builds. \
                The client uses tokio runtime and arbitrary task spawning, which are not compatible with Cloudflare Workers. \
                Use the HTTP client directly for REST API operations."
            );
            std::process::exit(1);
        }
    }

    println!("cargo:rustc-check-cfg=cfg(tokio_unstable, ignore_serenity_deprecated)");
}
