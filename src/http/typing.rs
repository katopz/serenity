use std::sync::Arc;

#[cfg(all(feature = "http", target_arch = "wasm32"))]
compile_error!(
    "Typing indicators are not supported in WASM builds. \
    Cloudflare Workers cannot maintain long-running background tasks. \
    Use Discord Interactions (Slash Commands) or Webhooks instead."
);

#[cfg(not(target_arch = "wasm32"))]
use tokio::time::{sleep, Duration};

use crate::http::Http;
#[cfg(not(target_arch = "wasm32"))]
use crate::internal::async_runtime::spawn_named;
use crate::internal::prelude::*;
use crate::internal::sync::oneshot;
use crate::model::id::ChannelId;

/// A struct to start typing in a [`Channel`] for an indefinite period of time.
///
/// It indicates that the current user is currently typing in the channel.
///
/// Typing is started by using the [`Typing::start`] method and stopped by using the
/// [`Typing::stop`] method. Note that on some clients, typing may persist for a few seconds after
/// [`Typing::stop`] is called. Typing is also stopped when the struct is dropped.
///
/// If a message is sent while typing is triggered, the user will stop typing for a brief period of
/// time and then resume again until either [`Typing::stop`] is called or the struct is dropped.
///
/// This should rarely be used for bots, although it is a good indicator that a long-running
/// command is still being processed.
///
/// ## Examples
///
/// ```rust,no_run
/// # use serenity::{http::{Http, Typing}, Result, model::prelude::*};
/// # use std::sync::Arc;
/// #
/// # fn long_process() {}
/// # fn main() {
/// # let http: Http = unimplemented!();
/// let channel_id = ChannelId::new(7);
/// // Initiate typing (assuming `http` is bound)
/// let typing = Typing::start(Arc::new(http), channel_id);
///
/// // Run some long-running process
/// long_process();
///
/// // Stop typing
/// typing.stop();
/// # }
/// ```
///
/// [`Channel`]: crate::model::channel::Channel
#[derive(Debug)]
pub struct Typing(oneshot::Sender<()>);

impl Typing {
    /// Starts typing in the specified [`Channel`] for an indefinite period of time.
    ///
    /// Returns [`Typing`]. To stop typing, you must call the [`Typing::stop`] method on the
    /// returned [`Typing`] object or wait for it to be dropped. Note that on some clients, typing
    /// may persist for a few seconds after stopped.
    ///
    /// # Errors
    ///
    /// Returns an  [`Error::Http`] if there is an error.
    ///
    /// [`Channel`]: crate::model::channel::Channel
    #[cfg(not(feature = "wasm"))]
    pub fn start(http: Arc<Http>, channel_id: ChannelId) -> Self {
        let (sx, mut rx) = oneshot::channel();

        spawn_named::<_, Result<_>>("typing::start", async move {
            loop {
                tokio::select! {
                    _ = &mut rx => break,
                    _ = sleep(Duration::from_secs(7)) => {
                        http.broadcast_typing(channel_id).await?;
                    }
                }
            }

            Ok(())
        });

        Self(sx)
    }

    /// Compile-time assertion that Typing is not supported in WASM.
    ///
    /// Cloudflare Workers cannot support long-running background tasks like typing indicators,
    /// which require sending HTTP requests every 7 seconds indefinitely.
    ///
    /// Use Discord Interactions (Slash Commands) or Webhooks instead, which work within the
    /// request-response cycle of Workers.
    #[cfg(feature = "wasm")]
    pub fn start(_http: Arc<Http>, _channel_id: ChannelId) -> Self {
        compile_error!(
            "Typing indicators are not supported in WASM builds. \
            Cloudflare Workers cannot maintain long-running background tasks. \
            Use Discord Interactions (Slash Commands) or Webhooks instead."
        );
        unreachable!("compile_error above prevents this from being called")
    }

    /// Stops typing in [`Channel`].
    ///
    /// This should be used to stop typing after it is started using [`Typing::start`]. Typing may
    /// persist for a few seconds on some clients after this is called. Returns false if typing has
    /// already stopped.
    ///
    /// [`Channel`]: crate::model::channel::Channel
    #[allow(clippy::must_use_candidate)]
    pub fn stop(self) -> bool {
        self.0.send(()).is_ok()
    }
}
