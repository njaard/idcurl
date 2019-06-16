use std::rc::Rc;
use std::cell::RefCell;
use std::io::Read;

use crate::client::*;
use crate::*;


pub struct Response
{
	pub(crate) h: RefCell<curl::multi::Easy2Handle<Tranceiver>>,
	pub(crate) tx: Rc<RefCell<TranceiverData>>,
	pub(crate) multi: curl::multi::Multi,
}

impl Response
{
	pub fn status(&self) -> StatusCode
	{
		StatusCode::from_u16(
			self.h.borrow_mut().get_req().response_code().expect("no response code")
				as u16
		)
	}

	pub fn content_length(&self) -> Option<u64>
	{
		None
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

	pub fn data(&mut self)
		-> std::io::Result<Vec<u8>>
	{
		let mut d = vec!();
		self.read_to_end(&mut d)?;
		Ok(d)
	}

	//pub fn headers(&self) -> &Headers
}


impl std::io::Read for Response
{
	fn read(&mut self, buf: &mut [u8])
		-> std::io::Result<usize>
	{
		let mut pos = 0;
		while pos != buf.len() && !self.tx.borrow().transfer_done
		{
			let num1;
			let num2;
			{
				let mut data = self.tx.borrow_mut();
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

			wait_and_do(&mut self.multi, &self.tx).unwrap();
		}

		Ok(pos)
	}
}
