use std::io::Read;
use std::collections::VecDeque;

use crate::client::*;
use crate::header::*;
use crate::*;

pub(crate) struct ResponseData
{
	pub(crate) read_queue: VecDeque<u8>,
	pub(crate) headers_done: bool,
	pub(crate) transfer_done: bool,
	pub(crate) headers: HeaderMap,
	pub(crate) status_code: StatusCode,
}

impl ResponseData
{
	pub(crate) fn new() -> Self
	{
		Self
		{
			read_queue: VecDeque::new(),
			headers_done: false,
			transfer_done: false,
			headers: HeaderMap::new(),
			status_code: StatusCode::NOT_IMPLEMENTED,
		}
	}
}

/// Represents the result of an HTTP request
///
/// This object implements `Read`, which means
/// you can read it in a streaming fashion or
/// use the accessors to read it into memory.
pub struct Response
{
	pub(crate) client: Client,
	pub(crate) rd: Box<ResponseData>,
//	pub(crate) h: RefCell<curl::multi::Easy2Handle<Tranceiver>>,
//	pub(crate) tx: Rc<RefCell<TranceiverData>>,
//	pub(crate) multi: curl::multi::Multi,
}

impl Response
{
	/// Returns the HTTP status code.
	///
	/// `is_success()` is the most convenient way to
	/// make sure the message was received.
	pub fn status(&self) -> StatusCode
	{
		self.rd.status_code
	}

	/// Gets the Content-Length of the returned body.
	///
	/// If the server reported the length of the returned body,
	/// then this returns it, and None if the server didn't
	/// specify. This value is available before the body
	/// is read with [`data()`](#method.data) or [`text_as_utf8()`](#method.text_as_utf8)
	///
	/// It may also be a lie.
	pub fn content_length(&self) -> Option<u64>
	{
		self.header(CONTENT_LENGTH)
			.map(|v| v.to_str().ok()?.parse().ok())?
	}

	/// Read the entire document and interpret it as UTF-8.
	///
	/// Read the entire message body and
	pub fn text_as_utf8(&mut self) -> std::io::Result<String>
	{
		Ok(String::from_utf8(self.data()?).unwrap())
	}

	/// Copies this Read object into another Write object
	///
	/// Returns the number of bytes read or an Error
	/// if the request failed at some point.
	pub fn copy_to<W: std::io::Write+?Sized>(&mut self, w: &mut W)
		-> std::io::Result<u64>
	{
		std::io::copy(self, w)
	}

	/// Gets a specific HTTP header by name
	pub fn header<K: AsHeaderName>(&self, k: K) -> Option<&HeaderValue>
	{
		self.headers().get(k)
	}

	/// Reads all data into a vector, emptying this Response
	pub fn data(&mut self)
		-> std::io::Result<Vec<u8>>
	{
		let mut d = vec!();
		self.read_to_end(&mut d)?;
		Ok(d)
	}

	/// Gets a multimap of all HTTP headers received
	pub fn headers(&self) -> &HeaderMap
	{
		&self.rd.headers
	}
}




impl std::io::Read for Response
{
	fn read(&mut self, buf: &mut [u8])
		-> std::io::Result<usize>
	{
		let mut pos = 0;
		while pos != buf.len()
		{
			if self.rd.read_queue.len() == 0 && !self.rd.transfer_done
			{
				self.rd.transfer_done = self.client.wait_and_process()?;
			}
			if self.rd.read_queue.len() == 0 && self.rd.transfer_done
			{
				break;
			}

			let num1;
			let num2;
			let data = &mut self.rd;
			{
				let (a,b) = data.read_queue.as_slices();
				num1 = std::cmp::min(buf.len()-pos, a.len());
				buf[pos .. pos+num1].copy_from_slice(&a[0 .. num1]);
				pos += num1;
				num2 = std::cmp::min(buf.len()-pos, b.len());
				buf[pos .. pos+num2].copy_from_slice(&b[0 .. num2]);
				pos += num2;
			}
			data.read_queue.drain( 0 .. num1+num2 );
		}

		Ok(pos)
	}
}
