use async_trait::async_trait;
use futures::{
    future::BoxFuture,
    ready, stream,
    stream::{Stream, StreamExt},
    Future, FutureExt,
};
use pin_project::pin_project;
use std::task::Poll;

pub async fn main() {
    let mut stream = StreamFut::new(&Sleeper);
    let next = stream.next().await;
    dbg!(next);
}

#[async_trait]
pub trait Async {
    async fn run(&self, s: u64) -> bool;
}
struct Sleeper;
#[async_trait]
impl Async for Sleeper {
    async fn run(&self, s: u64) -> bool {
        tokio::time::sleep(tokio::time::Duration::from_secs(s)).await;
        true
    }
}

#[pin_project]
pub struct StreamFut<'a, F, Fut> {
    f: &'a F,
    #[pin]
    fut: Option<Fut>,
}
impl<'a, F, Fut> StreamFut<'a, F, Fut> {
    pub fn new(f: &'a F) -> Self {
        Self { f, fut: None }
    }
}

// This works, because the future returned by f now captures a lifetime
// tied to the StreamFut struct rather than to the anonymous lifetime
// of the Pin<&mut Self> passed to poll_next(). But, we want run() to
// take &mut self
impl<'a, F> Stream for StreamFut<'a, F, BoxFuture<'a, bool>>
where
    F: Async,
{
    type Item = bool;

    fn poll_next(
        self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Option<Self::Item>> {
        let mut this = self.project();
        Poll::Ready(loop {
            if let Some(fut) = this.fut.as_mut().as_pin_mut() {
                let item = ready!(fut.poll(cx));
                this.fut.set(None);
                break Some(item);
            } else {
                this.fut.set(Some( this.f.run(3) ));
            }
        })
    }
}
