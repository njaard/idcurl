use libc::{c_ulong,c_char,size_t,c_void};

use crate::*;
use crate::header::*;

pub(crate) struct Client
{
	multi: *mut sys::CURLM,
	pub(crate) easy: *mut sys::CURL,
//	data: Rc<RefCell<TranceiverData>>,
}

/// eventually will let one reuse the same
/// connection
impl Client
{
	pub(crate) fn new() -> Client
	{
		crate::init();
		unsafe
		{
			let multi = sys::curl_multi_init();
			assert!(!multi.is_null());
			let easy = sys::curl_easy_init();
			assert!(!easy.is_null());
			crm(sys::curl_multi_add_handle(
				multi,
				easy,
			)).expect("adding handle");


			Self
			{
				multi,
				easy,
			}
		}
	}

	pub(crate) fn execute(self, request: Request)
		-> Result<Response>
	{
		unsafe
		{
			let e = self.execute2(request);
			e
		}
	}

	unsafe fn execute2(mut self, mut request: Request)
		-> Result<Response>
	{
		let mut rd = Box::new(ResponseData::new());
		let easy = self.easy;

		sys::curl_easy_reset(easy);

		let url = std::ffi::CString::new(request.url.as_ref().unwrap().as_str())
			.expect("making string");

		cr(sys::curl_easy_setopt(easy, curl_sys::CURLOPT_URL, url.as_ptr()))?;

		if let Some(n) = request.redirect_limit
		{
			cr(sys::curl_easy_setopt(easy, sys::CURLOPT_FOLLOWLOCATION, 1 as c_ulong))?;
			cr(sys::curl_easy_setopt(easy, sys::CURLOPT_MAXREDIRS, n as c_ulong))?;
		}

		let m: (&[u8], bool);
		match request.method
		{
			Method::GET => m=(b"GET\0", false),
			Method::POST => m=(b"POST\0", true),
			Method::PUT => m=(b"PUT\0", true),
			Method::DELETE => m=(b"DELETE\0", false),
			Method::HEAD => m=(b"HEAD\0", false),
			Method::OPTIONS => m=(b"OPTIONS\0", false),
			Method::TRACE => m=(b"TRACE\0", false),
		}
		if m.1
		{
			// we plan to send a body
			cr(sys::curl_easy_setopt(easy, sys::CURLOPT_UPLOAD, 1 as c_ulong))?;
		}
		cr(sys::curl_easy_setopt(easy, sys::CURLOPT_CUSTOMREQUEST, m.0.as_ptr()))?;


		cr(sys::curl_easy_setopt(easy, sys::CURLOPT_HTTPHEADER, request.headers.as_ref().unwrap().headers))?;

		{
			let rd = &mut request as &mut Request as *mut Request;
			cr(sys::curl_easy_setopt(
				easy,
				sys::CURLOPT_READDATA,
				rd,
			))?;
		}
		cr(sys::curl_easy_setopt(
			easy,
			sys::CURLOPT_READFUNCTION,
			read_callback as sys::curl_read_callback
				as *const sys::curl_read_callback
		))?;

		{
			let rd = &mut rd as &mut ResponseData as *mut ResponseData;
			cr(sys::curl_easy_setopt(
				easy,
				sys::CURLOPT_WRITEDATA,
				rd,
			))?;
		}
		cr(sys::curl_easy_setopt(
			easy,
			sys::CURLOPT_WRITEFUNCTION,
			write_callback as sys::curl_write_callback
				as *const sys::curl_write_callback
		))?;

		{
			let rd = &mut rd as &mut ResponseData as *mut ResponseData;
			cr(sys::curl_easy_setopt(
				easy,
				sys::CURLOPT_HEADERDATA,
				rd,
			))?;
		}
		cr(sys::curl_easy_setopt(
			easy,
			sys::CURLOPT_HEADERFUNCTION,
			header_callback as sys::curl_write_callback
				as *const sys::curl_write_callback
		))?;

		loop
		{
			let done = self.wait_and_process()?;
			if done || rd.headers_done || rd.completed
			{
				rd.completed = done;
				break;
			}
		}
		{
			let mut status: libc::c_long = 0;
			cr(sys::curl_easy_getinfo(
				self.easy,
				sys::CURLINFO_RESPONSE_CODE,
				&mut status as *mut _
			))?;
			rd.status_code = StatusCode::from_u16(status as u16)
				.map_err(
					|e|
						Error::new(
							Kind::Curl(format!("invalid status code: {}", e)),
							None
						)
				)?;
		}

		let response = Response
		{
			client: self,
			rd,
		};

		Ok(response)
	}

