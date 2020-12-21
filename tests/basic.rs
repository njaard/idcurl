use std::io::Write;
use std::io::Read;

#[test]
fn remote_address()
{
	let e = idcurl::get("http://example.com/")
		.expect("request");
	let v = e.remote_address().unwrap();
	let nd = v.chars().filter(|&c| c == '.').count();
	let nc = v.chars().filter(|&c| c == ':').count();
	assert!((nd == 3 && nc==0) || (nd == 0 && nc>=6));
}

#[test]
fn bad_address()
{
	let e = idcurl::get("http://___________/").unwrap_err();
	eprintln!("{}", e);
}

#[test]
fn bad_connection()
{
	let e = idcurl::get("http://localhost:1/").unwrap_err();
	match e.kind()
	{
		idcurl::Kind::Connect => { },
		a => panic!("{:?}", a),
	}
}

#[test]
fn remote_disconnects()
{
	let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
	let port = listener.local_addr().unwrap().port();

	let t = std::thread::spawn(
		move ||
		{
			let q = listener.accept().unwrap();
			let mut s = q.0;
			std::thread::sleep(std::time::Duration::from_secs(20));
			s.write_all(b"HTTP/1.1 200 OK\r\n").unwrap();
			s.write_all(b"\r\n").unwrap();
			std::thread::sleep(std::time::Duration::from_secs(20));
		}
	);
	let e = idcurl::get(&format!("http://localhost:{}/", port));
	assert!(e.is_err());
	t.join().unwrap();
}

#[test]
fn long_wait1()
{
	let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
	let port = listener.local_addr().unwrap().port();

	let t = std::thread::spawn(
		move ||
		{
			let q = listener.accept().unwrap();
			let mut s = q.0;
			s.write_all(b"HTTP/1.1 200 OK\r\n").unwrap();
			s.write_all(b"Content-Length: 5\r\n").unwrap();
			s.write_all(b"\r\n").unwrap();
			s.write_all(b"hello").unwrap();
			std::thread::sleep(std::time::Duration::from_secs(20));
		}
	);
	let e = idcurl::get(&format!("http://localhost:{}/", port));
	assert!(e.is_ok());
	t.join().unwrap();
}

#[test]
fn long_wait2()
{
	let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
	let port = listener.local_addr().unwrap().port();

	let t = std::thread::spawn(
		move ||
		{
			let q = listener.accept().unwrap();
			let mut s = q.0;
			s.write_all(b"HTTP/1.1 200 OK\r\n").unwrap();
			s.write_all(b"Content-Length: 5\r\n").unwrap();
			s.write_all(b"\r\n").unwrap();
			s.write_all(b"hello").unwrap();
			std::thread::sleep(std::time::Duration::from_secs(20));
		}
	);
	let e = idcurl::get(&format!("http://localhost:{}/", port)).unwrap();
	e.bytes().last().unwrap().unwrap();
	eprintln!("done");
	t.join().unwrap();
}
#[test]
fn long_wait3()
{
	let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
	let port = listener.local_addr().unwrap().port();

	let t = std::thread::spawn(
		move ||
		{
			let q = listener.accept().unwrap();
			let mut s = q.0;
			s.write_all(b"HTTP/1.1 200 OK\r\n").unwrap();
			s.write_all(b"Content-Length: 6\r\n").unwrap();
			s.write_all(b"\r\n").unwrap();
			std::thread::sleep(std::time::Duration::from_secs(10));
		}
	);
	assert!(idcurl::get(&format!("http://localhost:{}/", port)).is_err());
	t.join().unwrap();
}
#[test]
fn some_data1()
{
	let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
	let port = listener.local_addr().unwrap().port();

	let t = std::thread::spawn(
		move ||
		{
			let q = listener.accept().unwrap();
			let mut s = q.0;
			s.write_all(b"HTTP/1.1 200 OK\r\n").unwrap();
			s.write_all(b"Content-Length: 5\r\n").unwrap();
			s.write_all(b"\r\n").unwrap();
			s.write_all(b"hello").unwrap();
			s.shutdown(std::net::Shutdown::Write).unwrap();
		}
	);
	let e = idcurl::get(&format!("http://localhost:{}/", port)).unwrap();
	e.bytes().last().unwrap().unwrap();
	t.join().unwrap();
}
#[test]
fn some_data2()
{
	let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
	let port = listener.local_addr().unwrap().port();

	let t = std::thread::spawn(
		move ||
		{
			let q = listener.accept().unwrap();
			let mut s = q.0;
			s.write_all(b"HTTP/1.1 200 OK\r\n").unwrap();
			s.write_all(b"Content-Length: 5\r\n").unwrap();
			s.write_all(b"\r\n").unwrap();
			s.write_all(b"world").unwrap();
			s.shutdown(std::net::Shutdown::Write).unwrap();
		}
	);
	let e = idcurl::get(&format!("http://localhost:{}/", port)).unwrap();
	e.bytes().last().unwrap().unwrap();
	t.join().unwrap();
}
#[test]
fn some_data3()
{
	let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
	let port = listener.local_addr().unwrap().port();

	let t = std::thread::spawn(
		move ||
		{
			let q = listener.accept().unwrap();
			let mut s = std::io::BufWriter::new(q.0);
			s.write_all(b"HTTP/1.1 200 OK\r\n").unwrap();
			s.write_all(b"Content-Length: 10000\r\n").unwrap();
			s.write_all(b"\r\n").unwrap();
			std::thread::sleep(std::time::Duration::from_secs(2));
			for _ in 0..10000
				{ s.write_all(b"a").unwrap(); }
			s.flush().unwrap();
			std::thread::sleep(std::time::Duration::from_secs(2));
		}
	);
	let e = idcurl::get(&format!("http://localhost:{}/", port)).unwrap();
	e.bytes().last().unwrap().unwrap();
	t.join().unwrap();
}
#[test]
fn some_data4()
{
	let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
	let port = listener.local_addr().unwrap().port();

	let _t = std::thread::spawn(
		move ||
		{
			let q = listener.accept().unwrap();
			let mut s = q.0;
			s.write_all(
				b"HTTP/1.1 200 OK\r\n\
				Content-Length: 1\r\n\
				\r\n\
				a\
			").unwrap();
		}
	);
	let e = idcurl::get(&format!("http://localhost:{}/", port)).unwrap();
	e.bytes().last().unwrap().unwrap();
}

#[test]
fn test_ownership()
{
	let _ = give_body();
	let v = vec!();
	let _ = take_body(&v);
}

fn give_body() -> idcurl::Request<'static>
{
	let v = vec!();
	idcurl::Request::post("http://example.com/".to_string())
		.body(std::io::Cursor::new(v))
}

fn take_body<'a>(v: &'a Vec<u8>) -> idcurl::Request<'a>
{
	idcurl::Request::post("http://example.com/".to_string())
		.body(std::io::Cursor::new(v))
}

