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

// Define domain group configuration
struct DomainGroup {
    patterns: Vec<&'static str>,
    origin: &'static str,
    referer: &'static str,
    sec_fetch_site: &'static str,
    use_cache_headers: bool,
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
            sec_fetch_site: "cross-site",
            use_cache_headers: false,
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
            sec_fetch_site: "cross-site",
            use_cache_headers: false,
        },
        DomainGroup {
            patterns: vec![r"(?i)\.akamaized\.net$"],
            origin: "https://players.akamai.com",
            referer: "https://players.akamai.com/",
            sec_fetch_site: "cross-site",
            use_cache_headers: false,
        },
        DomainGroup {
            patterns: vec![r"(?i)(?:^|\.)shadowlandschronicles\."],
            origin: "https://cloudnestra.com",
            referer: "https://cloudnestra.com/",
            sec_fetch_site: "cross-site",
            use_cache_headers: false,
        },        
        DomainGroup {
            patterns: vec![r"(?i)(?:^|\.)viddsn\."],
            origin: "https://vidwish.live/",
            referer: "https://vidwish.live/",
            sec_fetch_site: "cross-site",
            use_cache_headers: false,
        },
        DomainGroup {
            patterns: vec![r"(?i)(?:^|\.)dotstream\.", r"(?i)(?:^|\.)playcloud1\."],
            origin: "https://megaplay.buzz/",
            referer: "https://megaplay.buzz/",
            sec_fetch_site: "cross-site",
            use_cache_headers: false,
        },        
        DomainGroup {
            patterns: vec![r"(?i)\.cloudfront\.net$"],
            origin: "https://d2zihajmogu5jn.cloudfront.net",
            referer: "https://d2zihajmogu5jn.cloudfront.net/",
            sec_fetch_site: "same-site",
            use_cache_headers: false,
        },
        DomainGroup {
            patterns: vec![r"(?i)\.ttvnw\.net$"],
            origin: "https://www.twitch.tv",
            referer: "https://www.twitch.tv/",
            sec_fetch_site: "same-site",
            use_cache_headers: false,
        },
        DomainGroup {
            patterns: vec![r"(?i)\.xx\.fbcdn\.net$"],
            origin: "https://www.facebook.com",
            referer: "https://www.facebook.com/",
            sec_fetch_site: "same-site",
            use_cache_headers: false,
        },
        DomainGroup {
            patterns: vec![r"(?i)\.anih1\.top$", r"(?i)\.xyk3\.top$"],
            origin: "https://ee.anih1.top",
            referer: "https://ee.anih1.top/",
            sec_fetch_site: "cross-site",
            use_cache_headers: false,
        },
        DomainGroup {
            patterns: vec![r"(?i)\.premilkyway\.com$"],
            origin: "https://uqloads.xyz",
            referer: "https://uqloads.xyz/",
            sec_fetch_site: "cross-site",
            use_cache_headers: false,
        },
        DomainGroup {
            patterns: vec![r"(?i)\.streamcdn\.com$"],
            origin: "https://anime.uniquestream.net",
            referer: "https://anime.uniquestream.net/",
            sec_fetch_site: "cross-site",
            use_cache_headers: false,
        },
        DomainGroup {
            patterns: vec![r"(?i)\.raffaellocdn\.net$", r"(?i)\.feetcdn\.com$", r"(?i)clearskydrift45\.site$"],
            origin: "https://kerolaunochan.online",
            referer: "https://kerolaunochan.online/",
            sec_fetch_site: "cross-site",
            use_cache_headers: false,
        },
        DomainGroup {
            patterns: vec![r"(?i)dewbreeze84\.online$", r"(?i)cloudydrift38\.site$", r"(?i)sunshinerays93\.live$", r"(?i)clearbluesky72\.wiki$", r"(?i)breezygale56\.online$", r"(?i)frostbite27\.pro$", r"(?i)frostywinds57\.live$", r"(?i)icyhailstorm64\.wiki$", r"(?i)icyhailstorm29\.online$", r"(?i)windflash93\.xyz$", r"(?i)stormdrift27\.site$", r"(?i)tempestcloud61\.wiki$", r"(?i)sunburst66\.pro$", r"(?i)douvid\.xyz$"],
            origin: "https://megacloud.blog",
            referer: "https://megacloud.blog/",
            sec_fetch_site: "cross-site",
            use_cache_headers: false,
        },
        DomainGroup {
            patterns: vec![r"(?i)\.echovideo\.to$"],
            origin: "https://aniwave.at",
            referer: "https://aniwave.at/",
            sec_fetch_site: "cross-site",
            use_cache_headers: false,
        },
        DomainGroup {
            patterns: vec![r"(?i)\.vid-cdn\.xyz$"],
            origin: "https://anizone.to/",
            referer: "https://anizone.to/",
            sec_fetch_site: "cross-site",
            use_cache_headers: false,
        },
        DomainGroup {
            patterns: vec![r"(?i)\.1stkmgv1\.com$"],
            origin: "https://animeyy.com",
            referer: "https://animeyy.com/",
            sec_fetch_site: "cross-site",
            use_cache_headers: false,
        },
        DomainGroup {
            patterns: vec![r"(?i)lightningspark77\.pro$", r"(?i)thunderwave48\.xyz$", r"(?i)stormwatch95\.site$", r"(?i)windyrays29\.online$", r"(?i)thunderstrike77\.online$", r"(?i)lightningflash39\.live$", r"(?i)cloudburst82\.xyz$", r"(?i)drizzleshower19\.site$", r"(?i)rainstorm92\.xyz$"],
            origin: "https://megacloud.club",
            referer: "https://megacloud.club/",
            sec_fetch_site: "cross-site",
            use_cache_headers: false,
        },
        DomainGroup {
            patterns: vec![r"(?i)vmeas\.cloud$"],
            origin: "https://vidmoly.to",
            referer: "https://vidmoly.to/",
            sec_fetch_site: "cross-site",
            use_cache_headers: false,
        },
        DomainGroup {
            patterns: vec![r"(?i)nextwaveinitiative\.xyz$", r"(?i)shadowlandschronicles\.com$"],
            origin: "https://edgedeliverynetwork.org",
            referer: "https://edgedeliverynetwork.org/",
            sec_fetch_site: "cross-site",
            use_cache_headers: false,
        },
        DomainGroup {
            patterns: vec![r"(?i)lightningbolts\.ru$", r"(?i)lightningbolt\.site$", r"(?i)vyebzzqlojvrl\.top$"],
            origin: "https://vidsrc.cc",
            referer: "https://vidsrc.cc/",
            sec_fetch_site: "cross-site",
            use_cache_headers: false,
        },
        DomainGroup {
            patterns: vec![r"(?i)vidlvod\.store$"],
            origin: "https://vidlink.pro",
            referer: "https://vidlink.pro/",
            sec_fetch_site: "cross-site",
            use_cache_headers: false,
        },
        DomainGroup {
            patterns: vec![r"(?i)sunnybreeze16\.live$"],
            origin: "https://megacloud.store",
            referer: "https://megacloud.store/",
            sec_fetch_site: "cross-site",
            use_cache_headers: false,
        },
        DomainGroup {
            patterns: vec![r"(?i)heatwave90\.pro$", r"(?i)humidmist27\.wiki$", r"(?i)frozenbreeze65\.live$", r"(?i)drizzlerain73\.online$", r"(?i)sunrays81\.xyz$"],
            origin: "https://kerolaunochan.live",
            referer: "https://kerolaunochan.live/",
            sec_fetch_site: "cross-site",
            use_cache_headers: false,
        },
        DomainGroup {
            patterns: vec![r"(?i)\.vkcdn5\.com$"],
            origin: "https://vkspeed.com",
            referer: "https://vkspeed.com/",
            sec_fetch_site: "cross-site",
            use_cache_headers: false,
        },
        DomainGroup {
            patterns: vec![r"(?i)embed\.su$", r"(?i)usbigcdn\.cc$", r"(?i)\.congacdn\.cc$"],
            origin: "https://embed.su",
            referer: "https://embed.su/",
            sec_fetch_site: "cross-site",
            use_cache_headers: false,
        },
    ]
});

