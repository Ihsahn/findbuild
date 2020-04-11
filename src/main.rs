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
use std::error::Error;

fn main() -> Result<(), Box<dyn Error>> {
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
                let part = get_page_and_find_matching(&user, &passwd, pattern, &path)?;
                if part.is_none() {
                    return Err("Pattern not found in specified url".into());
                }
                last_part = part.unwrap();
                path = format!("{}/{}", path, last_part.as_str());
            }
        }
    }
    let specific_build_url = path;
    let package_name = last_part;

    println!("Full url: {:?}", specific_build_url);
    let mut package_response = get(specific_build_url.as_str(), &user, &passwd)?;
    let mut out = File::create(package_name)?;
    let _ = io::copy(&mut package_response, &mut out)?;
    Ok(())
}

fn get_page_and_find_matching(user: &str, passwd: &str, pattern: &str, path: &str) -> Result<Option<String>, Box<dyn Error>> {
    let platform_page = get_page(path, user, passwd)?;
    let build_package_name = find_last_matching_url(&platform_page, pattern)?;
    let result = build_package_name.map(|v| v.to_string());
    Ok(result)
}

fn get(url: &str, user: &str, passwd: &str) -> Result<Response,  reqwest::Error> {
    let client = Client::new();
    let rsp = client
        .get(url)
        .basic_auth(user, Some(passwd))
        .send()?;
    Ok(rsp)
}

fn get_page(url: &str, user: &str, passwd: &str) -> Result<String, reqwest::Error> {
    let mut rsp = get(url, user, passwd)?;
    let body = rsp.text()?;
    Ok(body)
}

fn find_last_matching_url<'a>(page: &'a String, version_pattern: &str) -> Result<Option<&'a str>, regex::Error> {
    let regexp = Regex::new(version_pattern)?;
    let value = regexp.find_iter(page.as_str())
        .last();
     match value {
         None => Ok(None),
         Some(s) => Ok(Some(s.as_str()))
     }
}

#[cfg(test)]
mod tests {

    #[test]
    fn find_last_matching_url_invalid_regexp() {
        let page = r#"empty page"#;
        let page = String::from(page);
        let pattern ="bd+)";
        let result = super::find_last_matching_url(&page, pattern);

        assert!(result.is_err());
    }

    #[test]
    fn find_last_matching_url_empty_page() {
        let page = r#"empty page"#;
        let page = String::from(page);
        let pattern ="(b\\d+)";
        let result = super::find_last_matching_url(&page, pattern);

        assert_eq!(None, result.unwrap());
    }

    #[test]
    fn find_last_matching_url_match_found() {
        let page = r#"<!DOCTYPE html PUBLIC \"-//W3C//DTD HTML 3.2 Final//EN\"><html>
        \n<title>Directory listing for /builds/develop//b3/linux64/</title>\n<body>\n<h2>Directory listing for
        /builds/develop//b3/linux64/</h2>\n<hr>\n<ul>\n<li><a href=\"findbuild.deb\">findbuild.deb</a>\n<li><a href=\"findbuild.rpm\">
        findbuild.rpm</a>\n<li><a href=\"findbuild_2019-09-28.snap\">findbuild_2019-09-28.snap</a>\n</ul>\n<hr>\n</body>\n</html>\n"#;
        let page = String::from(page);
        let pattern ="(findbuild_.*?snap)";
        let result = super::find_last_matching_url(&page, pattern);

        assert_eq!("findbuild_2019-09-28.snap", result.unwrap().unwrap());
    }

}