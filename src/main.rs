fn main() {
    let http_request_head: String = String::from("POST /api/auth HTTP/1.1"); // use regex or tokenizition to check
    let parts: Vec<&str> = http_request_head.split(' ').collect();
    println!("{}", parts[0]); // check length if i use tokenizition and handle each part to get valid method path and protocol/version
}

// parsing :
// 	method path http_version
// 	header(key): value
//
//	body.
