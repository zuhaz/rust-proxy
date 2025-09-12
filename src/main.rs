use actix_web::{
    get, http::header, middleware::Compress, web::Bytes, App, HttpRequest, HttpResponse,
    HttpServer, Responder, http::Method,
};
use futures_util::{stream::StreamExt};
use once_cell::sync::Lazy;
use reqwest::{
    header::{HeaderName, HeaderValue},
    Client,
};
use std::{collections::HashMap, str::FromStr, env};
use url::Url;
use tokio::task;

mod templates;

static ALLOWED_ORIGINS: Lazy<Vec<String>> = Lazy::new(|| {
    env::var("ALLOWED_ORIGINS")
        .unwrap_or_else(|_| "http://localhost:5173,http://localhost:3000".to_string())
        .split(',')
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect()
});

// Reqwest client pool
static CLIENT: Lazy<Client> = Lazy::new(|| {
    Client::builder()
        .pool_idle_timeout(std::time::Duration::from_secs(90))
        .http2_adaptive_window(true)
        .pool_max_idle_per_host(10)
        .danger_accept_invalid_certs(true)  // Accept invalid SSL certificates
        .build()
        .expect("Failed to build reqwest client")
});

static ENABLE_CORS: Lazy<bool> = Lazy::new(|| {
    std::env::var("ENABLE_CORS")
        .map(|v| v == "true" || v == "1")
        .unwrap_or(false)
});

fn validate_url(url: &str) -> Result<String, HttpResponse> {
    let url = url.trim();
    
    // Just validate it's a proper URL
    if Url::parse(url).is_ok() {
        Ok(url.to_string())
    } else {
        Err(HttpResponse::BadRequest().body(format!("Invalid URL: {}", url)))
    }
}

// Check if request has valid Origin or Referer - more permissive
fn get_valid_origin(req: &HttpRequest) -> Option<String> {
    if !*ENABLE_CORS {
        return None;
    }

    if let Some(origin) = req.headers().get(header::ORIGIN) {
        if let Ok(origin_str) = origin.to_str() {
            if ALLOWED_ORIGINS.contains(&origin_str.to_string()) {
                return Some(origin_str.to_string());
            }
        }
    }

    if let Some(referer) = req.headers().get(header::REFERER) {
        if let Ok(referer_str) = referer.to_str() {
            if let Some(allowed) = ALLOWED_ORIGINS
                .iter()
                .find(|origin| referer_str.starts_with(origin.as_str()))
            {
                return Some(allowed.clone());
            }
        }
    }

    None
}

fn get_url(line: &str, base: &Url) -> Url {
    if let Ok(absolute) = Url::parse(line) {
        return absolute;
    }
    base.join(line).unwrap_or_else(|_| base.clone())
}

