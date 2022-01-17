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
    let mut sleeper = Sleeper;
    let mut stream = StreamFut::new(&mut sleeper);
    let next = stream.next().await;
    dbg!(next.unwrap());
}

#[async_trait]
pub trait Async {
    async fn run(&mut self, s: u64) -> bool;
}
struct Sleeper;
#[async_trait]
impl Async for Sleeper {
    async fn run(&mut self, s: u64) -> bool {
        tokio::time::sleep(tokio::time::Duration::from_secs(s)).await;
        true
    }
}

#[pin_project]
pub struct StreamFut<'a, F, Fut> {
    f: &'a mut F,
    #[pin]
    fut: Option<Fut>,
}
impl<'a, F, Fut> StreamFut<'a, F, Fut> {
    pub fn new(f: &'a mut F) -> Self {
        Self { f, fut: None }
    }
}

// I think the problem is we can no longer Copy the &'a F reference from
// StreamFut, so we now need our reference to the StreamFut itself to live
// as long as the future, but we only get it for the anonymous lifetime
// in Pin<&mut Self>.
impl<'a, F> Stream for StreamFut<'_, F, BoxFuture<'a, bool>>
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
/// error[E0495]: cannot infer an appropriate lifetime for lifetime parameter `'a` due to conflicting requirements
///   --> src/mut_ref_trait.rs:57:29
///    |
/// 57 |         let mut this = self.project();
///    |                             ^^^^^^^
///    |
/// note: first, the lifetime cannot outlive the lifetime `'_` as defined here...
///   --> src/mut_ref_trait.rs:47:34
///    |
/// 47 | impl<'a, F> Stream for StreamFut<'_, F, BoxFuture<'a, bool>>
///    |                                  ^^
/// note: ...so that the types are compatible
///   --> src/mut_ref_trait.rs:57:29
///    |
/// 57 |         let mut this = self.project();
///    |                             ^^^^^^^
///    = note: expected `Pin<&mut mut_ref_trait::StreamFut<'_, F, Pin<Box<(dyn futures::Future<Output = bool> + std::marker::Send + 'a)>>>>`
///               found `Pin<&mut mut_ref_trait::StreamFut<'_, F, Pin<Box<(dyn futures::Future<Output = bool> + std::marker::Send + 'a)>>>>`
/// note: but, the lifetime must be valid for the lifetime `'a` as defined here...
///   --> src/mut_ref_trait.rs:47:6
///    |
/// 47 | impl<'a, F> Stream for StreamFut<'_, F, BoxFuture<'a, bool>>
///    |      ^^
/// note: ...so that the expression is assignable
///   --> src/mut_ref_trait.rs:65:30
///    |
/// 65 |                 this.fut.set(Some( this.f.run(3) ));
///    |                              ^^^^^^^^^^^^^^^^^^^^^
///    = note: expected `Option<Pin<Box<(dyn futures::Future<Output = bool> + std::marker::Send + 'a)>>>`
///               found `Option<Pin<Box<dyn futures::Future<Output = bool> + std::marker::Send>>>`
                this.fut.set(Some( this.f.run(3) ));
                todo!()
            }
        })
    }
}