// Define DomainTemplate struct with configuration
pub struct DomainTemplate {
    pub pattern: Regex,
    pub origin: String,
    pub referer: String,
    pub sec_fetch_site: String,
    pub use_cache_headers: bool,
}

impl DomainTemplate {
    fn generate_headers(&self) -> HeaderMap {
        let mut headers = HeaderMap::new();
        let base_headers = DEFAULT_HEADERS.clone();

        // Add base headers, overriding sec-fetch-site
        for (key, value) in base_headers.iter() {
            let final_value = if key == "sec-fetch-site" {
                &self.sec_fetch_site
            } else {
                value
            };
            if let (Ok(name), Ok(val)) = (
                HeaderName::from_str(key),
                HeaderValue::from_str(final_value),
            ) {
                headers.insert(name, val);
            }
        }

        // Add cache headers if needed
        if self.use_cache_headers {
            if let (Ok(name), Ok(val)) = (
                HeaderName::from_str("cache-control"),
                HeaderValue::from_str("no-cache"),
            ) {
                headers.insert(name, val);
            }
            if let (Ok(name), Ok(val)) = (
                HeaderName::from_str("pragma"),
                HeaderValue::from_str("no-cache"),
            ) {
                headers.insert(name, val);
            }
        }

        // Add origin and referer
        if let (Ok(name), Ok(val)) = (
            HeaderName::from_str("origin"),
            HeaderValue::from_str(&self.origin),
        ) {
            headers.insert(name, val);
        }
        if let (Ok(name), Ok(val)) = (
            HeaderName::from_str("referer"),
            HeaderValue::from_str(&self.referer),
        ) {
            headers.insert(name, val);
        }

        headers
    }
}

