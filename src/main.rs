use actix_files as fs;
use actix_web::{web, middleware, App, HttpResponse, HttpServer, Responder, get, http};
use serde::{Deserialize, Serialize};
use reqwest;
use regex::Regex;
use std::env::{args};

fn main() {
    std::env::set_var("RUST_LOG", "actix_server=info,actix_web=info");
    env_logger::init();

    let host_port = args().nth(1).unwrap_or("0.0.0.0:8080".to_string());
    
    HttpServer::new(|| {
        let static_path = args().nth(2).unwrap_or("static".to_string());
        App::new()
            .wrap(middleware::Compress::default())
            .wrap(middleware::Logger::default())
            .service(api_pic_hd)
            .service(fs::Files::new("/", static_path).index_file("index.html"))
    })
    .bind(host_port)
    .unwrap()
    .run()
    .unwrap();
}

#[derive(Serialize, Deserialize)]
struct HdPicFound {
    username: String,
    url: String,
}

#[derive(Serialize, Deserialize)]
struct HdPicNotFound {
    message: String,
}

#[get("/api/hd-pic/{username}")]
fn api_pic_hd(username: web::Path<String>) -> impl Responder {
    let result_html = grab_html(username.to_string());
    let not_found = HttpResponse::build(http::StatusCode::NOT_FOUND).json(HdPicNotFound {
        message: format!("No profile pic for user: {}", username)
    });
    
    if result_html.is_err() {
        return not_found;
    }
    
    let html = result_html.unwrap();
    if html == "" {
        return not_found;
    }

    let url = scrape_url(html);
    if url == "" {
        return not_found;
    }

    HttpResponse::Ok().json(HdPicFound {
        username: username.to_string(),
        url: url,
    })
}

fn grab_html(username: String) -> Result<String, reqwest::Error> {
    let url = format!("https://www.instadp.com/fullsize/{}", username);
    reqwest::get(&url)?.text()
}

fn scrape_url(html: String) -> String {
    let re = Regex::new(r#"<img class="picture" src="(?P<url>[^"]+)""#).unwrap();

    match re.captures(&html) {
        Some(matched) => matched["url"].to_string(),
        _ => "".to_string(),
    }
}