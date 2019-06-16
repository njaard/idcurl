
use std::collections::VecDeque;
use std::rc::Rc;
use std::cell::RefCell;

use crate::method::*;
use crate::request::*;
use crate::response::*;

pub(crate) struct Client
{
	multi: curl::multi::Multi,
//	data: Rc<RefCell<TranceiverData>>,
}

impl Client
{
	pub(crate) fn new() -> Client
	{
		Self
		{
			multi: curl::multi::Multi::new(),
		}
	}

	pub(crate) fn execute(mut self, request: Request)
		-> std::io::Result<Response>
	{
		let tx = TranceiverData::new();
		let tx = Rc::new(RefCell::new(tx));
		let tv = Tranceiver{ data: tx.clone() };

		let mut curl = curl::easy::Easy2::new(tv);
		curl.url(request.url.as_str())?;

		if let Some(n) = request.redirect_limit
		{
			curl.max_redirections(n as u32)?;
			curl.follow_location(true)?;
		}

		let m;
		match request.method
		{
			Method::GET => m=("GET", false),
			Method::POST => m=("POST", true),
			Method::PUT => m=("PUT", true),
			Method::DELETE => m=("DELETE", false),
			Method::HEAD => m=("HEAD", false),
		}
		// tells curl if we're going to send a body
		if m.1
			{ curl.post(true)?; }
		else
			{ curl.get(true)?; }

		curl.custom_request(m.0)?;

		if let Some(ref l) = request.content_length
			{ curl.post_field_size(*l)?; }

		let h = self.multi.add2(curl).unwrap();

		while !tx.borrow().headers_done
		{
			wait_and_do(&mut self.multi, &tx).unwrap();
		}

		// now we're ready to start reading payload,
		// so we can give the user the Response object

		let response = Response
		{
			tx,
			h: RefCell::new(h),
			multi: self.multi,
		};

		Ok(response)
	}
}

pub(crate) fn wait_and_do(
	curl: &mut curl::multi::Multi,
	tx: &RefCell<TranceiverData>,

) -> Result<(), curl::MultiError>
{
	curl.wait(&mut [], std::time::Duration::from_secs(100000))?;
	let count = curl.perform()?;
	if count == 0
	{
		let mut tx = tx.borrow_mut();
		tx.headers_done = true;
		tx.transfer_done = true;
	}

	Ok(())
}


pub(crate) struct TranceiverData
{
	pub(crate) read_queue: VecDeque<u8>,
	pub(crate) reader_callback: Option<Box<FnMut(&[u8])>>,
	pub(crate) headers_done: bool,
	pub(crate) transfer_done: bool,
	pub(crate) headers: Vec<Vec<u8>>,
}

impl TranceiverData
{
	fn new() -> Self
	{
		Self
		{
			read_queue: VecDeque::new(),
			reader_callback: None,
			headers_done: false,
			transfer_done: false,
			headers: vec!(),
		}
	}
}


pub(crate) struct Tranceiver
{
	data: Rc<RefCell<TranceiverData>>,
}

impl curl::easy::Handler for Tranceiver
{
	fn write(&mut self, buf: &[u8]) -> Result<usize, curl::easy::WriteError>
	{
		let mut data = self.data.borrow_mut();
		data.headers_done = true;
		data.read_queue.reserve(buf.len());
		for &c in buf
			{ data.read_queue.push_back(c); }
		if buf.len() == 0 { data.transfer_done = true; }
		Ok(buf.len())
	}

	fn read(&mut self, _data: &mut [u8]) -> Result<usize, curl::easy::ReadError>
	{
		Ok(0)
	}

	fn header(&mut self, buf: &[u8]) -> bool
	{
		eprintln!("header {:?}", String::from_utf8(buf.to_vec()));
		let mut data = self.data.borrow_mut();
		data.headers.push( buf.to_vec() );
		true
	}
}
