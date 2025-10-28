AXI UARTLITE driver
========

This is a native Rust driver for the AMD AXI UART Lite v2.0 IP core.

# Core features

- Basic driver which can be created with a given IP core base address and supports as basic
  byte-level read and write API.
- Support for [`embedded-io`](https://docs.rs/embedded-io/latest/embedded_io/) and
  [`embedded-io-async`](https://docs.rs/embedded-io-async/latest/embedded_io_async/)

# Features

If the asynchronous support for the TX side is used, the number of statically provided wakers
can be configured using the following features:

- `1-waker` which is the default
- `2-wakers`
- `4-wakers`
- `8-wakers`
- `16-wakers`
- `32-wakers`
