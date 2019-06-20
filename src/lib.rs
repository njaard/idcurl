//! An idiomatic synchronous Rust library for making HTTP requests.
//!
//! It's implemented in terms of curl.
//!
//! # Example
//!
//! ```rust
//! let mut output = vec!();
//! idcurl::get("http://example.com")
//!     .expect("error making request")
//!     .copy_to(&mut output)
//!     .unwrap();
//! ```
//!
//! ```rust
//! let body = r#"{ "hello": "world" }"#;
//!
//! let mut response = idcurl::Request::post(
//!     url::Url::parse("http://example.com").unwrap()
//! )
//!     .header("Content-Type", "application/json")
//!     .body(std::io::Cursor::new(body))
//!     .send()
//!     .expect("http request");
//! assert!(response.status().is_success());
//! std::io::copy(&mut response, &mut std::io::stdout())
//!     .expect("reading response");
//! ```

mod client;
mod request;
mod response;
mod method;
mod into_url;
mod error;

pub mod header
{
	pub use http::header::*;
}

pub use error::*;
pub use into_url::*;
pub use request::*;
pub use response::*;
pub use method::*;

pub type Result<T> = std::result::Result<T, Error>;
use std::sync::{Once};

/// Make a basic http GET request to the given URL
///
/// Returns an error if the url couldn't be parsed
/// or the request couldn't be made.
///
/// The response is ready for reading as an `std::io::Read`,
/// which you may want to convert to a `std::io::BufRead`.
///
/// ```
/// let mut response = idcurl::get("http://example.com")
///     .expect("failed to make HTTP request");
/// assert!(response.status().is_success());
/// response.copy_to(&mut std::io::stdout()).unwrap();
/// ```
pub fn get<U: TryIntoUrl>(url: U)
	-> Result<Response>
{
	let url = U::try_into_url(url)?;
	Request::get(url)
		.send()
}

/// Sends an http POST request to the given URL.
///
/// The payload to send is read from `r`, which can be easily made
/// with `std::io::Cursor` in case you're using a slice as a source.
///
/// ```
/// let data = b"something to send";
/// idcurl::post("http://example.com", std::io::Cursor::new(data))
///     .unwrap()
///     .copy_to(&mut std::io::stdout())
///     .unwrap();
/// ```
pub fn post<'b, U: TryIntoUrl, R: std::io::Read+'b>(url: U, r: R)
	-> Result<Response>
{
	let url = U::try_into_url(url)?;
	Request::post(url)
		.body(r)
		.send()
}

/// Sends an http PUT request to the given URL.
///
/// The payload to send is read from `r`, which can be easily made
/// with `std::io::Cursor` in case you're using a slice as a source.
///
/// ```
/// let data = b"something to send";
/// idcurl::put("http://example.com", std::io::Cursor::new(data))
///     .unwrap()
///     .copy_to(&mut std::io::stdout())
///     .unwrap();
/// ```
pub fn put<'b, U: TryIntoUrl, R: std::io::Read+'b>(url: U, r: R)
	-> Result<Response>
{
	let url = U::try_into_url(url)?;
	Request::put(url)
		.body(r)
		.send()
}

use curl_sys as sys;

pub use http::status::*;

/// Initializes the underlying libcurl library.
///
/// It's not required to call this before the library is used, but it's
/// recommended to do so as soon as the program starts.
pub fn init()
{
	static INIT: Once = Once::new();
	INIT.call_once(||
	{
		platform_init();
		unsafe
		{
			assert_eq!(curl_sys::curl_global_init(curl_sys::CURL_GLOBAL_ALL), 0);
		}

		// Note that we explicitly don't schedule a call to
		// `curl_global_cleanup`. The documentation for that function says
		//
		// > You must not call it when any other thread in the program (i.e. a
		// > thread sharing the same memory) is running. This doesn't just mean
		// > no other thread that is using libcurl.
		//
		// We can't ever be sure of that, so unfortunately we can't call the
		// function.
	});

	#[cfg(need_openssl_init)]
	fn platform_init()
	{
		openssl_sys::init();
	}

	#[cfg(not(need_openssl_init))]
	fn platform_init() {}
}
