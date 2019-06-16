use url::Url;

mod client;
mod request;
mod response;
mod method;

use request::*;
use method::*;

pub fn get(url: Url) -> Request
{
	Request::new(method::Method::GET, url)
}

pub struct StatusCode(u16);

impl StatusCode
{
	pub fn is_success(&self) -> bool
	{
		self.0 >= 200 && self.0 <= 299
	}

	pub fn from_u16(v: u16) -> StatusCode
	{
		StatusCode(v)
	}
}

