use reqwest::blocking::Client;
use serde::Deserialize;
use std::collections::HashMap;
use std::env;
use std::fs::File;
use std::io::Write;

#[derive(Deserialize)]
struct Repo {
    languages_url: String,
    fork: bool,
}

/// Official GitHub Linguist hex colors for common languages
fn get_language_color(lang: &str) -> &'static str {
    match lang {
        "Java" => "#b07219",
        "JavaScript" => "#f1e05a",
        "TypeScript" => "#3178c6",
        "C#" => "#178600",
        "Rust" => "#dea584",
        "Python" => "#3572A5",
        "HTML" => "#e34c26",
        "CSS" => "#563d7c",
        "C++" => "#f34b7d",
        "C" => "#555555",
        "Shell" => "#89e051",
        "Dockerfile" => "#384d54",
        _ => "#58a6ff", // Default fallback blue
    }
}

/// Fetches total bytes per language across non-forked repositories
fn fetch_top_languages(username: &str, token: Option<&str>) -> Vec<(String, u64)> {
    let client = Client::new();
    let url = format!("https://api.github.com/users/{}/repos?per_page=100", username);

    let mut req = client.get(&url).header("User-Agent", "rust-anglerfish-readme");
    if let Some(t) = token {
        req = req.bearer_auth(t);
    }

    let repos: Vec<Repo> = req
        .send()
        .expect("Failed to contact GitHub API")
        .json()
        .expect("Failed to parse repo JSON");

    let mut totals: HashMap<String, u64> = HashMap::new();

    for repo in repos.into_iter().filter(|r| !r.fork) {
        let mut l_req = client
            .get(&repo.languages_url)
            .header("User-Agent", "rust-anglerfish-readme");
        if let Some(t) = token {
            l_req = l_req.bearer_auth(t);
        }

        if let Ok(resp) = l_req.send() {
            if let Ok(langs) = resp.json::<HashMap<String, u64>>() {
                for (lang, bytes) in langs {
                    *totals.entry(lang).or_insert(0) += bytes;
                }
            }
        }
    }

    let mut sorted: Vec<(String, u64)> = totals.into_iter().collect();
    sorted.sort_by(|a, b| b.1.cmp(&a.1));
    sorted
}

fn main() {
    let username = env::var("GH_USERNAME").unwrap_or_else(|_| "Anrogu".to_string());
    let token = env::var("GH_TOKEN").ok();

    let top_langs = fetch_top_languages(&username, token.as_deref());

    // Map top 3 languages to regions of the anglerfish:
    // #1 Most used -> Main Body ('B')
    // #2 Second most used -> Glowing Lure & Eye ('L')
    // #3 Third most used -> Fins & Stalk ('F')
    let primary_color = top_langs.get(0).map(|(l, _)| get_language_color(l)).unwrap_or("#b07219");
    let lure_color = top_langs.get(1).map(|(l, _)| get_language_color(l)).unwrap_or("#f1e05a");
    let fin_color = top_langs.get(2).map(|(l, _)| get_language_color(l)).unwrap_or("#3178c6");

    // 16x12 Anglerfish Pixel Matrix
    // '.' = empty, 'L' = glowing esca/eye, 'F' = stalk/fins, 'B' = body, 'T' = teeth, 'E' = pupil
    let grid = [
        "..LL..FFFF......",
        ".LLLLF....F.....",
        "..LL......F...F.",
        ".........BBBB.FF",
        ".......BBBBBBBFF",
        "......BBLBBBBBBF",
        ".....BBLEBBBBBBB",
        ".....BBBBBBBBBBF",
        ".....TTTTBBBBBBF",
        "........BBBBBB.F",
        ".....TTTTBBBB...",
        "......BBBBBB....",
    ];

    let pixel_size = 20;
    let width = 16 * pixel_size;
    let height = 12 * pixel_size;

    let mut svg = format!(
        r#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 {} {}" width="{}" height="{}" shape-rendering="crispEdges">"#,
        width, height, width, height
    );

    for (y, row) in grid.iter().enumerate() {
        for (x, ch) in row.chars().enumerate() {
            let color = match ch {
                'B' => Some(primary_color),
                'L' => Some(lure_color),
                'F' => Some(fin_color),
                'T' => Some("#e6edf3"), // Off-white teeth
                'E' => Some("#0d1117"), // Dark pupil
                _ => None,
            };

            if let Some(hex) = color {
                svg.push_str(&format!(
                    r#"<rect x="{}" y="{}" width="{}" height="{}" fill="{}" />"#,
                    x * pixel_size,
                    y * pixel_size,
                    pixel_size,
                    pixel_size,
                    hex
                ));
            }
        }
    }

    svg.push_str("</svg>");

    let mut file = File::create("anglerfish.svg").expect("Unable to create anglerfish.svg");
    file.write_all(svg.as_bytes()).expect("Unable to write SVG data");
}