	pub(crate) fn wait_and_process(&mut self) -> Result<bool>
	{
		unsafe
		{
			sys::curl_multi_wait(
				self.multi,
				std::ptr::null_mut(),
				0,
				100000,
				std::ptr::null_mut(),
			);

			let mut n_handles = 0;
			crm(sys::curl_multi_perform(
				self.multi,
				&mut n_handles as *mut _,
			))?;

			if n_handles == 0
			{
				let mut msgs_left = 0;
				loop
				{
					let m = sys::curl_multi_info_read(self.multi, &mut msgs_left);
					if m.is_null() { break; }
					if (*m).msg == sys::CURLMSG_DONE
					{
						let c = (*m).data as sys::CURLcode;
						cr(c)?;
						return Ok(true);
					}
				}
			}

			Ok(false)
		}
	}
}

impl Drop for Client
{
	fn drop(&mut self)
	{
		unsafe
		{
			sys::curl_multi_remove_handle(self.multi, self.easy);
			sys::curl_easy_cleanup(self.easy);
			sys::curl_multi_cleanup(self.multi);
		}
	}
}

extern "C" fn write_callback(
	bytes: *mut c_char,
	size: size_t,
	nmemb: size_t,
	data: *mut c_void
) -> size_t
{
	unsafe
	{
		let buf = std::slice::from_raw_parts(bytes as *const u8, size*nmemb);
		let response = data as *mut ResponseData;
		let response = &mut *response;

		response.headers_done = true;

		response.read_queue.reserve(buf.len());
		for &c in buf
			{ response.read_queue.push_back(c); }
	}
	size*nmemb
}

extern "C" fn read_callback(
	bytes: *mut c_char,
	size: size_t,
	nmemb: size_t,
	data: *mut c_void
) -> size_t
{
	unsafe
	{
		let buf = std::slice::from_raw_parts_mut(bytes as *mut u8, size*nmemb);
		let request = data as *mut Request;
		let request = &mut *request;

		match request.request_body.as_mut()
		{
			Some(b) =>
			{
				let e = b.read(buf);
				if let Ok(e) = e
					{ e as size_t }
				else
					{ sys::CURL_READFUNC_ABORT }
			},
			None =>
				0,
		}
	}
}

extern "C" fn header_callback(
	bytes: *mut c_char,
	size: size_t,
	nmemb: size_t,
	data: *mut c_void
) -> size_t
{
	unsafe
	{
		let buf = std::slice::from_raw_parts(bytes as *const u8, size*nmemb);
		let response = data as *mut ResponseData;
		let response = &mut *response;

		let colon = buf.iter().enumerate()
			.find_map(|(idx,&b)| if b == b':' { Some(idx) } else { None });
		if colon.is_none() { return size*nmemb; }
		let colon = colon.unwrap();

		let mut name = &buf[0 .. colon];
		while !name.is_empty() && name.last().unwrap().is_ascii_whitespace()
			{ name = &name[ 0 .. name.len()-1]; }

		let mut value = &buf[colon+1 .. ];
		while value.starts_with(&b" "[..]) { value = &value[ 1 ..]; }
		if value.ends_with(&b"\n"[..]) { value = &value[ 0 .. value.len()-1]; }
		if value.ends_with(&b"\r"[..]) { value = &value[ 0 .. value.len()-1]; }

		let name = HeaderName::from_bytes(name);
		if name.is_err() { return 0; }

		let value = HeaderValue::from_bytes(value);
		if value.is_err() { return 0; }

		response.headers.append(name.unwrap(), value.unwrap());
	}
	size*nmemb
}

pub(crate) fn cr(rc: sys::CURLcode) -> Result<()>
{
	if rc == sys::CURLE_OK { return Ok(()); }

	let kind = kind_from_curl(rc);
	Err(Error::new(kind, None))
}

fn crm(rc: sys::CURLMcode) -> Result<()>
{
	if rc == sys::CURLM_OK { return Ok(()); }
	let kind = kind_from_curl(rc as sys::CURLcode);
	Err(Error::new(kind, None))
}

