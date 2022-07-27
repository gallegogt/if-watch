use if_watch::{AsioProvider, Watcher};
use std::pin::Pin;

fn main() {
    env_logger::init();
    futures::executor::block_on(async {
        let mut set = Watcher::<AsioProvider>::new().await.unwrap();
        loop {
            println!("Got event {:?}", Pin::new(&mut set).await);
        }
    });
}
