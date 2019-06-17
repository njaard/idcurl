use crate::*;
use crate::client::*;
use crate::method::*;
use crate::response::*;
use url::Url;

/// Represent an unsent query.
///
/// Calling `send` actually makes the request.
pub struct Request
{
	pub(crate) method: Method,
	pub(crate) url: Url,
	pub(crate) headers: *mut sys::curl_slist,
	pub(crate) content_length: Option<u64>,
	pub(crate) redirect_limit: Option<usize>,
	pub(crate) request_body: Option<Box<dyn std::io::Read>>
}

impl Request
{
	/// Create a request for a specific HTTP method and Url
	///
	///
	pub fn new(method: Method, url: Url) -> Self
	{
		Self
		{
			method,
			url,
			//request_body: vec![].into(),
			headers: std::ptr::null_mut(),
			content_length: None,
			redirect_limit: Some(10),
			request_body: None,
		}
	}

	/// create a GET request
	///
	/// You may then use the builder-pattern to configure the request
	/// and then call [`send()`](#method.send) to begin it
	pub fn get(url: Url) -> Self
	{
		Self::new(Method::GET, url)
	}

	/// create a POST request
	///
	/// You should specify a payload to send with [`body()`](#method.body)
	/// or [`set_body()`](#method.set_body), which will be read
	/// before the request turns into a Response.
	pub fn post(url: Url) -> Self
	{
		Self::new(Method::POST, url)
	}

	/// create a PUT request
	///
	/// You should specify a payload to send with [`body()`](#method.body)
	/// or [`set_body()`](#method.set_body), which will be read
	/// before the request turns into a Response.
	pub fn put(url: Url) -> Self
	{
		Self::new(Method::PUT, url)
	}

	/// set the HTTP Content-Length header
	///
	/// If specified, the value is sent as the `Content-Length`
	/// header.
	///
	/// It's undefined what happens if you set this incorrectly.
	pub fn set_content_length(&mut self, l: u64)
	{
		self.content_length = Some(l);
	}

	/// set the HTTP Content-Length header
	///
	/// If specified, the value is sent as the `Content-Length`
	/// header.
	///
	/// It's undefined what happens if you set this incorrectly.
	pub fn content_length(mut self, l: u64) -> Self
	{
		self.set_content_length(l);
		self
	}

	/// Add one HTTP header to the request.
	///
	/// Any values may be sent, even those that may be invalid
	/// according to the HTTP specification. You should
	/// prefer to use the [`Header` constants](https://docs.rs/http/0.1.17/http/header/index.html)
	/// in the `idcurl::Header` module.
	pub fn set_header<K,V>(&mut self, k: K, v: V)
		where K: AsRef<[u8]>, V: AsRef<[u8]>
	{
		let k = k.as_ref();
		let v = v.as_ref();
		let mut h: Vec<u8> = Vec::with_capacity(k.len() + 2 + v.len() + 1);
		h.extend_from_slice(k);
		h.extend_from_slice(b": ");
		h.extend_from_slice(v);
		h.push(b'\0');

		unsafe
		{
			sys::curl_slist_append(self.headers, h.as_ptr() as *const i8);
		}
	}

	pub fn header<K,V>(mut self, k: K, v: V)
		-> Self
		where K: AsRef<[u8]>, V: AsRef<[u8]>
	{
		self.set_header(k,v);
		self
	}

	/// Sets the reader which the payload to send is read from
	///
	/// The entire body is read before [`send()`](#method.send) completes.
	///
	/// The body is not read for the GET and DELETE methods.
	pub fn set_body<R>(&mut self, r: R)
		where R: std::io::Read + 'static
	{
		let r = Box::new(r);
		self.request_body = Some(r);
	}

	pub fn body<R>(mut self, r: R)
		-> Self
		where R: std::io::Read + 'static
	{
		self.set_body(r);
		self
	}

	/// Make the HTTP request.
	///
	/// The configured request is sent along with its headers,
	/// then if specified, the [`body`](#method.body) is sent
	/// if the Method is appropriate.
	///
	/// The function succeeds and you get a Response if
	/// the server could be contacted and an HTTP session was
	/// initiated.
	///
	/// You should then call [`Response::status().is_success()`](status.html#method.is_success)
	/// to check for an HTTP status code in the 200 range.
	pub fn send(self) -> Result<Response>
	{
		Client::new()
			.execute(self)
	}

	/// sets the number of redirects that will be followed
	///
	/// The default is 10. An error will be returned if the
	/// number is exceeded.
	///
	/// Specify `None` to disable handling redirects. A redirect
	/// is not an error, instead you will get a `Response`
	/// for the redirect response.
	pub fn set_redirect_limit(&mut self, n: Option<usize>) -> &mut Self
	{
		self.redirect_limit = n;
		self
	}

	/// sets the number of redirects that will be followed
	///
	/// The default is 10. An error will be returned if the
	/// number is exceeded.
	///
	/// Specify `None` to disable handling redirects. A redirect
	/// is not an error, instead you will get a `Response`
	/// for the redirect response.
	pub fn redirect_limit(mut self, n: Option<usize>) -> Self
	{
		self.set_redirect_limit(n);
		self
	}
}

impl Drop for Request
{
	fn drop(&mut self)
	{
		unsafe
		{
			sys::curl_slist_free_all(self.headers);
		}
	}
}
