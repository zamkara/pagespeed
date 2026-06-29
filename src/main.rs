use clap::Parser;
use serde_json::Value;
use std::io::{self, Write};

const VERSION: &str = env!("CARGO_PKG_VERSION");
const REPO: &str = "zamkara/pagespeed";

#[derive(Parser)]
#[command(name = "pagespeed", version = VERSION, about = "Google PageSpeed Insights CLI")]
struct Cli {
    /// Domain or URL to analyze
    url: Option<String>,

    /// Strategy: mobile or desktop
    #[arg(short, long, default_value = "mobile")]
    strategy: String,

    /// Categories to analyze (comma-separated: performance,accessibility,best-practices,seo)
    #[arg(short, long, default_value = "performance")]
    categories: String,

    /// Google API key (optional if PAGESPEED_API_KEY env var is set)
    #[arg(short, long, env = "PAGESPEED_API_KEY")]
    key: Option<String>,

    /// Update pagespeed to the latest release
    #[arg(short, long)]
    update: bool,
}

fn resolve_api_key(key_arg: Option<String>) -> Result<String, Box<dyn std::error::Error>> {
    if let Some(k) = key_arg {
        return Ok(k);
    }

    eprintln!("PAGESPEED_API_KEY is not set.");
    eprintln!("Get your API key at: https://console.cloud.google.com/");
    eprint!("Enter API key: ");
    io::stderr().flush()?;

    let key = rpassword::read_password()?;
    let key = key.trim().to_string();

    if key.is_empty() {
        eprintln!("API key cannot be empty.");
        std::process::exit(1);
    }

    save_api_key(&key)?;

    Ok(key)
}

fn save_api_key(key: &str) -> Result<(), Box<dyn std::error::Error>> {
    let shell = std::env::var("SHELL").unwrap_or_default();
    let home = dirs::home_dir().ok_or("Cannot find home directory")?;

    let rc_file = if shell.contains("zsh") {
        home.join(".zshrc")
    } else if shell.contains("fish") {
        home.join(".config/fish/config.fish")
    } else {
        home.join(".bashrc")
    };

    let export_line = if shell.contains("fish") {
        format!("\nset -x PAGESPEED_API_KEY \"{}\"\n", key)
    } else {
        format!("\nexport PAGESPEED_API_KEY=\"{}\"\n", key)
    };

    let mut file = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(&rc_file)?;

    file.write_all(export_line.as_bytes())?;

    eprintln!("API key saved to {}", rc_file.display());
    eprintln!("Run: source {}", rc_file.display());

    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();

    if cli.update {
        return self_update().await;
    }

    let raw_url = match cli.url {
        Some(ref u) => u.clone(),
        None => {
            eprintln!("Error: URL is required. Example: pagespeed example.com");
            eprintln!("Use --help for usage.");
            std::process::exit(1);
        }
    };

    let api_key = resolve_api_key(cli.key)?;

    let url = if raw_url.starts_with("http://") || raw_url.starts_with("https://") {
        raw_url.clone()
    } else {
        format!("https://{}", raw_url)
    };

    let categories: Vec<String> = cli
        .categories
        .split(',')
        .map(|s| s.trim().to_string())
        .collect();

    let mut query = vec![
        ("url".to_string(), url.clone()),
        ("strategy".to_string(), cli.strategy.clone()),
        ("key".to_string(), api_key),
    ];

    for cat in &categories {
        query.push(("category".to_string(), cat.clone()));
    }

    let client = reqwest::Client::new();
    let response = client
        .get("https://www.googleapis.com/pagespeedonline/v5/runPagespeed")
        .query(&query)
        .send()
        .await?;

    let status = response.status();
    let json: Value = response.json().await?;

    if !status.is_success() {
        let error_msg = json
            .pointer("/error/message")
            .and_then(|v| v.as_str())
            .unwrap_or("unknown error");
        eprintln!("API error {}: {}", status, error_msg);
        std::process::exit(1);
    }

    let output = build_report(&json, &url, &cli.strategy, &categories);

    let (folder, basename) = build_output_path(&url, &cli.strategy);
    std::fs::create_dir_all(&folder)?;

    let report_path = format!("{}/report.json", folder);
    let json_str = serde_json::to_string_pretty(&output)?;
    std::fs::write(&report_path, &json_str)?;

    let summary = build_summary(&output);
    std::fs::write(format!("{}/summary.txt", folder), &summary)?;

    eprintln!("Output folder: {}/", folder);
    eprintln!("  - report.json");
    eprintln!("  - summary.txt");

    println!("{}", basename);

    Ok(())
}

