# tinyhttp

A `no_std` (and no `alloc`) async HTTP/1.1 ergonomic server implementation, based on the amazing [`embassy-net`](https://github.com/embassy-rs/embassy)/[`smoltcp`](https://github.com/smoltcp-rs/smoltcp) network stack.

This crate intentionally does not implement HTTP/2 (or HTTP/3), or any TLS integration

> [!NOTE]  
> While this HTTP server implementation is under heavy development and working, it's NOT meant for production use yet, and its API surface is NOT stable.

## Design goals
- No standard library
- No dynamic allocations
- No panics
- Low memory/flash footprint
- RFC compliant
- Suitable for embedded web UIs
- Blazingly fast (🚀🚀🚀)

## Design non-goals
- Resiliency against state machine or Denial of Service attacks
- Many concurrent clients