
fn main()
{
	let url = std::env::args().nth(1)
		.expect("url expected as parameter");
	let url = url::Url::parse(&url).expect("parsing url");
	let mut e = idcurl::post(url, std::io::stdin())
		.expect("error making request");
	assert!(e.status().is_success());
	e.copy_to(&mut std::io::stdout()).unwrap();
}