async fn self_update() -> Result<(), Box<dyn std::error::Error>> {
    eprintln!("Checking for latest release on GitHub...");

    let client = reqwest::Client::builder()
        .user_agent(format!("pagespeed/{}", VERSION))
        .build()?;

    let release: Value = client
        .get(format!("https://api.github.com/repos/{}/releases/latest", REPO))
        .send()
        .await?
        .json()
        .await?;

    let latest = release["tag_name"]
        .as_str()
        .unwrap_or("")
        .trim_start_matches('v');

    if latest.is_empty() {
        eprintln!("Failed to fetch latest version.");
        std::process::exit(1);
    }

    if latest == VERSION {
        eprintln!("Already up to date (v{}).", VERSION);
        return Ok(());
    }

    eprintln!("Current version : v{}", VERSION);
    eprintln!("Latest version  : v{}", latest);
    eprintln!("Downloading update...");

    let target = current_target();
    let ext = if cfg!(target_os = "windows") { "zip" } else { "tar.gz" };
    let asset_name = format!("pagespeed-v{}-{}.{}", latest, target, ext);

    let asset_url = release["assets"]
        .as_array()
        .and_then(|a| {
            a.iter().find(|asset| {
                asset["name"].as_str().unwrap_or("") == asset_name
            })
        })
        .and_then(|a| a["browser_download_url"].as_str())
        .map(|s| s.to_string());

    let Some(url) = asset_url else {
        eprintln!("No asset found for target: {}", target);
        eprintln!("Check releases manually: https://github.com/{}/releases", REPO);
        std::process::exit(1);
    };

    let bytes = client.get(&url).send().await?.bytes().await?;

    let current_exe = std::env::current_exe()?;
    let tmp = current_exe.with_extension("tmp");

    #[cfg(unix)]
    {
        use std::io::Read;
        let gz = flate2::read::GzDecoder::new(std::io::Cursor::new(&bytes));
        let mut archive = tar::Archive::new(gz);
        for entry in archive.entries()? {
            let mut entry = entry?;
            let path = entry.path()?;
            if path.file_name().and_then(|n| n.to_str()) == Some("pagespeed") {
                let mut buf = Vec::new();
                entry.read_to_end(&mut buf)?;
                std::fs::write(&tmp, &buf)?;
                break;
            }
        }
    }

    #[cfg(windows)]
    {
        let cursor = std::io::Cursor::new(&bytes);
        let mut archive = zip::ZipArchive::new(cursor)?;
        let mut file = archive.by_name("pagespeed.exe")?;
        let mut buf = Vec::new();
        std::io::Read::read_to_end(&mut file, &mut buf)?;
        std::fs::write(&tmp, &buf)?;
    }

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(&tmp, std::fs::Permissions::from_mode(0o755))?;
    }

    std::fs::rename(&tmp, &current_exe)?;
    eprintln!("Updated successfully to v{}.", latest);

    Ok(())
}

fn current_target() -> &'static str {
    #[cfg(all(target_os = "linux", target_arch = "x86_64"))]
    return "x86_64-unknown-linux-gnu";
    #[cfg(all(target_os = "linux", target_arch = "aarch64"))]
    return "aarch64-unknown-linux-gnu";
    #[cfg(all(target_os = "linux", target_arch = "arm"))]
    return "armv7-unknown-linux-gnueabihf";
    #[cfg(all(target_os = "macos", target_arch = "x86_64"))]
    return "x86_64-apple-darwin";
    #[cfg(all(target_os = "macos", target_arch = "aarch64"))]
    return "aarch64-apple-darwin";
    #[cfg(all(target_os = "windows", target_arch = "x86_64"))]
    return "x86_64-pc-windows-msvc";
    #[cfg(all(target_os = "windows", target_arch = "aarch64"))]
    return "aarch64-pc-windows-msvc";
    #[allow(unreachable_code)]
    "unknown"
}

