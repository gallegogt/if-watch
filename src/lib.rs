//! IP address watching.
#![deny(missing_docs)]
#![deny(warnings)]

use futures::future::BoxFuture;
pub use ipnet::{IpNet, Ipv4Net, Ipv6Net};
use std::{
    collections::hash_set::Iter,
    future::Future,
    io::Result,
    marker::Unpin,
    pin::Pin,
    task::{Context, Poll},
};

#[cfg(target_os = "macos")]
mod apple;
#[cfg(target_os = "ios")]
mod apple;
#[cfg(not(any(
    target_os = "ios",
    target_os = "linux",
    target_os = "macos",
    target_os = "windows",
)))]
mod fallback;
#[cfg(target_os = "linux")]
mod linux;
#[cfg(target_os = "windows")]
mod win;

#[cfg(target_os = "macos")]
use apple as platform_impl;
#[cfg(target_os = "ios")]
use apple as platform_impl;
#[cfg(not(any(
    target_os = "ios",
    target_os = "linux",
    target_os = "macos",
    target_os = "windows",
)))]
use fallback as platform_impl;

#[cfg(target_os = "linux")]
use linux as platform_impl;

#[cfg(target_os = "windows")]
use win as platform_impl;

#[cfg(feature = "async-io")]
/// Async-Io IfWatcher
pub type AsioProvider = platform_impl::AsioWatcher;

#[cfg(feature = "tokio")]
/// Tokio IfWatcher
pub type TokioProvider = platform_impl::TokioWatcher;

/// An address change event.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum IfEvent {
    /// A new local address has been added.
    Up(IpNet),
    /// A local address has been deleted.
    Down(IpNet),
}

/// Iface
pub trait Iface: Send {
    /// Init
    fn init() -> BoxFuture<'static, Result<Self>>
    where
        Self: Sized;

    /// Iterate over current networks.
    fn networks(&self) -> Iter<IpNet>;
}

/// Watches for interface changes.
#[derive(Debug)]
pub struct Watcher<T>(T);

impl<T> Watcher<T>
where
    T: Iface,
{
    /// Create a watcher
    pub async fn new() -> Result<Self> {
        Ok(Self(T::init().await?))
    }

    /// Iterate over current networks.
    pub fn networks(&self) -> impl Iterator<Item = &IpNet> {
        self.0.networks()
    }
}

impl<T> Future for Watcher<T>
where
    T: Unpin + Future<Output = Result<IfEvent>>,
{
    type Output = Result<IfEvent>;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context) -> Poll<Self::Output> {
        Pin::new(&mut self.0).poll(cx)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::pin::Pin;

    #[cfg(feature = "tokio")]
    #[tokio::test]
    async fn test_ip_watch() {
        let mut set = TokioProvider::init().await.unwrap();
        let event = Pin::new(&mut set).await.unwrap();
        println!("Got event {:?}", event);
    }

    #[cfg(feature = "tokio")]
    #[tokio::test]
    async fn test_is_send() {
        fn is_send<T: Send>(_: T) {}
        is_send(TokioProvider::init());
        is_send(TokioProvider::init().await.unwrap());
        is_send(Pin::new(&mut TokioProvider::init().await.unwrap()));
    }

    #[cfg(feature = "async-io")]
    #[test]
    fn async_io_test_ip_watch() {
        futures::executor::block_on(async {
            let mut set = AsioProvider::init().await.unwrap();
            let event = Pin::new(&mut set).await.unwrap();
            println!("Got event {:?}", event);
        });
    }

    #[cfg(feature = "async-io")]
    #[test]
    fn async_io_test_is_send() {
        futures::executor::block_on(async {
            fn is_send<T: Send>(_: T) {}
            is_send(AsioProvider::init());
            is_send(AsioProvider::init().await.unwrap());
            is_send(Pin::new(&mut AsioProvider::init().await.unwrap()));
        });
    }
}
