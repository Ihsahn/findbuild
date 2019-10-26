extern crate reqwest;
extern crate regex;

use reqwest::{Client, Response};
use regex::Regex;
use std::fs::File;
use std::io;

fn main() {
    let user = "username";
    let passwd = "password";

    let parent_path = "http://localhost:8000/builds/develop/";
    let version_pattern = "(b\\d+)";
    let platform = "linux64";
    let package_pattern = "(findbuild_.*?snap)";

    let build_version_name = get_page_and_find_matching(user, passwd, version_pattern, parent_path);

    let platform_path = parent_path.to_owned() + build_version_name.as_str() + "/" + platform;

    let package_name = get_page_and_find_matching(user, passwd, package_pattern, &platform_path);

    let specific_build_url = platform_path + "/" + &package_name;

    println!("Full url: {:?}", specific_build_url);
    let mut package_response = get(specific_build_url.as_str(), user, passwd);
    let mut out = File::create(package_name).unwrap();
    io::copy(&mut package_response, &mut out).unwrap();
}

fn get_page_and_find_matching(user: &str, passwd: &str, pattern: &str, path: &str) -> String {
    let platform_page = get_page(path, user, passwd);
    let build_package_name = find_last_matching_url(&platform_page, pattern);
    build_package_name.to_string()
}

fn get(url: &str, user: &str, passwd: &str) -> Response {
    let client = Client::new();
    let rsp = client
        .get(url)
        .basic_auth(user, Some(passwd))
        .send()
        .unwrap();
    rsp
}

fn get_page(url: &str, user: &str, passwd: &str) -> String {
    let mut rsp = get(url, user, passwd);
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
