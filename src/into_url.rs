use url::Url;
use crate::*;

/// Tries to convert Strings into Url types
pub trait TryIntoUrl
{
	fn try_into_url(self) -> Result<Url>;
}

impl<'a> TryIntoUrl for &'a str
{
	fn try_into_url(self) -> Result<Url>
	{
		Url::parse(&self)
			.map_err(|e| Error::new(Kind::UrlParse(e), None))
	}
}

impl TryIntoUrl for Url
{
	fn try_into_url(self) -> Result<Url>
	{
		Ok(self)
	}
}
impl<'a> TryIntoUrl for &'a Url
{
	fn try_into_url(self) -> Result<Url>
	{
		Ok(self.clone())
	}
}

impl<'a> TryIntoUrl for &'a String
{
	fn try_into_url(self) -> Result<Url>
	{
		Url::parse(&self)
			.map_err(|e| Error::new(Kind::UrlParse(e), None))
	}
}

