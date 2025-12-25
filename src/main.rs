use rand::seq::IndexedRandom as _;
use std::env;
use std::fs;
use std::io::{self, Write};
use std::path::PathBuf;
use std::thread::sleep;
use std::time::{Duration, Instant};
use unicode_width::UnicodeWidthChar;

fn get_lang_code() -> String {
    env::var("LANG")
        .ok()
        .and_then(|lang| {
            lang.split('.')
                .next()
                .map(|s| s.replace("_", "-").to_lowercase())
        })
        .unwrap_or_else(|| "en-us".to_string())
}

fn read_texts(lang_code: &str) -> String {
    let i18n_path = PathBuf::from("i18n").join(format!("{}.txt", lang_code));
    let fallback_path = PathBuf::from("i18n").join("en-us.txt");
    fs::read_to_string(&i18n_path)
        .unwrap_or_else(|_| fs::read_to_string(&fallback_path).unwrap_or_default())
}

fn get_code_files() -> Vec<PathBuf> {
    fs::read_dir("code")
        .map(|rd| {
            rd.filter_map(|e| {
                let e = e.ok()?;
                let path = e.path();
                if path.is_file() { Some(path) } else { None }
            })
            .collect()
        })
        .unwrap_or_default()
}

fn render_frame(texts: &str, pre_run: &dyn Fn(), i: usize, char_delay: f32) -> bool {
    let mut not_end = false;
    let start_time = Instant::now();
    // let columns = term_size::dimensions().map(|(w, _)| w).unwrap_or(80);
    let columns = 80;
    print!("\x1b[H");
    pre_run();
    for (l, line) in texts.lines().enumerate() {
        let mut text_list = Vec::new();
        for ch in line.chars() {
            if ch == '\t' {
                text_list.push((8, ch));
                for _ in 0..7 {
                    text_list.push((0, '\0'));
                }
            } else if UnicodeWidthChar::width(ch).unwrap_or(1) == 2 {
                text_list.push((2, ch));
                text_list.push((0, '\0'));
            } else {
                text_list.push((1, ch));
            }
        }
        let mut chars = vec![' '; columns];
        for (j, (w, ch)) in text_list.iter().enumerate() {
            if *ch == '\0' {
                continue;
            }
            let t = i as f32 / 100.0 - j as f32 * char_delay - l as f32 * 0.5;
            if t > 1.0 {
                if j + *w - 1 < columns {
                    chars[j] = *ch;
                    for k in 1..*w {
                        chars[j + k] = '\0';
                    }
                }
                continue;
            }
            not_end = true;
            if t < 0.0 {
                continue;
            }
            let n = ((1.0 - t).powi(3) * columns as f32) as usize;
            if j + n + *w - 1 < columns {
                chars[j + n] = *ch;
                for k in 1..*w {
                    chars[j + n + k] = '\0';
                }
            }
        }
        let line: String = chars.iter().filter(|&&c| c != '\0').collect();
        println!("{}", line);
    }
    io::stdout().flush().unwrap();
    let elapsed = start_time.elapsed();
    if elapsed < Duration::from_millis(10) {
        sleep(Duration::from_millis(10) - elapsed);
    }
    not_end
}

fn main() {
    print!("\x1b[?1049h\x1b[?25l");
    io::stdout().flush().unwrap();
    let lang_code = get_lang_code();
    let texts = read_texts(&lang_code);
    let files = get_code_files();
    let char_delay = 0.05;
    let mut i = 0;
    while render_frame(&texts, &|| {}, i, char_delay) {
        i += 1;
    }
    let mut rng = rand::rng();
    loop {
        let random_file = files.choose(&mut rng).unwrap();
        let code = fs::read_to_string(random_file).unwrap_or_default();
        let first_line = texts.lines().next().unwrap_or("");
        let code = code.replace("$$$", first_line);
        print!("\x1b[H\x1b[2J");
        let mut i = 0;
        while render_frame(&code, &|| println!("{texts}"), i, char_delay) {
            i += 1;
        }
        sleep(Duration::from_secs(1));
    }
    print!("\x1b[?25h\x1b[?1049l");
    println!("{}", texts);
}
