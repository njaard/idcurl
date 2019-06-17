 
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
