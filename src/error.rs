use std::error::Error as StdError;

use crate::*;

/// Specifies the type of error
#[derive(Debug)]
pub enum Kind
{
	/// failure resolving proxy
	ResolveProxy,
	/// failure resolving host
	ResolveHost,
	/// failed connecting to host
	Connect,
	/// HTTP2 framing error
	Http2,
	/// failure reading from the specified Body
	BodyStreamFailure,
	/// outgoing interface could not be used
	InterfaceFailure,
	/// Redirect loop or too many redirects
	TooManyRedirects,
	/// The server sent nothing
	NothingFromServer,
	/// Failed to send on socket
	SendError,
	/// Failed to receive from socket
	RecvError,
	/// failure performing SSL handshake
	SslConnect,
	/// local client certificate problem
	SslLocalCertificate,
	/// The SSL cipher is invalid
	SslCipher,
	/// Remote server's SSL certificate is invalid
	SslCertificate,
	/// The remote server did not close via SSL
	SslShutdownFailed,
	/// unknown CURL error
	Curl(String),
	/// The expected was not the reported size
	PartialFile,
	/// The specified timeout was exceeded
	Timeout,
	/// The contents were not valid UTF-8
	NotUtf8(std::string::FromUtf8Error),
}

#[derive(Debug)]
pub struct Error
{
	kind: Kind,
	url: Option<String>,
}

impl Error
{
	pub fn new(kind: Kind, url: Option<String>)
		-> Error
	{
		Error
		{
			kind,
			url,
		}
	}

	pub fn kind(&self) -> &Kind
	{
		&self.kind
	}
}
impl std::fmt::Display for Error
{
	fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result
	{
		if let Some(ref url) = self.url
		{
			std::fmt::Display::fmt(url, f)?;
			f.write_str(": ")?;
		}
		std::fmt::Display::fmt(self.description(), f)
	}
}

impl StdError for Error
{
	fn description(&self) -> &str
	{
		match &self.kind
		{
			Kind::ResolveProxy => "failure resolving proxy",
			Kind::ResolveHost => "failure resolving host",
			Kind::Connect => "failed connecting to host",
			Kind::Http2 => "HTTP2 framing error",
			Kind::BodyStreamFailure => "failure reading from the specified Body",
			Kind::InterfaceFailure => "outgoing interface could not be used",
			Kind::TooManyRedirects => "Redirect loop or too many redirects",
			Kind::NothingFromServer => "The server sent nothing",
			Kind::SendError => "Failed to send on socket",
			Kind::RecvError => "Failed to receive from socket",
			Kind::SslConnect => "failure performing SSL handshake",
			Kind::SslLocalCertificate => "local client certificate problem",
			Kind::SslCipher => "The SSL cipher is invalid",
			Kind::SslCertificate => "Remote server's SSL certificate is invalid",
			Kind::SslShutdownFailed => "The remote server did not securely close its socket over SSL",
			Kind::Curl(a) => &a,
			Kind::PartialFile => "The expected was not the reported size",
			Kind::Timeout => "The specified timeout was exceeded",
			Kind::NotUtf8(_) => "The contents were not UTF-8",
		}
	}

	fn source(&self) -> Option<&(dyn StdError + 'static)>
	{
		None
	}
}

pub(crate) fn kind_from_curl(c: sys::CURLcode) -> Kind
{
	match c
	{
		sys::CURLE_COULDNT_RESOLVE_PROXY => Kind::ResolveProxy,
		sys::CURLE_COULDNT_RESOLVE_HOST => Kind::ResolveHost,
		sys::CURLE_COULDNT_CONNECT => Kind::Connect,
		sys::CURLE_HTTP2 | sys::CURLE_HTTP2_STREAM => Kind::Http2,
		sys::CURLE_ABORTED_BY_CALLBACK => Kind::BodyStreamFailure,
		sys::CURLE_PARTIAL_FILE => Kind::PartialFile,
		sys::CURLE_SSL_CONNECT_ERROR => Kind::SslConnect,
		sys::CURLE_TOO_MANY_REDIRECTS => Kind::TooManyRedirects,
		sys::CURLE_INTERFACE_FAILED => Kind::InterfaceFailure,
		sys::CURLE_GOT_NOTHING => Kind::NothingFromServer,
		sys::CURLE_SEND_ERROR => Kind::SendError,
		sys::CURLE_RECV_ERROR => Kind::RecvError,
		sys::CURLE_SSL_CERTPROBLEM => Kind::SslLocalCertificate,
		sys::CURLE_SSL_CIPHER => Kind::SslCipher,
		sys::CURLE_SSL_CACERT => Kind::SslCertificate,
		sys::CURLE_SSL_SHUTDOWN_FAILED => Kind::SslShutdownFailed,
		sys::CURLE_OPERATION_TIMEDOUT => Kind::Timeout,
		a => Kind::Curl(format!("curl error {}", a)),
	}
}
