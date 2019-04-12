extern crate reqwest;

use reqwest::Client;

fn main() {
    let user = "username";
    let passwd = "password";

    let parent_path = "http://localhost:8000/builds/develop/";

    let page = get_page(parent_path, user, passwd);
    println!("Page: {:?}", page);
}

fn get_page(url: &str, user: &str, passwd: &str) -> String {
    let client = Client::new();
    let mut rsp = client
        .get(url)
        .basic_auth(user, Some(passwd))
        .send()
        .unwrap();
    let body = rsp.text().unwrap();
    body
}
