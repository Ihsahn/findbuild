extern crate reqwest;
extern crate regex;

use reqwest::Client;
use regex::Regex;

fn main() {
    let user = "username";
    let passwd = "password";

    let parent_path = "http://localhost:8000/builds/develop/";
    let version_pattern = "(b\\d+)";

    let page = get_page(parent_path, user, passwd);
    println!("Page: {:?}", page);

    let result = find_last_matching_url(&page, version_pattern);
    println!("Result: {:?}", result);
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

fn find_last_matching_url<'a>(page: &'a String, version_pattern: &str) -> &'a str {
    let regexp = Regex::new(version_pattern).unwrap();
    let value = regexp.find_iter(page.as_str())
        .last()
        .unwrap()
        .as_str();
    value
}
