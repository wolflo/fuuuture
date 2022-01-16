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
    let mut stream = StreamFut::new(Sleeper);
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
pub struct StreamFut<F, Fut> {
    f: F,
    #[pin]
    fut: Option<Fut>,
}
impl<F, Fut> StreamFut<F, Fut> {
    pub fn new(f: F) -> Self {
        Self { f, fut: None }
    }
}

impl<F> Stream for StreamFut<F, BoxFuture<'_, bool>>
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
                // Error: `cannot infer an appropriate lifetime for lifetime parameter 'pin.`
                // The future returned by f.run now captures the lifetime of
                // its reference to f, and that reference cannot outlive the
                // anonymous lifetime Pin<&mut Self> that we have access to
                // from poll_next(). We can't hold on to our future once we
                // return from this function.
                // this.fut.set(Some( this.f.run(3) ));
                todo!()
            }
        })
    }
}
