use if_watch::{TokioProvider, Watcher};
use std::pin::Pin;

#[tokio::main(flavor = "current_thread")]
async fn main() {
    env_logger::init();
    let mut set = Watcher::<TokioProvider>::new().await.unwrap();
    loop {
        println!("Got event {:?}", Pin::new(&mut set).await);
    }
}
