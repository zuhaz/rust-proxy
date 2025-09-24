use regex::Regex;
use once_cell::sync::Lazy;
use url::Url;
use reqwest::header::{HeaderMap, HeaderName, HeaderValue};
use std::collections::HashMap;
use std::str::FromStr;

// Define default headers used by most templates
static DEFAULT_HEADERS: Lazy<HashMap<String, String>> = Lazy::new(|| {
    HashMap::from([
        ("user-agent".to_string(), "Mozilla/5.0 (Windows NT 10.0; Win64; x64; rv:137.0) Gecko/20100101 Firefox/137.0".to_string()),
        ("accept".to_string(), "*/*".to_string()),
        ("accept-language".to_string(), "en-US,en;q=0.5".to_string()),
        ("sec-fetch-dest".to_string(), "empty".to_string()),
        ("sec-fetch-mode".to_string(), "cors".to_string()),
        ("sec-fetch-site".to_string(), "cross-site".to_string()),
    ])
});

// Define domain group configuration - simplified and focused
struct DomainGroup {
    patterns: Vec<&'static str>,
    origin: &'static str,
    referer: &'static str,
    custom_headers: Option<HashMap<&'static str, &'static str>>, // New: custom headers per domain
}

static DOMAIN_GROUPS: Lazy<Vec<DomainGroup>> = Lazy::new(|| {
    vec![
        DomainGroup {
            patterns: vec![
                r"(?i)\.padorupado\.ru$",
                r"(?i)\.kwikie\.ru$",
            ],
            origin: "https://kwik.si",
            referer: "https://kwik.si/",
            custom_headers: Some(HashMap::from([
                ("cache-control", "no-cache"),
                ("pragma", "no-cache"),
            ])),
        },
        DomainGroup {
            patterns: vec![
                r"(?i)\.streamtape\.to$",
            ],
            origin: "https://streamtape.to",
            referer: "https://streamtape.to/",
            custom_headers: None,
        },
        DomainGroup {
            patterns: vec![
               r"(?i)vidcache\.net$",
            ],
            origin: "https://www.animegg.org",
            referer: "https://www.animegg.org/",
            custom_headers: None,
        },
        DomainGroup {
            patterns: vec![
                r"(?i)krussdomi\.com$",
                r"(?i)revolutionizingtheweb\.xyz$",
                r"(?i)nextgentechnologytrends\.xyz$",
                r"(?i)smartinvestmentstrategies\.xyz$",
                r"(?i)creativedesignstudioxyz\.xyz$",
                r"(?i)breakingdigitalboundaries\.xyz$",
                r"(?i)ultimatetechinnovation\.xyz$",
            ],
            origin: "https://krussdomi.com",
            referer: "https://krussdomi.com/",
            custom_headers: None,
        },
        DomainGroup {
            patterns: vec![r"(?i)\.akamaized\.net$"],
            origin: "https://players.akamai.com",
            referer: "https://players.akamai.com/",
            custom_headers: None,
        },
        DomainGroup {
            patterns: vec![r"(?i)(?:^|\.)shadowlandschronicles\.", r"(?i)digitalshinecollective\.xyz$", r"(?i)thrivequesthub\.xyz$", r"(?i)novaedgelabs\.xyz$"],
            origin: "https://cloudnestra.com",
            referer: "https://cloudnestra.com/",
            custom_headers: None,
        },        
        DomainGroup {
            patterns: vec![r"(?i)(?:^|\.)viddsn\.", r"(?i)\.anilike\.cyou$"],
            origin: "https://vidwish.live/",
            referer: "https://vidwish.live/",
            custom_headers: None,
        },
        DomainGroup {
            patterns: vec![r"(?i)(?:^|\.)dotstream\.", r"(?i)(?:^|\.)playcloud1\."],
            origin: "https://megaplay.buzz/",
            referer: "https://megaplay.buzz/",
            custom_headers: None,
        },        
        DomainGroup {
            patterns: vec![r"(?i)\.cloudfront\.net$"],
            origin: "https://d2zihajmogu5jn.cloudfront.net",
            referer: "https://d2zihajmogu5jn.cloudfront.net/",
            custom_headers: None,
        },
        DomainGroup {
            patterns: vec![r"(?i)\.ttvnw\.net$"],
            origin: "https://www.twitch.tv",
            referer: "https://www.twitch.tv/",
            custom_headers: None,
        },
        DomainGroup {
            patterns: vec![r"(?i)\.xx\.fbcdn\.net$"],
            origin: "https://www.facebook.com",
            referer: "https://www.facebook.com/",
            custom_headers: None,
        },
        DomainGroup {
            patterns: vec![r"(?i)\.anih1\.top$", r"(?i)\.xyk3\.top$"],
            origin: "https://ee.anih1.top",
            referer: "https://ee.anih1.top/",
            custom_headers: None,
        },
        DomainGroup {
            patterns: vec![r"(?i)\.premilkyway\.com$"],
            origin: "https://uqloads.xyz",
            referer: "https://uqloads.xyz/",
            custom_headers: None,
        },
        DomainGroup {
            patterns: vec![r"(?i)\.streamcdn\.com$"],
            origin: "https://anime.uniquestream.net",
            referer: "https://anime.uniquestream.net/",
            custom_headers: None,
        },
        DomainGroup {
            patterns: vec![r"(?i)\.raffaellocdn\.net$", r"(?i)\.feetcdn\.com$", r"(?i)clearskydrift45\.site$"],
            origin: "https://kerolaunochan.online",
            referer: "https://kerolaunochan.online/",
            custom_headers: None,
        },
        DomainGroup {
            patterns: vec![r"(?i)dewbreeze84\.online$", r"(?i)cloudydrift38\.site$", r"(?i)sunshinerays93\.live$", r"(?i)clearbluesky72\.wiki$", r"(?i)breezygale56\.online$", r"(?i)frostbite27\.pro$", r"(?i)frostywinds57\.live$", r"(?i)icyhailstorm64\.wiki$", r"(?i)icyhailstorm29\.online$", r"(?i)windflash93\.xyz$", r"(?i)stormdrift27\.site$", r"(?i)tempestcloud61\.wiki$", r"(?i)sunburst66\.pro$", r"(?i)douvid\.xyz$"],
            origin: "https://megacloud.blog",
            referer: "https://megacloud.blog/",
            custom_headers: Some(HashMap::from([
                ("cache-control", "no-cache"),
                ("pragma", "no-cache"),
            ])),
        },
        DomainGroup {
            patterns: vec![r"(?i)\.echovideo\.to$"],
            origin: "https://aniwave.se",
            referer: "https://aniwave.se/",
            custom_headers: None,
        },
        DomainGroup {
            patterns: vec![r"(?i)\.vid-cdn\.xyz$"],
            origin: "https://anizone.to/",
            referer: "https://anizone.to/",
            custom_headers: None,
        },
        DomainGroup {
            patterns: vec![r"(?i)\.1stkmgv1\.com$"],
            origin: "https://animeyy.com",
            referer: "https://animeyy.com/",
            custom_headers: None,
        },
        DomainGroup {
            patterns: vec![r"(?i)lightningspark77\.pro$", r"(?i)thunderwave48\.xyz$", r"(?i)stormwatch95\.site$", r"(?i)windyrays29\.online$", r"(?i)thunderstrike77\.online$", r"(?i)lightningflash39\.live$", r"(?i)cloudburst82\.xyz$", r"(?i)drizzleshower19\.site$", r"(?i)rainstorm92\.xyz$"],
            origin: "https://megacloud.club",
            referer: "https://megacloud.club/",
            custom_headers: None,
        },
        DomainGroup {
            patterns: vec![r"(?i)cloudburst99\.xyz$", r"(?i)frostywinds73\.pro$", r"(?i)stormwatch39\.live$", r"(?i)sunnybreeze16\.live$", r"(?i)mistydawn62\.pro$", r"(?i)lightningbolt21\.live$", r"(?i)gentlebreeze85\.xyz$"],
            origin: "https://videostr.net",
            referer: "https://videostr.net/",            
            custom_headers: None,
        },        
        DomainGroup {
            patterns: vec![r"(?i)vmeas\.cloud$"],
            origin: "https://vidmoly.to",
            referer: "https://vidmoly.to/",
            custom_headers: None,
        },
        DomainGroup {
            patterns: vec![r"(?i)nextwaveinitiative\.xyz$"],
            origin: "https://edgedeliverynetwork.org",
            referer: "https://edgedeliverynetwork.org/",
            custom_headers: None,
        },
        DomainGroup {
            patterns: vec![r"(?i)lightningbolts\.ru$", r"(?i)lightningbolt\.site$", r"(?i)vyebzzqlojvrl\.top$"],
            origin: "https://vidsrc.cc",
            referer: "https://vidsrc.cc/",
            custom_headers: None,
        },
        DomainGroup {
            patterns: vec![r"(?i)vidlvod\.store$"],
            origin: "https://vidlink.pro",
            referer: "https://vidlink.pro/",
            custom_headers: None,
        },
        DomainGroup {
            patterns: vec![r"(?i)sunnybreeze16\.live$"],
            origin: "https://megacloud.store",
            referer: "https://megacloud.store/",
            custom_headers: None,
        },
        DomainGroup {
            patterns: vec![r"(?i)heatwave90\.pro$", r"(?i)humidmist27\.wiki$", r"(?i)frozenbreeze65\.live$", r"(?i)drizzlerain73\.online$", r"(?i)sunrays81\.xyz$"],
            origin: "https://kerolaunochan.live",
            referer: "https://kerolaunochan.live/",
            custom_headers: None,
        },
        DomainGroup {
            patterns: vec![r"(?i)\.vkcdn5\.com$"],
            origin: "https://vkspeed.com",
            referer: "https://vkspeed.com/",
            custom_headers: None,
        },
        DomainGroup {
            patterns: vec![r"(?i)embed\.su$", r"(?i)usbigcdn\.cc$", r"(?i)\.congacdn\.cc$"],
            origin: "https://embed.su",
            referer: "https://embed.su/",
            custom_headers: None,
        },
    ]
});

