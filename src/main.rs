extern crate reqwest;
extern crate regex;
extern crate clap;

use reqwest::{Client, Response};
use regex::Regex;
use std::fs::File;
use std::io;
use std::env;
use clap::{crate_authors, crate_description, crate_name, crate_version, Arg, App};
use std::process::exit;

fn main() {
    let arguments = App::new(crate_name!())
        .version(crate_version!())
        .author(crate_authors!())
        .about(crate_description!())
        .arg(Arg::with_name("user")
            .short("u")
            .long("user")
            .value_name("USER")
            .takes_value(true))
        .arg(Arg::with_name("password")
            .short("p")
            .long("password")
            .value_name("PASSWORD")
            .takes_value(true))
        .arg(Arg::with_name("path")
            .long("path")
            .value_name("PATH")
            .takes_value(true))
        .get_matches();

    let user = arguments.value_of("user")
        .map(|s| s.to_owned())
        .or(env::var("FINDBUILD_USERNAME").ok())
        .expect("No username specified");
    let passwd = arguments.value_of("password")
        .map(|s| s.to_owned())
        .or(env::var("FINDBUILD_PASSWORD").ok())
        .expect("No password specified");
    let path = arguments.value_of("path")
        .map(|s| s.to_owned())
        .or(env::var("FINDBUILD_PATH").ok())
        .expect("No path specified");

    let mut search_paths = path.split("|");

    println!("Username used: {:?}", user);
    println!("Path(s) used: ");
    search_paths.clone().for_each(|p| println!("  {:?}", p));

    if search_paths.clone().count() < 2 {
        println!("Path should contain at least two segments");
        exit(-1);
    }

    let mut path = search_paths.next().unwrap().to_string();
    let mut last_part = String::from("");
    loop {
        match search_paths.next() {
            None => { break; }
            Some(pattern) => {
                println!("looking for {} at {}", pattern, path);
                last_part = get_page_and_find_matching(&user, &passwd, pattern, &path);
                path = format!("{}/{}", path, last_part.as_str());
            }
        }
    }
    let specific_build_url = path;
    let package_name = last_part;

    println!("Full url: {:?}", specific_build_url);
    let mut package_response = get(specific_build_url.as_str(), &user, &passwd);
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
