# `futures_become`

[![Build Status](https://github.com/Rufflewind/futures/actions/workflows/build.yml_become.svg?branch=master)](https://github.com/Rufflewind/futures/actions/workflows/build.yml_become)

[Documentation for the `master` branch](https://rufflewind.com/futures_become)

Tail-call elimination for [`futures`](https://crates.io/crates/futures).

Dispatching a long chain of futures built with `and_then` may result in a
linear cost.  One could avoid this using e.g. `loop_fn`, but this is hard to
compose due to shared state.

This package provides an alternative interface through a `Become` construct,
which acts as a kind of "tail-call elimination" at the futures level.  Turns
out it can be done without modifying the existing `Future` trait, but at the
cost of changing `Error` type of each future to a wrapped one using `Become`.
Then, returning `Err(Become::Become(next))` can be used to indicate that the
current future should be discarded and replaced with `next`.  This of course
means that we have to box each future, as the type of the current future could
be wildly different from that of the `next` future.
