use futures::prelude::*;
use indexmap::IndexSet;
use linkerd2_error::Error;
use linkerd2_proxy_transport::listen::Addrs;
use std::{
    future::Future,
    pin::Pin,
    sync::Arc,
    task::{Context, Poll},
};
use tower::util::ServiceExt;

#[derive(Clone, Debug)]
pub struct Detect<D, F> {
    detect: D,
    forward: F,
    skip_ports: Arc<IndexSet<u16>>,
}

#[derive(Clone, Debug)]
pub enum Accept<D, F> {
    Detect(D),
    Forward(F),
}

impl<D, F> Detect<D, F> {}

impl<D, F> tower::Service<Addrs> for Detect<D, F>
where
    D: tower::Service<Addrs> + Clone + Send + 'static,
    D::Error: Into<Error>,
    D::Future: Send,
    F: tower::Service<Addrs> + Clone + Send + 'static,
    F::Error: Into<Error>,
    F::Future: Send,
{
    type Response = Accept<D::Response, F::Response>;
    type Error = Error;
    type Future = Pin<
        Box<dyn Future<Output = Result<Accept<D::Response, F::Response>, Error>> + Send + 'static>,
    >;

    fn poll_ready(&mut self, _: &mut Context<'_>) -> Poll<Result<(), Error>> {
        Poll::Ready(Ok(()))
    }

    fn call(&mut self, addrs: Addrs) -> Self::Future {
        if self.skip_ports.contains(&addrs.target_addr().port()) {
            let forward = self.forward.clone();
            Box::pin(async move {
                let f = forward.oneshot(addrs).err_into::<Error>().await?;
                Ok(Accept::Forward(f))
            })
        } else {
            let detect = self.detect.clone();
            Box::pin(async move {
                let d = detect.oneshot(addrs).err_into::<Error>().await?;
                Ok(Accept::Detect(d))
            })
        }
    }
}

impl<D, F, T> tower::Service<T> for Accept<D, F>
where
    D: tower::Service<T, Response = ()>,
    D::Error: Into<Error>,
    D::Future: Send + 'static,
    F: tower::Service<T, Response = ()>,
    F::Error: Into<Error>,
    F::Future: Send + 'static,
{
    type Response = ();
    type Error = Error;
    type Future = Pin<Box<dyn Future<Output = Result<(), Error>> + Send + 'static>>;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Error>> {
        let res = match self {
            Self::Detect(d) => futures::ready!(d.poll_ready(cx)).map_err(Into::into),
            Self::Forward(f) => futures::ready!(f.poll_ready(cx)).map_err(Into::into),
        };
        Poll::Ready(res)
    }

    fn call(&mut self, io: T) -> Self::Future {
        match self {
            Self::Detect(d) => {
                let fut = d.call(io).err_into::<Error>();
                Box::pin(async move { fut.await })
            }
            Self::Forward(f) => {
                let fut = f.call(io).err_into::<Error>();
                Box::pin(async move { fut.await })
            }
        }
    }
}
