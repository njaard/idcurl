
fn main()
{
	let url = std::env::args().nth(1)
		.expect("url expected as parameter");
	let url = url::Url::parse(&url).expect("parsing url");
	let mut e = idcurl::Request::post(
		url
	)
		.body(std::io::stdin())
		.content_length(100000)
		.send()
		.expect("error making request");
	assert!(e.status().is_success());
	e.copy_to(&mut std::io::stdout()).unwrap();
}
