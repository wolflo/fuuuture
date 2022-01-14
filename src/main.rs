use futures::{
    future::BoxFuture,
    ready, stream,
    stream::{Stream, StreamExt},
    Future, FutureExt,
};
use std::task::Poll;

#[tokio::main]
async fn main() {
    let stream = StreamFut::new();
    let mut stream = WrapStream::new(stream);
    let next = stream.next().await;
    dbg!(next);
}

async fn sleep(s: u64) -> u64 {
    tokio::time::sleep(tokio::time::Duration::from_secs(s)).await;
    s
}
#[pin_project]
pub struct StreamFut<Fut, F> {
    #[pin]
    fut: Option<Fut>,
    f: F,
}
impl StreamFut<BoxFuture<'static, u64>, fn(u64) -> BoxFuture<'static, u64>> {
    pub fn new() -> Self {
        Self {
            fut: None,
            f: |s| Box::pin(sleep(s)),
        }
    }
}
impl<Fut, F> Stream for StreamFut<Fut, F> where Fut: Future, F: Fn(u64) -> Fut {
    type Item = Fut::Output;

    fn poll_next(self: std::pin::Pin<&mut Self>, cx: &mut std::task::Context<'_>) -> Poll<Option<Self::Item>> {
        let mut this = self.project();
        println!("StreamFut polled.");
        Poll::Ready(loop {
            if let Some(fut) = this.fut.as_mut().as_pin_mut() {
                println!("Polling StreamFut's fut.");
                let item = ready!(fut.poll(cx));
                this.fut.set(None);
                break Some(item)
            } else {
                println!("Storing StreamFut's fut.");
                this.fut.set(Some((this.f)(3)));
            }
        })
    }
}
use pin_project::pin_project;
#[pin_project]
struct WrapStream<S> {
    #[pin]
    stream: S,
}
impl<S> WrapStream<S> {
    pub fn new(stream: S) -> Self {
        Self { stream}
    }
}
impl<S> Stream for WrapStream<S> where S: Stream {
    type Item = S::Item;
    fn poll_next(self: std::pin::Pin<&mut Self>, cx: &mut std::task::Context<'_>) -> Poll<Option<Self::Item>> {
        println!("WrapStream polled.");
        let mut this = self.project();
        let res = ready!(this.stream.as_mut().poll_next(cx));
        println!("Got WrapStream res.");
        Poll::Ready(res)
    }
}