// Generate headers for a URL with optional custom origin
pub fn generate_headers_for_url(url: &Url, custom_origin: Option<&str>) -> HeaderMap {
    let mut headers = HeaderMap::new();
    
    // Add base headers
    for (key, value) in DEFAULT_HEADERS.iter() {
        if let (Ok(name), Ok(val)) = (
            HeaderName::from_str(key),
            HeaderValue::from_str(value),
        ) {
            headers.insert(name, val);
        }
    }

    // If custom origin is provided, use it
    if let Some(origin) = custom_origin {
        if let (Ok(name), Ok(val)) = (
            HeaderName::from_str("origin"),
            HeaderValue::from_str(origin),
        ) {
            headers.insert(name, val);
        }
        
        // Set referer based on custom origin
        let referer = if origin.ends_with('/') {
            origin.to_string()
        } else {
            format!("{}/", origin)
        };
        
        if let (Ok(name), Ok(val)) = (
            HeaderName::from_str("referer"),
            HeaderValue::from_str(&referer),
        ) {
            headers.insert(name, val);
        }
    } else {
        // Find matching domain template and use its headers
        let hostname = url.host_str().unwrap_or("");
        if let Some(group) = DOMAIN_GROUPS.iter().find(|g| {
            g.patterns.iter().any(|pattern| {
                Regex::new(pattern).map(|re| re.is_match(hostname)).unwrap_or(false)
            })
        }) {
            // Add origin and referer from template
            if let (Ok(name), Ok(val)) = (
                HeaderName::from_str("origin"),
                HeaderValue::from_str(group.origin),
            ) {
                headers.insert(name, val);
            }
            
            if let (Ok(name), Ok(val)) = (
                HeaderName::from_str("referer"),
                HeaderValue::from_str(group.referer),
            ) {
                headers.insert(name, val);
            }

            // Add custom headers for this domain group if they exist
            if let Some(custom_headers) = &group.custom_headers {
                for (header_name, header_value) in custom_headers {
                    if let (Ok(name), Ok(val)) = (
                        HeaderName::from_str(header_name),
                        HeaderValue::from_str(header_value),
                    ) {
                        headers.insert(name, val);
                    }
                }
            }
        } else {
            // Fallback: use the URL's own origin and referer when no template is found
            let scheme = url.scheme();
            if let Some(host) = url.host_str() {
                let mut origin = String::with_capacity(scheme.len() + host.len() + 3);
                origin.push_str(scheme);
                origin.push_str("://");
                origin.push_str(host);

                if let (Ok(name), Ok(val)) = (
                    HeaderName::from_str("origin"),
                    HeaderValue::from_str(&origin),
                ) {
                    headers.insert(name, val);
                }

                // Ensure referer ends with '/'
                let mut referer = origin;
                referer.push('/');
                if let (Ok(name), Ok(val)) = (
                    HeaderName::from_str("referer"),
                    HeaderValue::from_str(&referer),
                ) {
                    headers.insert(name, val);
                }
            }
        }
    }

    headers
}


// DomainGroup {
//     patterns: vec![r"(?i)\.example\.com$"],
//     origin: "https://example.com",
//     referer: "https://example.com/",
//     custom_headers: Some(HashMap::from([
//         ("cache-control", "no-cache"),
//         ("pragma", "no-cache"),
//         ("x-custom-header", "custom-value"),
//     ])),
// },