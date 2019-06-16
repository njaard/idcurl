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
	pub fn status(&self) -> StatusCode
	{
		self.rd.status_code
	}

	pub fn content_length(&self) -> Option<u64>
	{
		self.header(CONTENT_LENGTH)
			.map(|v| v.to_str().ok()?.parse().ok())?
	}

	pub fn text_as_utf8(&mut self) -> std::io::Result<String>
	{
		Ok(String::from_utf8(self.data()?).unwrap())
	}

	pub fn copy_to<W: std::io::Write+?Sized>(&mut self, w: &mut W)
		-> std::io::Result<u64>
	{
		std::io::copy(self, w)
	}

	pub fn header<K: AsHeaderName>(&self, k: K) -> Option<&HeaderValue>
	{
		self.headers().get(k)
	}

	pub fn data(&mut self)
		-> std::io::Result<Vec<u8>>
	{
		let mut d = vec!();
		self.read_to_end(&mut d)?;
		Ok(d)
	}

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
