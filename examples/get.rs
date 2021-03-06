
fn main()
{
	let url = std::env::args().nth(1)
		.expect("url expected as parameter");
	let mut e = idcurl::get(&url)
		.expect("error making request");
	assert!(e.status().is_success());
	e.copy_to(&mut std::io::stdout()).unwrap();
}
