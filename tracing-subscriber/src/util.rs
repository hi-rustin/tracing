//! Extension traits and other utilities to make working with subscribers more
//! ergonomic.
use std::{error::Error, fmt};
use tracing_core::dispatcher::{self, Dispatch};

/// Extension trait adding utility methods for subscriber initialization.
///
/// This trait provides extension methods to make configuring and setting a
/// [default subscriber] more ergonomic. It is automatically implemented for all
/// types that can be converted into a [trace dispatcher]. Since `Dispatch`
/// implements `From<T>` for all `T: Subscriber`, all `Subscriber`
/// implementations will implement this extension trait as well. Types which
/// can be converted into `Subscriber`s, such as builders that construct a
/// `Subscriber`, may implement `Into<Dispatch>`, and will also receive an
/// implementation of this trait.
///
/// [default subscriber]: https://docs.rs/tracing/0.1.20/tracing/dispatcher/index.html#setting-the-default-subscriber
/// [trace dispatcher]: https://docs.rs/tracing/0.1.20/tracing/dispatcher/index.html
pub trait SubscriberInitExt
where
    Self: Into<Dispatch>,
{
    /// Sets `self` as the [default subscriber] in the current scope, returning a
    /// guard that will unset it when dropped.
    ///
    /// If the "tracing-log" feature flag is enabled, this will also initialize
    /// a [`log`] compatibility layer. This allows the subscriber to consume
    /// `log::Record`s as though they were `tracing` `Event`s.
    ///
    /// [default subscriber]: https://docs.rs/tracing/0.1.20/tracing/dispatcher/index.html#setting-the-default-subscriber
    /// [`log`]: https://crates.io/log
    fn set_default(self) -> dispatcher::DefaultGuard {
        #[cfg(feature = "tracing-log")]
        let _ = tracing_log::LogTracer::init();

        dispatcher::set_default(&self.into())
    }

    /// Attempts to set `self` as the [global default subscriber] in the current
    /// scope, returning an error if one is already set.
    ///
    /// If the "tracing-log" feature flag is enabled, this will also attempt to
    /// initialize a [`log`] compatibility layer. This allows the subscriber to
    /// consume `log::Record`s as though they were `tracing` `Event`s.
    ///
    /// This method returns an error if a global default subscriber has already
    /// been set, or if a `log` logger has already been set (when the
    /// "tracing-log" feature is enabled).
    ///
    /// [global default subscriber]: https://docs.rs/tracing/0.1.20/tracing/dispatcher/index.html#setting-the-default-subscriber
    /// [`log`]: https://crates.io/log
    fn try_init(self) -> Result<(), TryInitError> {
        #[cfg(feature = "tracing-log")]
        tracing_log::LogTracer::init().map_err(TryInitError::new)?;

        dispatcher::set_global_default(self.into()).map_err(TryInitError::new)?;

        Ok(())
    }

    /// Attempts to set `self` as the [global default subscriber] in the current
    /// scope, panicking if this fails.
    ///
    /// If the "tracing-log" feature flag is enabled, this will also attempt to
    /// initialize a [`log`] compatibility layer. This allows the subscriber to
    /// consume `log::Record`s as though they were `tracing` `Event`s.
    ///
    /// This method panics if a global default subscriber has already been set,
    /// or if a `log` logger has already been set (when the "tracing-log"
    /// feature is enabled).
    ///
    /// [global default subscriber]: https://docs.rs/tracing/0.1.20/tracing/dispatcher/index.html#setting-the-default-subscriber
    /// [`log`]: https://crates.io/log
    fn init(self) {
        self.try_init()
            .expect("failed to set global default subscriber")
    }
}

impl<T> SubscriberInitExt for T where T: Into<Dispatch> {}

/// Error returned by [`try_init`](SubscriberInitExt::try_init) if a global default subscriber could not be initialized.
pub struct TryInitError {
    inner: Box<dyn Error + Send + Sync + 'static>,
}

// ==== impl TryInitError ====

impl TryInitError {
    fn new(e: impl Into<Box<dyn Error + Send + Sync + 'static>>) -> Self {
        Self { inner: e.into() }
    }
}

impl fmt::Debug for TryInitError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Debug::fmt(&self.inner, f)
    }
}

impl fmt::Display for TryInitError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(&self.inner, f)
    }
}

impl Error for TryInitError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        self.inner.source()
    }
}