#[inline]
fn process_m3u8_line(
    line: &str,
    scrape_url: &Url,
    headers_param: &Option<String>,
) -> String {
    if line.is_empty() {
        return String::new();
    }
    
    let first_char = unsafe { line.as_bytes().get_unchecked(0) };
    
    if (*first_char) == b'#' {
        // Comment line processing
        if line.len() > 11 && line.as_bytes()[10] == b'K' && line.starts_with("#EXT-X-KEY") {
            // #EXT-X-KEY processing
            if let Some(uri_start) = line.find("URI=\"") {
                let key_uri_start = uri_start + 5;
                if let Some(quote_pos) = line[key_uri_start..].find('"') {
                    let key_uri_end = key_uri_start + quote_pos;
                    let key_uri = &line[key_uri_start..key_uri_end];
                    let resolved = get_url(key_uri, scrape_url);
                    
                    let mut new_q = String::with_capacity(resolved.as_str().len() + 50);
                    new_q.push_str("url=");
                    new_q.push_str(&urlencoding::encode(resolved.as_str()));
                    if let Some(h) = headers_param {
                        new_q.push_str("&headers=");
                        new_q.push_str(h);
                    }
                    
                    let mut result = String::with_capacity(line.len() + new_q.len());
                    result.push_str(&line[..key_uri_start]);
                    result.push_str("/?");
                    result.push_str(&new_q);
                    result.push_str(&line[key_uri_end..]);
                    return result;
                }
            }
            return line.to_string();
        }
        
        if line.len() > 16 && line.starts_with("#EXT-X-MAP:URI=\"") {
            // #EXT-X-MAP processing
            let inner_url = &line[16..line.len()-1]; // Remove prefix and trailing quote
            let resolved = get_url(inner_url, scrape_url);
            
            let mut new_q = String::with_capacity(resolved.as_str().len() + 50);
            new_q.push_str("url=");
            new_q.push_str(&urlencoding::encode(resolved.as_str()));
            if let Some(h) = headers_param {
                new_q.push_str("&headers=");
                new_q.push_str(h);
            }
            
            let mut fixed = String::from("#EXT-X-MAP:URI=\"/?");
            fixed.push_str(&new_q);
            fixed.push('"');
            return fixed;
        }
        
        // Generic URI/URL processing for other tags
        if line.len() > 20 && (line.contains("URI=") || line.contains("URL=")) {
            if let Some(colon_pos) = line.find(':') {
                let prefix = &line[..colon_pos + 1];
                let attrs = &line[colon_pos + 1..];
                
                let mut result = String::with_capacity(line.len() + 100);
                result.push_str(prefix);
                
                let mut first_attr = true;
                for attr in attrs.split(',') {
                    if !first_attr {
                        result.push(',');
                    }
                    first_attr = false;
                    
                    if let Some(eq_pos) = attr.find('=') {
                        let key = attr[..eq_pos].trim();
                        let value = attr[eq_pos + 1..].trim().trim_matches('"');
                        
                        if key == "URI" || key == "URL" {
                            let resolved = get_url(value, scrape_url);
                            
                            let mut new_q = String::with_capacity(resolved.as_str().len() + 50);
                            new_q.push_str("url=");
                            new_q.push_str(&urlencoding::encode(resolved.as_str()));
                            if let Some(h) = headers_param {
                                new_q.push_str("&headers=");
                                new_q.push_str(h);
                            }
                            
                            result.push_str(key);
                            result.push_str("=\"/?");
                            result.push_str(&new_q);
                            result.push('"');
                        } else {
                            result.push_str(attr);
                        }
                    } else {
                        result.push_str(attr);
                    }
                }
                return result;
            }
        }
        
        return line.to_string();
    }
    
    // URL line processing
    let resolved = get_url(line, scrape_url);
    let mut new_q = String::with_capacity(resolved.as_str().len() + 50);
    new_q.push_str("url=");
    new_q.push_str(&urlencoding::encode(resolved.as_str()));
    if let Some(h) = headers_param {
        new_q.push_str("&headers=");
        new_q.push_str(h);
    }
    
    let mut result = String::with_capacity(new_q.len() + 10);
    result.push_str("/?");
    result.push_str(&new_q);
    result
}

// Handle CORS preflight requests - more permissive
async fn handle_options(req: HttpRequest) -> impl Responder {
    let origin = match get_valid_origin(&req) {
        Some(o) => o,
        None => {
            if *ENABLE_CORS { return HttpResponse::Forbidden().finish(); }
            "*".to_string()
        },
    };

    HttpResponse::Ok()
        .insert_header((header::ACCESS_CONTROL_ALLOW_ORIGIN, origin))
        .insert_header((header::ACCESS_CONTROL_ALLOW_METHODS, "GET, POST, OPTIONS, HEAD"))
        .insert_header((header::ACCESS_CONTROL_ALLOW_HEADERS, "Content-Type, Authorization, Range, X-Requested-With, Origin, Accept, Accept-Encoding, Accept-Language, Cache-Control, Pragma, Sec-Fetch-Dest, Sec-Fetch-Mode, Sec-Fetch-Site, Sec-Ch-Ua, Sec-Ch-Ua-Mobile, Sec-Ch-Ua-Platform, Connection"))
        .insert_header((header::ACCESS_CONTROL_EXPOSE_HEADERS, "Content-Length, Content-Range, Accept-Ranges, Content-Type, Cache-Control, Expires, Vary, ETag, Last-Modified"))
        .insert_header((header::ACCESS_CONTROL_MAX_AGE, "86400"))
        .insert_header((header::CROSS_ORIGIN_RESOURCE_POLICY, "cross-origin"))
        .insert_header(("Vary", "Origin"))
        .finish()
}

