use crate::*;
use crate::client::*;
use crate::method::*;
use crate::response::*;
use url::Url;

/// Represent an unsent query.
///
/// Calling `send` actually makes the request.
pub struct Request<'body>
{
	pub(crate) method: Method,
	pub(crate) url: Option<Url>,
	pub(crate) headers: Option<CurlList>,
	pub(crate) redirect_limit: Option<usize>,
	pub(crate) request_body: Option<Box<dyn std::io::Read + 'body>>
}


pub(crate) struct CurlList
{
	pub(crate) headers: *mut sys::curl_slist,
}

impl Drop for CurlList
{
	fn drop(&mut self)
	{
		unsafe
		{
			sys::curl_slist_free_all(self.headers);
		}
	}
}

impl<'body> Request<'body>
{
	/// Create a request for a specific HTTP method and Url
	///
	///
	pub fn new(method: Method, url: Url) -> Request<'body>
	{
		Request
		{
			method,
			url: Some(url),
			//request_body: vec![].into(),
			headers: Some( CurlList{ headers: std::ptr::null_mut() } ),
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
	/// This is the number of bytes expected to be read by [`body()`](#method.body).
	///
	/// If specified, the value is sent as the `Content-Length`
	/// header.
	///
	/// It does not matter to curl if you specify the wrong value,
	/// but the server may object.
	pub fn set_content_length(&mut self, l: u64)
	{
		self.set_header(crate::header::CONTENT_LENGTH, format!("{}",l));
	}

	/// set the HTTP Content-Length header
	///
	/// This is the number of bytes expected to be read by [`body()`](#method.body).
	///
	/// If specified, the value is sent as the `Content-Length`
	/// header. Either way, curl will use
	/// `Transfer-Encoding: chunked`.
	///
	/// It does not matter to curl if you specify the wrong value,
	/// but the server may object.
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
			let l = sys::curl_slist_append(self.headers.as_ref().unwrap().headers, h.as_ptr() as *const i8);
			assert!(!l.is_null());
			self.headers.as_mut().unwrap().headers = l;
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
	///
	/// The specified body, if a reference, must outlive this `Request`.
	pub fn set_body<R: 'body>(&mut self, r: R)
		where R: std::io::Read + 'body
	{
		self.request_body = Some(Box::new(r));
	}

	/// Sets the reader which the payload to send is read from
	///
	/// The entire body is read before [`send()`](#method.send) completes.
	///
	/// The body is not read for the GET and DELETE methods.
	///
	/// The returned `Request` has the lifetime of your Read, because
	/// the body you intend to send needs to outlive the Request. You
	/// can either give a reference with a reader (example:
	/// `Cursor::new(&my_vector_object)`) or you can give ownership
	/// (`Cursor::new(owned_vector)`).
	pub fn body<'b, R: 'b>(self, r: R)
		-> Request<'b>
		where R: std::io::Read + 'b
	{
		let request_body = Some(Box::new(r) as Box<std::io::Read>);

		let Request
			{
				method,
				url,
				headers,
				redirect_limit,
				..
			} = self;

		Request
		{
			method: method,
			url: url,
			headers: headers,
			redirect_limit: redirect_limit,
			request_body,
		}
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
