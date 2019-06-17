 
#[test]
fn example()
{
	let mut e = idcurl::get("http://example.com/")
		.expect("request");
	assert!(e.data().expect("response").len() > 1000);
}
