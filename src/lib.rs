//use std::{borrow::Cow};
//use url::*;
use worker::*;
use serde::{Serialize, Deserialize};
use serde_json::{Value};
use futures::future::join_all;
use tokio::io::BufReader;
use tokio::io::AsyncReadExt;
use regex::Regex;
use std::str;
use anyhow::Result;
use anyhow::anyhow;
use wasm_bindgen::JsValue;
use std::fmt::Write as _;


#[derive(Serialize, Deserialize, Debug, Clone)]
struct Server {
	address: String,
	port: u16,
	ssl: bool  // TODO: This works but might need changing for TLS
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct TestPath {
	resource_path: Option<String>
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct ReturnDataTest {
	good_return: Option<String>
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct ReturnCodeTest {
	good_code: Option<u32>
}

#[derive(Serialize, Deserialize, Debug, Clone)]
enum TestType {    
    TCPBanner {server: Server, string_test: ReturnDataTest},
	HTTPSBanner {server: Server, string_test: ReturnDataTest},
    HTTPGet {server: Server, test_path: TestPath, string_test: ReturnDataTest, code_test: ReturnCodeTest},
    HTTPPost {server: Server, test_path: TestPath, string_test: ReturnDataTest, code_test: ReturnCodeTest},
    HTTPSGet {server: Server, test_path: TestPath, string_test: ReturnDataTest, code_test: ReturnCodeTest},
    HTTPSPost {server: Server, test_path: TestPath, string_test: ReturnDataTest, code_test: ReturnCodeTest}  
}

#[derive(Serialize, Deserialize, Debug)]
struct Config {
	to_email: String,
	to_name: String,
	from_email: String,
	from_name: String,
	subject: String,
	mailchannel_url: String
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct Resource {
    name: String,
    enabled: bool,	
	test: TestType,    
}

#[derive(Debug)]
struct TestResult<'a> {
	resource: &'a Resource,
	test_result: Result<String>
}

struct TestError {
	message: String
}


// Connects to a TCP resource or HTTPS resource and checks if the good_return regex is found in the output
async fn banner(svr: &Server, str_test: &ReturnDataTest) -> Result<String> {	
	let retval;
	let good_return = str_test.good_return.clone().unwrap();
	console_log!("Banner {0}:{1}, search {2}", svr.address, svr.port, good_return);

    // Connect using Worker Socket
	let mut bannerdata = [0; 1200];
	console_debug!("Create socket");
    let socket = match svr.ssl {
		false => Socket::builder().connect(svr.address.clone(), svr.port)?,
		true => {
			Socket::builder()
			.secure_transport(SecureTransport::On) 
			.connect(svr.address.clone(), svr.port)?			
		},
	};
	// Read banner from socket
	let mut buf_reader = BufReader::new(socket);	
    buf_reader.read(&mut bannerdata).await?;  
	console_debug!("After Read socket");

	// Check return value against good_return
	let s = str::from_utf8(&bannerdata)?;
	console_debug!("banner = {s}");
	let check = search_for_return(str_test, &s.to_string()).await?;
	retval = if check {Ok(String::from("Success"))} 
		else {Err(anyhow!("Expected text PLACEHOLDER not found in banner PLACEHOLDER for PLACEHOLDER resource"))};	
    return retval;
}

// For now, going to pull the whole page and search the text
// in the future, we might 
async fn search_for_return(res: &ReturnDataTest, resp: &String) -> Result<bool> {
	let retval;

	// Only search if we have a good_return in our config
	if let Some(search) = &res.good_return {
		// Only search if we get back text in the response
		
		let re = Regex::new(&search)?;		
		if let Some(_x) = re.find(resp) {
			retval = Ok(true);
		} else {
			retval = Ok(false);
			console_debug!("Search text {search} not found");
		}
		
	} else {
		// No config means automatic success, because there is nothing to check
		retval = Ok(true);
	}
	return retval;
}

async fn https_get(svr: &Server, test_path: &TestPath, string_test: &ReturnDataTest, code_test: &ReturnCodeTest) -> Result<String> {
	let retval;
	// grabbing these required values will terminate the entire program if they aren't present
	let search_code = code_test.good_code.unwrap();
	let resource_path = test_path.resource_path.as_ref().unwrap();
	let url = format!("https://{0}:{1}{2}", svr.address, svr.port.to_string(), resource_path);
	console_log!("HTTPSGet {url}");
    
	// Create request and fetch the URL resource we are checking
    let request = Request::new(&url, Method::Get)?;       
    let mut response = Fetch::Request(request).send().await?.cloned()?;

	// Check for expected status code
	if response.status_code() == search_code as u16 {
		// check for search string if specifed
		let check = search_for_return(&string_test, &response.text().await?).await?;
		retval = if check {Ok(String::from("Success"))} 
					else {Err(anyhow!("Search string PLACEHOLDER not found in output"))};
	} else {
		retval = Err(anyhow!("Bad Status code PLACEHOLDER"));
	}	

    return retval;
}

async fn test_resource(res: &Resource) -> Result<TestResult> {
	console_log!("Test {0}", res.name);
    let test_result = match &res.test {
        TestType::TCPBanner {server, string_test} => banner(&server, &string_test).await,
        TestType::HTTPSBanner {server, string_test} => banner(&server, &string_test).await,
        TestType::HTTPSGet {server, test_path, string_test, code_test} => https_get(server, test_path, string_test, code_test).await,        
       _ => panic!("BOOPS!")
    };

	let retval = TestResult {
		resource: res,
		test_result: test_result
	};
    
    // This will never fail because we panic for an unhandled option
    return Ok(retval);
}

async fn load_configs(_env: Env) -> Result<(Config, Vec<Resource>)> {
	// Load base config
	let config = _env.var("CONFIG")?.to_string();
	console_debug!("Config raw {:?}", config);
	let config: Config = serde_json::from_str(&config).unwrap();
    console_log!("deserialized config = {:?}", config);

	// Load resource list
    let resources = _env.var("RESOURCES")?.to_string();
	console_debug!("Org {:?}", resources);
	let resources: Vec<Resource> = serde_json::from_str(&resources).unwrap();
    console_log!("deserialized = {:?}", resources);

	// eliminate disabled tests
	let resources: Vec<_> = resources.iter().filter(|x| x.enabled).cloned().collect();
	console_debug!("run_list = {:?}", resources);

	return Ok((config, resources));
}

fn some_failure_present(wrappedtr: &Result<TestResult>) -> bool {
	// if the outer result shows error, we are done
	let mut retval = wrappedtr.is_err();

	// No error at the top level of test result, look inside
	if let Ok(inner) = wrappedtr {
		retval = inner.test_result.is_err();
	}	

	return retval;
	
}

async fn get_email_post<'a>(config: &Config, failures: &Vec<&Result<TestResult<'a>>>) -> Result<String> {
	console_log!("start get_email_post");

	// Create template
	let post_template = r#"
			{
				"personalizations": [
					{
						"to": [
							{
								"email": "test@example.com",
								"name": "Test Recipient"
							}],
					},
				],
				"from": {
					"email": "sender@example.com",
					"name": "Workers - MailChannels integration",
				},
				"subject": "Look! No servers",
				"content": [
					{
						"type": "text/plain",
						"value": "And no email service accounts and all for free too!",
					},
				],
			}"#;
	console_log!("get_email_post before from_str {post_template}");

	let mut v: Value = serde_json::from_str(post_template)?;
	console_log!("get_email_post after from_str");

	// Fill template
	console_log!("fill template");

	v["personalizations"][0]["email"] = serde_json::to_value(&config.to_email)?;
	v["personalizations"][0]["name"] = serde_json::to_value(&config.to_name)?;
	v["from"]["email"] = serde_json::to_value(&config.from_email)?;
	v["from"]["name"] = serde_json::to_value(&config.from_name)?;
	v["subject"] = serde_json::to_value(&config.subject)?;

	// Build notification from failures, unwrapping and checking is a little complicated using a for loop instead
	let mut notification_text = String::new();
	for _element in failures {
		//write!(&mut notification_text, "FAIL\n");
		console_log!("fail item");

	}	
	console_log!("fill template complete");

	v["content"]["value"] = serde_json::to_value(notification_text)?;
	return Ok(v.to_string());
}

async fn dispatch_notification<'a>(config: &Config, failures: &Vec<&Result<TestResult<'a>>>) -> Result<bool> {
	console_log!("start dispatch_notification");
	
	let post_body = get_email_post(config, failures).await?;
	console_debug!("Email POST Body {post_body}");
	let mut hdr = Headers::new();
	hdr.append("content-type", "application/json")?;

	let ri = RequestInit {
		body: Some(JsValue::from_str(&post_body)),
		headers: hdr,
		cf: CfProperties::new(),
		method: Method::Post,
		redirect: RequestRedirect::default()
	};
    let request = Request::new_with_init(&config.mailchannel_url, &ri)?;										 
    let mut _response = Fetch::Request(request).send().await?.cloned()?;
	return Ok(true);
}

async fn monitor_process(config: &Config, resources: Vec<Resource>) -> Result<bool> {
	// Run the tests
	let tests: Vec<Result<TestResult>> = join_all(resources.iter().map(|res| test_resource(res))).await;
	console_log!("Results = {:?}", tests);

	// Gather failures
	let failures: Vec<_> = tests
		.iter()
		.filter(|res| some_failure_present(res))
		.collect();
	console_log!("Fails = {:?}", failures);

	// Send notifications for failures
	if failures.len() > 0 {
		console_log!("running email send ");
		dispatch_notification(config, &failures).await?;
	}
	

	return Ok(true);
}

#[event(fetch)]
async fn fetch(
    _req: Request,
    _env: Env,
    _ctx: Context,
) -> Result<Response, worker::Error> {
    console_error_panic_hook::set_once();

	// Load configuration
	let (config, resources) = load_configs(_env).await.unwrap();	
	let _ = monitor_process(&config, resources).await;        
    Response::ok("Completed")
}