#[get("/")]
async fn m3u8_proxy(req: HttpRequest) -> impl Responder {
    // Parallel query parsing
    let query_future = task::spawn_blocking({
        let query_string = req.query_string().to_string();
        move || {
            query_string
                .split('&')
                .filter_map(|s| {
                    let mut split = s.splitn(2, '=');
                    let key = split.next()?;
                    let value = split.next().unwrap_or("");
                    Some((
                        key.to_string(),
                        urlencoding::decode(value).map(|v| v.into_owned()).ok()?,
                    ))
                })
                .collect::<HashMap<String, String>>()
        }
    });

    let query = match query_future.await {
        Ok(q) => q,
        Err(_) => return HttpResponse::InternalServerError().body("Query parsing failed"),
    };

    // Determine allowed CORS origin strictly from request headers
    let acao = get_valid_origin(&req);

    if *ENABLE_CORS && acao.is_none() {
        return HttpResponse::Forbidden().finish();
    }

    // Get and validate the URL
    let target_url = match query.get("url") {
        Some(u) => match validate_url(u) {
            Ok(validated) => validated,
            Err(resp) => return resp,
        },
        None => return HttpResponse::BadRequest().body("Missing URL"),
    };

    let target_url_parsed = match Url::parse(&target_url) {
        Ok(u) => u,
        Err(e) => return HttpResponse::BadRequest().body(format!("Invalid URL: {}", e)),
    };

    // Parallel header processing
    let headers_future = task::spawn_blocking({
        let target_url_parsed = target_url_parsed.clone();
        let query = query.clone();
        move || {
            // Use custom origin for upstream request if provided in query (for top-level fetch only)
            let origin_param = query.get("origin").map(|s| s.as_str());
            let mut headers = templates::generate_headers_for_url(&target_url_parsed, origin_param);

            // Custom headers support
            if let Some(header_json) = query.get("headers") {
                if let Ok(parsed) = serde_json::from_str::<HashMap<String, String>>(&header_json) {
                    for (k, v) in parsed {
                        if let (Ok(name), Ok(value)) = (
                            HeaderName::from_str(&k),
                            HeaderValue::from_str(&v),
                        ) {
                            headers.insert(name, value);
                        }
                    }
                }
            }

            // Debug: show chosen origin/referer for upstream
            let dbg_origin = headers.get("origin").and_then(|v| v.to_str().ok()).unwrap_or("-");
            let dbg_referer = headers.get("referer").and_then(|v| v.to_str().ok()).unwrap_or("-");
            eprintln!("Upstream headers for {} -> origin={}, referer={}", target_url_parsed.as_str(), dbg_origin, dbg_referer);

            headers
        }
    });

    let mut headers = match headers_future.await {
        Ok(h) => h,
        Err(_) => return HttpResponse::InternalServerError().body("Header processing failed"),
    };

    // Copy important headers from client request
    if let Some(range) = req.headers().get("Range") {
        headers.insert("Range", range.clone());
    }
    if let Some(if_range) = req.headers().get("If-Range") {
        headers.insert("If-Range", if_range.clone());
    }
    if let Some(if_none_match) = req.headers().get("If-None-Match") {
        headers.insert("If-None-Match", if_none_match.clone());
    }
    if let Some(if_modified_since) = req.headers().get("If-Modified-Since") {
        headers.insert("If-Modified-Since", if_modified_since.clone());
    }

    // Fetch target
    let resp = match CLIENT.get(&target_url).headers(headers).send().await {
        Ok(r) => r,
        Err(e) => {
            eprintln!("Failed to fetch target URL {}: {:?}", target_url, e);
            return HttpResponse::InternalServerError().body("Failed to fetch target URL");
        }
    };

    let status = resp.status();
    let headers_copy = resp.headers().clone();
    let content_type = headers_copy
        .get("Content-Type")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("")
        .to_ascii_lowercase();

    let ct_is_m3u8 = content_type.contains("mpegurl")
        || content_type.contains("application/vnd.apple.mpegurl")
        || content_type.contains("application/x-mpegurl");
    let url_looks_m3u8 = target_url.to_ascii_lowercase().ends_with(".m3u8");

    if ct_is_m3u8 || url_looks_m3u8 {
        let m3u8_text = match resp.text().await {
            Ok(t) => t,
            Err(e) => {
                eprintln!("Failed to read potential m3u8 content ({}): {:?}", target_url, e);
                return HttpResponse::InternalServerError().body("Failed to read m3u8");
            }
        };

        let looks_like_m3u8 = m3u8_text.trim_start().starts_with("#EXTM3U");
        if ct_is_m3u8 || looks_like_m3u8 {
            let scrape_url = Url::parse(&target_url).unwrap();
            let headers_param = query.get("headers").cloned();
            
            // Process m3u8 sequentially
            let lines = m3u8_text.lines();
            let mut processed_lines = Vec::with_capacity(lines.size_hint().0);
            
            for line in lines {
                processed_lines.push(process_m3u8_line(line, &scrape_url, &headers_param));
            }

            return HttpResponse::Ok()
                .insert_header((header::ACCESS_CONTROL_ALLOW_ORIGIN, acao.clone().unwrap_or("*".to_string())))
                .insert_header((header::ACCESS_CONTROL_ALLOW_METHODS, "GET, POST, OPTIONS, HEAD"))
                .insert_header((header::ACCESS_CONTROL_ALLOW_HEADERS, "Content-Type, Authorization, Range, Origin, Accept, Accept-Encoding, Accept-Language, Cache-Control, Pragma, Sec-Fetch-Dest, Sec-Fetch-Mode, Sec-Fetch-Site, Sec-Ch-Ua, Sec-Ch-Ua-Mobile, Sec-Ch-Ua-Platform, Connection"))
                .insert_header((header::ACCESS_CONTROL_EXPOSE_HEADERS, "Content-Length, Content-Range, Accept-Ranges, Content-Type, Cache-Control, Expires, Vary, ETag, Last-Modified"))
                .insert_header((header::CROSS_ORIGIN_RESOURCE_POLICY, "cross-origin"))
                .insert_header(("Vary", "Origin"))
                .content_type("application/vnd.apple.mpegurl")
                .insert_header((header::CACHE_CONTROL, "no-cache, no-store, must-revalidate"))
                .body(processed_lines.join("\n"));
        } else {
            let preview: String = m3u8_text.chars().take(200).collect();
            eprintln!(
                "Non-m3u8 body for URL ending with .m3u8 (status: {}, ct: {}): preview=\"{}\"",
                status.as_u16(),
                content_type,
                preview.replace('\n', "\\n")
            );

            let mut response_builder = HttpResponse::build(status);
            
            // Set CORS headers for all responses - more permissive
            response_builder.insert_header((header::ACCESS_CONTROL_ALLOW_ORIGIN, acao.clone().unwrap_or("*".to_string())));
            response_builder.insert_header((header::ACCESS_CONTROL_ALLOW_METHODS, "GET, POST, OPTIONS, HEAD"));
            response_builder.insert_header((header::ACCESS_CONTROL_ALLOW_HEADERS, "Content-Type, Authorization, Range, Origin, Accept, Accept-Encoding, Accept-Language, Cache-Control, Pragma, Sec-Fetch-Dest, Sec-Fetch-Mode, Sec-Fetch-Site, Sec-Ch-Ua, Sec-Ch-Ua-Mobile, Sec-Ch-Ua-Platform, Connection"));
            response_builder.insert_header((header::ACCESS_CONTROL_EXPOSE_HEADERS, "Content-Length, Content-Range, Accept-Ranges, Content-Type, Cache-Control, Expires, Vary, ETag, Last-Modified"));
            response_builder.insert_header((header::CROSS_ORIGIN_RESOURCE_POLICY, "cross-origin"));
            response_builder.insert_header(("Vary", "Origin"));
            
            // Copy important headers from the original response
            for (name, value) in headers_copy.iter() {
                let header_name = name.as_str().to_lowercase();
                if header_name == "content-type" 
                    || header_name == "content-length" 
                    || header_name == "content-range"
                    || header_name == "accept-ranges"
                    || header_name == "cache-control"
                    || header_name == "expires"
                    || header_name == "last-modified"
                    || header_name == "etag"
                    || header_name == "content-encoding"
                    || header_name == "vary" {
                    response_builder.insert_header((name.clone(), value.clone()));
                }
            }

            return response_builder.body(m3u8_text);
        }
    }

    let mut response_builder = HttpResponse::build(status);
    
    // Set CORS headers for all responses - more permissive
    response_builder.insert_header((header::ACCESS_CONTROL_ALLOW_ORIGIN, acao.clone().unwrap_or("*".to_string())));
    response_builder.insert_header((header::ACCESS_CONTROL_ALLOW_METHODS, "GET, POST, OPTIONS, HEAD"));
    response_builder.insert_header((header::ACCESS_CONTROL_ALLOW_HEADERS, "Content-Type, Authorization, Range, Origin, Accept, Accept-Encoding, Accept-Language, Cache-Control, Pragma, Sec-Fetch-Dest, Sec-Fetch-Mode, Sec-Fetch-Site, Sec-Ch-Ua, Sec-Ch-Ua-Mobile, Sec-Ch-Ua-Platform, Connection"));
    response_builder.insert_header((header::ACCESS_CONTROL_EXPOSE_HEADERS, "Content-Length, Content-Range, Accept-Ranges, Content-Type, Cache-Control, Expires, Vary, ETag, Last-Modified"));
    response_builder.insert_header((header::CROSS_ORIGIN_RESOURCE_POLICY, "cross-origin"));
    response_builder.insert_header(("Vary", "Origin"));
    
    // Copy important headers from the original response
    for (name, value) in headers_copy.iter() {
        let header_name = name.as_str().to_lowercase();
        if header_name == "content-type" 
            || header_name == "content-length" 
            || header_name == "content-range"
            || header_name == "accept-ranges"
            || header_name == "cache-control"
            || header_name == "expires"
            || header_name == "last-modified"
            || header_name == "etag"
            || header_name == "content-encoding"
            || header_name == "vary" {
            response_builder.insert_header((name.clone(), value.clone()));
        }
    }

    let stream = resp.bytes_stream().map(|chunk| {
        chunk
            .map(Bytes::from)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))
    });

    response_builder.body(actix_web::body::BodyStream::new(stream))
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenvy::dotenv().ok();

    println!("We alive bois: http://127.0.0.1:8080");
    if *ENABLE_CORS {
        println!("Allowed origins: {:?}", *ALLOWED_ORIGINS);
    }

    HttpServer::new(|| {
        App::new()
            .wrap(Compress::default())
            .wrap(actix_web::middleware::DefaultHeaders::new().add(("Vary", "Accept-Encoding")))
            .service(m3u8_proxy)
            .route("/", actix_web::web::method(Method::OPTIONS).to(handle_options))
    })
    .workers(num_cpus::get())
    .bind("0.0.0.0:8080")?
    .run()
    .await
}