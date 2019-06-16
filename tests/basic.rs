 
#[test]
fn example()
{
	let mut e = idcurl::get(url::Url::parse("http://example.com/").unwrap())
		.send().expect("request");
	assert!(e.data().expect("response").len() > 1000);
}