fn build_output_path(url: &str, strategy: &str) -> (String, String) {
    let domain = url
        .trim_start_matches("https://")
        .trim_start_matches("http://")
        .trim_end_matches('/')
        .replace('/', "_")
        .replace(':', "_");

    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();

    let secs = now % 86400;
    let days = now / 86400;
    let h = secs / 3600;
    let m = (secs % 3600) / 60;
    let s = secs % 60;
    let (year, month, day) = epoch_days_to_date(days);

    let name = format!(
        "{}-{}-{:04}{:02}{:02}-{:02}{:02}{:02}",
        domain, strategy, year, month, day, h, m, s
    );
    (name.clone(), name)
}

fn build_summary(report: &Value) -> String {
    let mut out = String::new();

    out.push_str(&format!("URL      : {}\n", report["url"].as_str().unwrap_or("-")));
    out.push_str(&format!("Strategy : {}\n", report["strategy"].as_str().unwrap_or("-")));
    out.push_str(&format!("Date     : {}\n", report["fetch_time"].as_str().unwrap_or("-")));
    out.push('\n');
    out.push_str("=== SCORES ===\n");
    if let Some(scores) = report["scores"].as_object() {
        for (k, v) in scores {
            out.push_str(&format!("  {:<20} {}\n", k, v));
        }
    }
    out.push('\n');
    out.push_str("=== KEY METRICS ===\n");
    let metric_keys = [
        "first-contentful-paint",
        "largest-contentful-paint",
        "total-blocking-time",
        "cumulative-layout-shift",
        "speed-index",
        "interactive",
    ];
    for key in &metric_keys {
        if let Some(m) = report["audits"][key].as_object() {
            let val = m.get("displayValue").and_then(|v| v.as_str()).unwrap_or("-");
            let score = m.get("score").and_then(|v| v.as_f64()).unwrap_or(0.0);
            out.push_str(&format!("  {:<35} {} (score: {:.2})\n", key, val, score));
        }
    }
    out.push('\n');
    out.push_str("=== ISSUES (score < 1) ===\n");
    if let Some(audits) = report["audits"].as_object() {
        for (key, audit) in audits {
            let score = audit["score"].as_f64().unwrap_or(1.0);
            if score < 1.0 && !audit["score"].is_null() {
                let title = audit["title"].as_str().unwrap_or(key);
                let dv = audit["displayValue"].as_str().unwrap_or("");
                out.push_str(&format!("  [{:.2}] {} {}\n", score, title, dv));
            }
        }
    }
    out
}

fn epoch_days_to_date(days: u64) -> (u64, u64, u64) {
    let z = days + 719468;
    let era = z / 146097;
    let doe = z % 146097;
    let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146096) / 365;
    let y = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    let y = if m <= 2 { y + 1 } else { y };
    (y, m, d)
}

fn build_report(raw: &Value, url: &str, strategy: &str, categories: &[String]) -> Value {
    let lhr = &raw["lighthouseResult"];

    let scores: serde_json::Map<String, Value> = categories
        .iter()
        .filter_map(|cat| {
            let score = lhr
                .pointer(&format!("/categories/{}/score", cat))
                .and_then(|v| v.as_f64())
                .map(|s| (s * 100.0).round() as u64);
            score.map(|s| (cat.clone(), Value::Number(s.into())))
        })
        .collect();

    let audits = extract_audits(lhr);

    serde_json::json!({
        "url": url,
        "strategy": strategy,
        "fetch_time": lhr["fetchTime"],
        "requested_url": lhr["requestedUrl"],
        "final_url": lhr["finalUrl"],
        "lighthouse_version": lhr["lighthouseVersion"],
        "user_agent": lhr["userAgent"],
        "environment": lhr["environment"],
        "timing": lhr["timing"],
        "config_settings": lhr["configSettings"],
        "scores": scores,
        "categories": lhr["categories"],
        "category_groups": lhr["categoryGroups"],
        "audits": audits,
        "i18n": lhr["i18n"],
        "stack_packs": lhr["stackPacks"],
        "entities": lhr["entities"],
        "full_page_screenshot": lhr["fullPageScreenshot"],
    })
}

fn extract_audits(lhr: &Value) -> Value {
    let Some(audits_obj) = lhr["audits"].as_object() else {
        return Value::Object(serde_json::Map::new());
    };

    let mut map = serde_json::Map::new();
    for (key, audit) in audits_obj {
        map.insert(key.clone(), audit.clone());
    }
    Value::Object(map)
}
