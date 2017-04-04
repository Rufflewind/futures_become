//! Tail-call elimination for futures.
//!
//! Dispatching a long chain of futures built with `and_then` may result in a
//! linear cost.  One could avoid this using e.g. `loop_fn`, but this is hard
//! to compose due to shared state.
//!
//! This package provides an alternative interface through a `Become`
//! construct, which acts as a kind of "tail-call elimination" at the futures
//! level.  Turns out it can be done without modifying the existing `Future`
//! trait, but at the cost of changing `Error` type of each future to a
//! wrapped one using `Become`.  Then, returning `Err(Become::Become(next))`
//! can be used to indicate that the current future should be discarded and
//! replaced with `next`.  This of course means that we have to box each
//! future, as the type of the current future could be wildly different from
//! that of the `next` future.
//!
//! ```
//! extern crate futures;
//! extern crate futures_become;
//!
//! use futures::{future, Async, Future};
//! use futures_become::{becomer, becomes, err};
//!
//! fn main() {
//!     let mut start = true;
//!     let mut root = becomer(future::poll_fn(|| {
//!         if start {
//!             start = false;
//!             Ok(Async::Ready(true))
//!         } else {
//!             unreachable!()
//!         }
//!     }).and_then(|b| {
//!         assert!(b);
//!         becomes(future::ok(":3").and_then(|s| {
//!             if s == ":3" {
//!                 future::ok(())
//!             } else {
//!                 err(())
//!             }
//!         }))
//!     }));
//!     match root.poll() {
//!         Ok(Async::Ready(())) => (),
//!         _ => unreachable!(),
//!     }
//! }
//! ```

extern crate futures;

use futures::{future, Poll, Future};

/// Controls the type of trait object used to dispatch tail calls.
pub trait Becoming {
    type Error;
    type Future: Future<Error=Result<Becomer<Self>, Self::Error>>;
}

impl<'b, 'a, T, E> Becoming for &'b mut (Future<Item=T, Error=E> + 'a) {
    type Error = E;
    type Future = &'b mut (Future<Item=T, Error=Result<Becomer<Self>, E>> + 'a);
}

impl<'b, 'a, T, E> Becoming for &'b mut (Future<Item=T, Error=E> + Send + 'a) {
    type Error = E;
    type Future = &'b mut (Future<Item=T, Error=Result<Becomer<Self>, E>> +
                           Send + 'a);
}

impl<'a, T, E> Becoming for BoxFuture<'a, T, E> {
    type Error = E;
    type Future = BoxFuture<'a, T, Result<Becomer<Self>, E>>;
}

impl<'a, T, E> Becoming for unsync::BoxFuture<'a, T, E> {
    type Error = E;
    type Future = unsync::BoxFuture<'a, T, Result<Becomer<Self>, E>>;
}

/// Dispatcher that can perform dynamic tail-call elimination.
///
/// This data type serves two purposes: one is to provide a `Future`
/// implementation that avoids breaking orphan rules, another is to tie the
/// knot in the recursive data structure.
///
/// This is a very generic type: you should probably use `becomer` instead to
/// avoid ambiguous type parameters.
pub struct Becomer<B: Becoming + ?Sized>(pub B::Future);

impl<B: Becoming + ?Sized> Future for Becomer<B> {
    type Item = <B::Future as Future>::Item;
    type Error = B::Error;
    fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
        loop {
            *self = match self.0.poll() {
                Err(Err(e)) => return Err(e),
                Err(Ok(next)) => next,
                Ok(async) => return Ok(async),
            };
        }
    }
}

/// An alias for a boxed `Future` that is `Send`.
pub type BoxFuture<'a, T, E> = Box<Future<Item=T, Error=E> + Send + 'a>;

/// An alias for a `Becomer` that is boxed and `Send`.
pub type BoxBecomer<'a, T, E> = Becomer<BoxFuture<'a, T, E>>;

/// Create a new future that has the ability to “run” a subfuture that
/// supports tail-call elimination.  This introduces a `Becomer` context.
pub fn becomer<'a, T, E, F>(f: F) -> BoxBecomer<'a, T, E>
    where F: Future<Item=T, Error=Result<BoxBecomer<'a, T, E>, E>> + Send + 'a
{
    Becomer(Box::new(f) as BoxFuture<_, _>)
}

/// A `Future` that simply becomes its contained value.  This only makes sense
/// when run under a `Becomer` context.
pub fn becomes<'a, F, T, E, U, D>(
    f: F) -> future::FutureResult<U, Result<BoxBecomer<'a, T, E>, D>>
    where F: Future<Item=T, Error=Result<BoxBecomer<'a, T, E>, E>> + Send + 'a
{
    future::err(Ok(becomer(f)))
}

/// A `Future` that simply raises an error.  Replaces `futures::future::err`
/// in `Becomer` contexts.
pub fn err<'a, T, E, U, D>(
    e: D) -> future::FutureResult<U, Result<BoxBecomer<'a, T, E>, D>>
{
    future::err(Err(e))
}

pub mod unsync {
    //! Analog of the outer module, but for non-`Send` futures.

    use futures::{future, Future};
    use super::Becomer;

    /// An alias for a boxed `Future` that is not `Send`.
    pub type BoxFuture<'a, T, E> = Box<Future<Item=T, Error=E> + 'a>;

    /// An alias for a `Becomer` that is boxed but not `Send`.
    pub type BoxBecomer<'a, T, E> = Becomer<BoxFuture<'a, T, E>>;

    /// Create a new future that has the ability to “run” a subfuture that
    /// supports tail-call elimination.  This introduces a `Becomer` context.
    pub fn becomer<'a, T, E, F>(f: F) -> BoxBecomer<'a, T, E>
        where F: Future<Item=T, Error=Result<BoxBecomer<'a, T, E>, E>> + 'a
    {
        Becomer(Box::new(f) as BoxFuture<_, _>)
    }

    /// A `Future` that simply becomes its contained value.  This only makes sense
    /// when run under a `Becomer` context.
    pub fn becomes<'a, F, T, E, U, D>(
        f: F) -> future::FutureResult<U, Result<BoxBecomer<'a, T, E>, D>>
        where F: Future<Item=T, Error=Result<BoxBecomer<'a, T, E>, E>> + 'a
    {
        future::err(Ok(becomer(f)))
    }

    /// A `Future` that simply raises an error.  Replaces `futures::future::err`
    /// in `Becomer` contexts.
    pub fn err<'a, T, E, U, D>(
        e: D) -> future::FutureResult<U, Result<BoxBecomer<'a, T, E>, D>>
    {
        future::err(Err(e))
    }
}

#[test]
fn main() {
    use futures::{future, Async};

    let mut start = true;
    let mut root = becomer(future::poll_fn(|| {
        if start {
            start = false;
            Ok(Async::Ready(true))
        } else {
            unreachable!()
        }
    }).and_then(|b| {
        assert!(b);
        becomes(future::ok(":3").and_then(|s| {
            if s == ":3" {
                future::ok(())
            } else {
                err(())
            }
        }))
    }));
    match root.poll() {
        Ok(Async::Ready(())) => (),
        _ => unreachable!(),
    }
}
