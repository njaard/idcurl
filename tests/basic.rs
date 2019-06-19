 
#[test]
fn example()
{
	let mut e = idcurl::get("http://example.com/")
		.expect("request");
	assert!(e.data().expect("response").len() > 1000);
}

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
fn test_ownership()
{
	let _ = give_body();
	let v = vec!();
	let _ = take_body(&v);
}

fn give_body() -> idcurl::Request<'static>
{
	let v = vec!();
	idcurl::Request::post(url::Url::parse("http://example.com/").unwrap())
		.body(std::io::Cursor::new(v))
}

fn take_body<'a>(v: &'a Vec<u8>) -> idcurl::Request<'a>
{
	idcurl::Request::post(url::Url::parse("http://example.com/").unwrap())
		.body(std::io::Cursor::new(v))
}

