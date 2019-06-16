
use crate::client::*;
use crate::method::*;
use crate::response::*;
use url::Url;

pub struct Request
{
	pub(crate) method: Method,
	pub(crate) url: Url,
	//request_body: Body,
	pub(crate) headers: curl::easy::List,
	pub(crate) content_length: Option<u64>,
	pub(crate) redirect_limit: Option<usize>,
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
			headers: curl::easy::List::new(),
			content_length: None,
			redirect_limit: Some(10),
		}
	}

	pub fn method(&mut self, m: Method) -> &mut Self
	{
		self.method = m;
		self
	}

	pub fn content_length(&mut self, l: u64) -> &mut Self
	{
		self.content_length = Some(l);
		self
	}

	/*fn body<T: IntoBody>(&mut self, body: T) -> &mut Self
	{
		self.body = body.into();
		self
	}*/

	pub fn header(&mut self, k: String, v: String) -> &mut Self
	{
		let _ = self.headers.append(&format!("{}: {}", k, v));
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