// Static domain templates
pub static DOMAIN_TEMPLATES: Lazy<Vec<DomainTemplate>> = Lazy::new(|| {
    let mut templates = Vec::new();

    for group in DOMAIN_GROUPS.iter() {
        let origin = group.origin.to_string();
        let referer = group.referer.to_string();
        let sec_fetch_site = group.sec_fetch_site.to_string();
        let use_cache_headers = group.use_cache_headers;

        // Special case for megacloud.blog domains with cache headers
        let cache_patterns = vec![r"(?i)frostbite27\.pro$", r"(?i)icyhailstorm64\.wiki$"];

        for pattern in &group.patterns {
            let pattern = Regex::new(pattern).unwrap();
            let is_cache_pattern = cache_patterns.contains(&&pattern.as_str());
            let effective_use_cache_headers = use_cache_headers || is_cache_pattern;

            templates.push(DomainTemplate {
                pattern,
                origin: origin.clone(),
                referer: referer.clone(),
                sec_fetch_site: sec_fetch_site.clone(),
                use_cache_headers: effective_use_cache_headers,
            });
        }
    }

    // Add catch-all template
    templates.push(DomainTemplate {
        pattern: Regex::new(r"(?i).*").unwrap(),
        origin: "".to_string(),
        referer: "".to_string(),
        sec_fetch_site: "cross-site".to_string(),
        use_cache_headers: false,
    });

    templates
});

// Find matching template for a URL
pub fn find_template_for_domain(url: &Url) -> &DomainTemplate {
    let hostname = url.host_str().unwrap_or("");
    DOMAIN_TEMPLATES
        .iter()
        .find(|template| template.pattern.is_match(hostname))
        .unwrap_or_else(|| DOMAIN_TEMPLATES.last().unwrap())
}

// Generate headers for a URL
pub fn generate_headers_for_url(url: &Url) -> HeaderMap {
    let template = find_template_for_domain(url);
    template.generate_headers()
}