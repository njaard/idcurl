use url::Url;

use std::io::{Error,ErrorKind};

/// Tries to convert Strings into Url types
pub trait TryIntoUrl
{
	fn try_into_url(self) -> std::io::Result<Url>;
}

impl<'a> TryIntoUrl for &'a str
{
	fn try_into_url(self) -> std::io::Result<Url>
	{
		Url::parse(&self)
			.map_err(|e| Error::new(ErrorKind::InvalidInput, e))
	}
}

impl TryIntoUrl for Url
{
	fn try_into_url(self) -> std::io::Result<Url>
	{
		Ok(self)
	}
}
impl<'a> TryIntoUrl for &'a Url
{
	fn try_into_url(self) -> std::io::Result<Url>
	{
		Ok(self.clone())
	}
}

impl<'a> TryIntoUrl for &'a String
{
	fn try_into_url(self) -> std::io::Result<Url>
	{
		Url::parse(&self)
			.map_err(|e| Error::new(ErrorKind::InvalidInput, e))
	}
}

