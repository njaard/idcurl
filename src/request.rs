
use crate::*;
use crate::client::*;
use crate::method::*;
use crate::response::*;
use url::Url;

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

	pub fn content_length(mut self, l: u64) -> Self
	{
		self.content_length = Some(l);
		self
	}

	/*fn body<T: IntoBody>(&mut self, body: T) -> &mut Self
	{
		self.body = body.into();
		self
	}*/

	pub fn header<K,V>(self, k: K, v: V) -> Self
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

		self
	}

	pub fn body<R>(mut self, r: R) -> Self
		where R: std::io::Read + 'static
	{
		let r = Box::new(r);
		self.request_body = Some(r);
		self
	}

	pub fn send(self) -> std::io::Result<Response>
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
