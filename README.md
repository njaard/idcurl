[![GitHub license](https://img.shields.io/badge/license-BSD-blue.svg)](https://raw.githubusercontent.com/njaard/idcurl/master/LICENSE)
[![Crates.io](https://img.shields.io/crates/v/idcurl.svg)](https://crates.io/crates/idcurl)

# Introduction

idcurl is a synchronous HTTP client using curl (and inheriting all of
its protocol support).

It's useful if you absolutely don't want to use futures.

# Examples


The most basic request:

	let mut output = vec!();
	idcurl::get("http://example.com")
		.expect("error making request")
		.copy_to(&mut output)
		.unwrap();

You can also configure your request:

	let body = r#"{ "hello": "world" }"#;

	let mut response = idcurl::Request::post(
		url::Url::parse("http://example.com").unwrap()
	)
		.header("Content-Type", "application/json")
		.body(std::io::Cursor::new(body))
		.send()
		.expect("http request");
	assert!(response.status().is_success());
	std::io::copy(&mut response, &mut std::io::stdout())
		.expect("reading response");
