# basic-cookies

Low-level [RFC 6265](https://tools.ietf.org/html/rfc6265.html) compatible cookie handling library for Rust.

[![Build Status](https://github.com/drjokepu/basic-cookies/actions/workflows/rust.yml/badge.svg)](https://github.com/drjokepu/basic-cookies/actions/workflows/rust.yml)
[![Docs](https://docs.rs/basic-cookies/badge.svg)](https://docs.rs/basic-cookies/)
[![crates.io](https://img.shields.io/crates/v/basic-cookies)](https://crates.io/crates/basic-cookies)
[![MIT](https://img.shields.io/crates/l/basic-cookies)](./LICENSE)

## Usage Example

```rust
use basic_cookies::Cookie;

let parsed_cookies = Cookie::parse("cookie1=value1; cookie2=value2").unwrap();

assert_eq!("cookie1", parsed_cookies[0].get_name());
assert_eq!("value1", parsed_cookies[0].get_value());

assert_eq!("cookie2", parsed_cookies[1].get_name());
assert_eq!("value2", parsed_cookies[1].get_value());
```