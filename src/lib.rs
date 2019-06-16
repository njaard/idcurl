use url::Url;

mod client;
mod request;
mod response;
mod method;
pub mod header
{
	pub use http::header::*;
}

use request::*;
use response::*;
use method::*;

use std::sync::{Once, ONCE_INIT};

pub fn get(url: Url) -> Request
{
	Request::new(method::Method::GET, url)
}

pub fn post(url: Url) -> Request
{
	Request::new(method::Method::POST, url)
}

use curl_sys as sys;

pub use http::status::*;

/// Initializes the underlying libcurl library.
///
/// It's not required to call this before the library is used, but it's
/// recommended to do so as soon as the program starts.
pub fn init()
{
	static INIT: Once = ONCE_INIT;
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
