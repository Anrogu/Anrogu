use gif::{Encoder, Frame, Repeat};
use reqwest::blocking::Client;
use serde::Deserialize;
use std::collections::HashMap;
use std::env;
use std::fs::File;
use std::io::Write;

fn hex_to_rgb(hex: &str) -> (u8, u8, u8) {
    let hex = hex.trim_start_matches('#');
    let r = u8::from_str_radix(&hex[0..2], 16).unwrap_or(0);
    let g = u8::from_str_radix(&hex[2..4], 16).unwrap_or(0);
    let b = u8::from_str_radix(&hex[4..6], 16).unwrap_or(0);
    (r, g, b)
}

fn lerp_rgb(a: (u8, u8, u8), b: (u8, u8, u8), t: f32) -> (u8, u8, u8) {
    (
        (a.0 as f32 + (b.0 as f32 - a.0 as f32) * t).round() as u8,
        (a.1 as f32 + (b.1 as f32 - a.1 as f32) * t).round() as u8,
        (a.2 as f32 + (b.2 as f32 - a.2 as f32) * t).round() as u8,
    )
}

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

    let top_language = top_langs
        .get(0)
        .map(|(l, _)| l.clone())
        .unwrap_or_else(|| "Unknown".to_string());

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
        r#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 {} {}" width="{}" height="{}" shape-rendering="crispEdges">
  <defs>
    <filter id="lure-glow" x="-50%" y="-50%" width="200%" height="200%">
      <feGaussianBlur stdDeviation="3" result="blur" />
      <feMerge>
        <feMergeNode in="blur" />
        <feMergeNode in="SourceGraphic" />
      </feMerge>
    </filter>
  </defs>
  <g id="anglerfish">
    <animateTransform attributeName="transform" attributeType="XML" type="translate" values="0 0;0 -6;0 0" dur="2.4s" repeatCount="indefinite" />
    <animateTransform attributeName="transform" attributeType="XML" type="rotate" values="0 {} {};2 {} {};0 {} {};-2 {} {};0 {} {}" dur="4.8s" repeatCount="indefinite" additive="sum" />"#,
        width, height, width, height,
        width / 2, height / 2,
        width / 2, height / 2,
        width / 2, height / 2,
        width / 2, height / 2,
        width / 2, height / 2,
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
                let is_lure = ch == 'L';
                let is_pupil = ch == 'E';
                if is_lure {
                    svg.push_str(&format!(
                        r#"<rect x="{}" y="{}" width="{}" height="{}" fill="{}" filter="url(#lure-glow)">
      <animate attributeName="opacity" values="1;0.35;1" dur="1.2s" repeatCount="indefinite" />
      <animate attributeName="fill" values="{};#ffffff;{}" dur="1.2s" repeatCount="indefinite" />
    </rect>"#,
                        x * pixel_size,
                        y * pixel_size,
                        pixel_size,
                        pixel_size,
                        hex,
                        hex,
                        hex
                    ));
                } else if is_pupil {
                    svg.push_str(&format!(
                        r#"<rect x="{}" y="{}" width="{}" height="{}" fill="{}">
      <animate attributeName="opacity" values="1;1;0;1" keyTimes="0;0.88;0.92;1" dur="4s" repeatCount="indefinite" />
    </rect>"#,
                        x * pixel_size,
                        y * pixel_size,
                        pixel_size,
                        pixel_size,
                        hex
                    ));
                } else {
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
    }

    svg.push_str("</g></svg>");

    let mut file = File::create("anglerfish.svg").expect("Unable to create anglerfish.svg");
    file.write_all(svg.as_bytes()).expect("Unable to write SVG data");

    // --- Animated GIF for GitHub README (SVG animations are stripped by GitHub) ---
    let frames_count: usize = 24;
    let primary_rgb = hex_to_rgb(primary_color);
    let lure_rgb = hex_to_rgb(lure_color);
    let fin_rgb = hex_to_rgb(fin_color);
    let teeth_rgb = hex_to_rgb("#e6edf3");
    let pupil_rgb = hex_to_rgb("#0d1117");
    let white = (255u8, 255u8, 255u8);

    // Glow variants for the lure (8 steps between lure color and white)
    let mut lure_variants: Vec<(u8, u8, u8)> = Vec::new();
    for i in 0..8 {
        lure_variants.push(lerp_rgb(lure_rgb, white, i as f32 / 7.0));
    }

    // Build a deduplicated palette (index 0 = transparent)
    let mut pal: Vec<(u8, u8, u8)> = vec![(0, 0, 0)];
    let mut add_color = |c: (u8, u8, u8)| -> u8 {
        if let Some(i) = pal.iter().position(|&x| x == c) {
            return i as u8;
        }
        pal.push(c);
        (pal.len() - 1) as u8
    };

    let idx_primary = add_color(primary_rgb);
    let idx_fin = add_color(fin_rgb);
    let idx_teeth = add_color(teeth_rgb);
    let idx_pupil = add_color(pupil_rgb);
    let idx_lure: Vec<u8> = lure_variants.iter().map(|&v| add_color(v)).collect();

    // Flatten palette to RGB triples
    let mut palette_bytes: Vec<u8> = Vec::with_capacity(pal.len() * 3);
    for (r, g, b) in &pal {
        palette_bytes.push(*r);
        palette_bytes.push(*g);
        palette_bytes.push(*b);
    }

    let width = (16 * pixel_size) as u16;
    let height = (12 * pixel_size) as u16;

    let mut gif_file = File::create("anglerfish.gif").expect("Unable to create anglerfish.gif");
    let mut encoder = Encoder::new(&mut gif_file, width, height, &palette_bytes)
        .expect("Unable to create GIF encoder");
    encoder.set_repeat(Repeat::Infinite).expect("Unable to set repeat");

    for i in 0..frames_count {
        let t = i as f32 / frames_count as f32;
        let bob_y = (4.0 * (t * std::f32::consts::TAU).sin()).round() as i32;
        let sway_x = (2.0 * (t * std::f32::consts::TAU + std::f32::consts::FRAC_PI_2).sin()).round() as i32;

        // Lure glow index (pulses twice per loop)
        let glow_t = ((t * 2.0 * std::f32::consts::TAU).sin() + 1.0) / 2.0;
        let glow_idx = (glow_t * 7.0).round() as usize;
        let glow_idx = glow_idx.min(idx_lure.len() - 1);
        let lure_color_idx = idx_lure[glow_idx];

        // Pupil blink: hidden for 2 frames every 12
        let pupil_visible = !(i % 12 == 0 || i % 12 == 1);

        let mut buffer = vec![0u8; (width as usize) * (height as usize)];

        for (y, row) in grid.iter().enumerate() {
            for (x, ch) in row.chars().enumerate() {
                let color_idx: u8 = match ch {
                    'B' => idx_primary,
                    'L' => lure_color_idx,
                    'F' => idx_fin,
                    'T' => idx_teeth,
                    'E' => {
                        if pupil_visible {
                            idx_pupil
                        } else {
                            continue;
                        }
                    }
                    _ => continue,
                };

                for yy in 0..pixel_size {
                    for xx in 0..pixel_size {
                        let px = (x * pixel_size) as i32 + xx as i32 + sway_x;
                        let py = (y * pixel_size) as i32 + yy as i32 + bob_y;
                        if px >= 0 && py >= 0 && px < width as i32 && py < height as i32 {
                            buffer[(py as usize) * (width as usize) + (px as usize)] = color_idx;
                        }
                    }
                }
            }
        }

        let mut frame = Frame::default();
        frame.width = width;
        frame.height = height;
        frame.buffer = buffer.into();
        frame.delay = 8; // centiseconds (~12.5 fps)
        frame.transparent = Some(0);
        encoder.write_frame(&frame).expect("Unable to write GIF frame");
    }

    // encoder finalizes on drop

    // Update README.md with the most used language
    if let Ok(readme) = std::fs::read_to_string("README.md") {
        let start_marker = "<!-- MOST_USED_LANGUAGE_START -->";
        let end_marker = "<!-- MOST_USED_LANGUAGE_END -->";
        if let (Some(start_idx), Some(end_idx)) = (readme.find(start_marker), readme.find(end_marker)) {
            let before = &readme[..start_idx];
            let after = &readme[end_idx + end_marker.len()..];
            let updated = format!(
                "{}{}\nmy most used language: {}\n{}{}",
                before, start_marker, top_language, end_marker, after
            );
            let _ = std::fs::write("README.md", updated);
        }
    }
}
