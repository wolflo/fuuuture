use async_trait::async_trait;
use futures::{
    future::BoxFuture,
    ready, stream,
    stream::{Stream, StreamExt},
    Future, FutureExt,
};
use std::task::Poll;

mod closure;
mod owned_trait;
mod ref_trait;
mod mut_ref_trait;
mod taker;

#[tokio::main]
async fn main() {
    closure::main().await;
    owned_trait::main().await;
    ref_trait::main().await;
    mut_ref_trait::main().await;
    taker::main().await;
}
