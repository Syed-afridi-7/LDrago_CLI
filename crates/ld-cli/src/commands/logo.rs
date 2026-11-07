use anyhow::Result;

/// TrueColor RGB styling helper
fn bold_rgb(r: u8, g: u8, b: u8, text: &str) -> String {
    format!("\x1b[1;38;2;{};{};{}m{}\x1b[0m", r, g, b, text)
}

fn dim_rgb(r: u8, g: u8, b: u8, text: &str) -> String {
    format!("\x1b[38;2;{};{};{}m{}\x1b[0m", r, g, b, text)
}

/// Strip ANSI escape codes from a formatted string
pub fn strip_ansi(s: &str) -> String {
    let mut out = String::new();
    let mut in_escape = false;
    for c in s.chars() {
        if c == '\x1b' {
            in_escape = true;
        } else if in_escape {
            if c == 'm' {
                in_escape = false;
            }
        } else {
            out.push(c);
        }
    }
    out
}

/// Render the clean 1-line technical banner
pub fn render_banner(model_name: &str) -> String {
    let cyan_glyph = bold_rgb(0, 220, 255, ">_ LDrago");
    let dot = dim_rgb(120, 120, 120, "•");
    let local = bold_rgb(220, 220, 220, "Local");
    let auto = bold_rgb(220, 220, 220, "Autonomous");
    let model = bold_rgb(255, 255, 255, &format!("[{model_name}]"));
    format!("{cyan_glyph} {dot} {local} {dot} {auto} {dot} {model}")
}

/// Execute the minimal logo command
pub fn execute_logo() -> Result<()> {
    println!("\n{}\n", render_banner("Coding"));
    Ok(())
}

/// Backwards-compatible banner printer
pub fn render_ldrago_dragon() {
    println!("{}", render_banner("Coding"));
}
