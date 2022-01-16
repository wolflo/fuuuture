use futures::{
    ready,
    stream::{Stream, StreamExt},
    Future,
};
use pin_project::pin_project;
use std::task::Poll;

pub async fn main() {
    let mut stream = StreamFut::new(|x| Box::pin(sleep(x)));
    let next = stream.next().await;
    dbg!(next);
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

impl<F, Fut> Stream for StreamFut<F, Fut>
where
    F: FnMut(u64) -> Fut,
    Fut: Future,
{
    type Item = Fut::Output;

    fn poll_next(
        self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> Poll<Option<Self::Item>> {
        let mut this = self.project();
        Poll::Ready(loop {
            if let Some(fut) = this.fut.as_mut().as_pin_mut() {
                let item = ready!(fut.poll(cx));
                this.fut.set(None);
                break Some(item);
            } else {
                this.fut.set(Some((this.f)(3)));
            }
        })
    }
}

async fn sleep(s: u64) -> bool {
    tokio::time::sleep(tokio::time::Duration::from_secs(s)).await;
    true
}
