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

### Woah, those function signatures are scary!

In the API, you will find function signatures like the following:
```rs
pub fn route<F>(self, f: F) -> RoutableHttpServer<'a, F, TX, RX, HTTP>
where
    F: for<'c, 'd, 'e> AsyncFn(&'c HttpConfig<'d>, RequestReader<'c, 'd, 'e>, ResponseWriter<'c, 'd>) -> Result<HttpResponse, Error>
```
These are necessary in order to provide the static guarantees that allow this crate to operate without a heap error-free.

I've done my best to abstract them away using convenient macros like `router!`.

Feel free to open issues if you have a better API in mind.