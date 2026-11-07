use anyhow::Result;
use clap::ValueEnum;

#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
pub enum LogoTarget {
    Ldrago,
    Claude,
    Grok,
    Ag,
    All,
}

// TrueColor RGB styling helpers
fn rgb(r: u8, g: u8, b: u8, text: &str) -> String {
    format!("\x1b[38;2;{};{};{}m{}\x1b[0m", r, g, b, text)
}

fn bold_rgb(r: u8, g: u8, b: u8, text: &str) -> String {
    format!("\x1b[1;38;2;{};{};{}m{}\x1b[0m", r, g, b, text)
}

/// 🐉 L-Drago (Beyblade Dragon Logo)
/// 3-color fusion: Golden Crest + Crimson Jaws/Flames + Electric Cyan/Blue Aura & Eyes
pub fn render_ldrago_dragon() {
    let gold = |s: &str| bold_rgb(255, 215, 0, s);
    let amber = |s: &str| bold_rgb(245, 158, 11, s);
    let red = |s: &str| bold_rgb(255, 34, 51, s);
    let dark_red = |s: &str| bold_rgb(185, 28, 28, s);
    let cyan = |s: &str| bold_rgb(0, 240, 255, s);
    let blue = |s: &str| bold_rgb(37, 99, 235, s);

    println!();
    // Dragon Horns & Celestial Crest (Golden Flame)
    println!("             {}", gold("▲                   ▲                   ▲"));
    println!("            {}", gold("█▓▲                 █▓▲                 ▲▓█"));
    println!("           {}", gold("████▲       ▲     ▲█████▲     ▲       ▲████"));
    println!("          {}", gold("███████▲   ▲█▓▲ ▲█████████▲ ▲▓█▲   ▲███████"));
    println!("         {}", amber("█████████▓▄█████▄███████████▄█████▄▓█████████"));
    
    // Dragon Brows & Electric Cyan Eyes
    print!("        {}", red("███████"));
    print!("{}", cyan("◤● █ ●◥"));
    print!("{}", gold("█████████████"));
    print!("{}", cyan("◤● █ ●◥"));
    println!("{}", red("███████"));

    print!("       {}", dark_red("████████"));
    print!("{}", cyan("▼ ⚡ ⚡ ▼"));
    print!("{}", gold("███████"));
    print!("{}", red("▲"));
    print!("{}", gold("███████"));
    print!("{}", cyan("▼ ⚡ ⚡ ▼"));
    println!("{}", dark_red("████████"));

    // Fierce Dragon Jaws & Left-Spin Vortex (Crimson Red & Electric Blue)
    println!("       {}", red(" ▀█████████████████████████████████████████▀ "));
    println!("        {}", red("   ▀█████████") + &blue("▓▒░ ◄ LEFT-SPIN ► ░▒▓") + &red("█████████▀   "));
    println!("          {}", red("     ▀████████") + &cyan("═══ VORTEX ═══") + &red("████████▀     "));
    println!("            {}", dark_red("       ▀█████████████████████▀       "));
    println!("              {}", amber("           ▀███████████▀           "));
    println!("                {}", gold("               ▀███▀               "));
    println!("                  {}", cyan("                 ▼                 "));

    // Title Banner
    println!();
    print!("  {}", red("⚡ ═══ "));
    print!("{}", gold("L - D R A G O"));
    print!(" ");
    print!("{}", cyan("B E Y B L A D E"));
    print!(" ");
    print!("{}", red("D R A G O N"));
    println!(" {}", red("═══ ⚡"));

    println!(
        "  {}",
        amber("  • THE DRAGON EMPEROR • METEO LEFT-SPIN NEURAL RECURRENT CORE •")
    );
    println!(
        "  {}",
        blue("  • Colors: ")
            + &gold("Golden Horns")
            + &blue(" | ")
            + &red("Crimson Fangs")
            + &blue(" | ")
            + &cyan("Electric Azure Aura")
    );
    println!();
}

/// 🌌 Anti-Gravity (AG Logo)
/// Quantum levitation, celestial cyan & deep purple floating delta crystal
pub fn render_ag_logo() {
    let cyan = |s: &str| bold_rgb(0, 255, 255, s);
    let purple = |s: &str| bold_rgb(168, 85, 247, s);
    let indigo = |s: &str| bold_rgb(99, 102, 241, s);
    let slate = |s: &str| rgb(148, 163, 184, s);

    println!();
    println!("                   {}", cyan("▲"));
    println!("                  {}", cyan("╱ ╲"));
    println!("                 {}", cyan("╱   ╲"));
    println!("                {}", cyan("╱ █ █ ╲"));
    println!("               {}", purple("╱ █ █ █ ╲"));
    println!("              {}", purple("╱ █ █ █ █ ╲"));
    println!("             {}", indigo("╱ █ █ ▲ █ █ ╲"));
    println!("            {}", indigo("╱ █ █ ╱ ╲ █ █ ╲"));
    println!("           {}", indigo("╱ █ █ ╱ █ ╲ █ █ ╲"));
    println!("          {}", purple("▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔"));
    println!("           {}", cyan("•     •     •     •"));
    println!("         {}", slate("≈≈≈≈ QUANTUM LEVITATION FIELD ≈≈≈≈"));
    println!("           {}", purple("▲     ▲     ▲     ▲"));

    println!();
    print!("  {}", purple("✦ ── "));
    print!("{}", cyan("A N T I - G R A V I T Y"));
    print!(" ");
    print!("{}", indigo("C L I"));
    println!(" {}", purple("── ✦"));

    println!(
        "  {}",
        slate("  • DEEPMIND ADVANCED AGENTIC REASONING & SYNTHESIS PLATFORM •")
    );
    println!();
}

/// ☀️ Claude Logo
/// Anthropic's iconic 8-pointed warm terracotta sunburst asterisk
pub fn render_claude_logo() {
    let terracotta = |s: &str| bold_rgb(217, 119, 87, s);
    let coral = |s: &str| bold_rgb(244, 162, 97, s);
    let amber = |s: &str| bold_rgb(231, 111, 81, s);
    let cream = |s: &str| rgb(254, 215, 170, s);

    println!();
    println!("                   {}", terracotta("█   █"));
    println!("                    {}", terracotta("▀█▄█▀"));
    println!("             {}", coral("▄█▄") + &terracotta("      █      ") + &coral("▄█▄"));
    println!("              {}", coral("▀██▄") + &terracotta("   ███   ") + &coral("▄██▀"));
    println!("       {}", amber("▀████▄▄") + &coral("  ▀███") + &terracotta("█████") + &coral("███▀  ") + &amber("▄▄████▀"));
    println!("          {}", amber("▀███████") + &terracotta("███████") + &amber("███████▀"));
    println!("       {}", amber("▄████▀▀") + &coral("  ▄███") + &terracotta("█████") + &coral("███▄  ") + &amber("▀▀████▄"));
    println!("              {}", coral("▄██▀") + &terracotta("   ███   ") + &coral("▀██▄"));
    println!("             {}", coral("▀█▀") + &terracotta("      █      ") + &coral("▀█▀"));
    println!("                    {}", terracotta("▄█▀█▄"));
    println!("                   {}", terracotta("█   █"));

    println!();
    print!("  {}", terracotta("✦ ── "));
    print!("{}", coral("C L A U D E"));
    print!(" ");
    print!("{}", amber("C L I"));
    println!(" {}", terracotta("── ✦"));

    println!(
        "  {}",
        cream("  • ANTHROPIC CONSTITUTIONAL AI • HIGH-DIMENSIONAL COGNITION •")
    );
    println!();
}

/// ⚡ Grok Logo
/// xAI cybernetic geometric slash monolithic monogram
pub fn render_grok_logo() {
    let white = |s: &str| bold_rgb(255, 255, 255, s);
    let silver = |s: &str| bold_rgb(148, 163, 184, s);
    let cyan_glow = |s: &str| bold_rgb(56, 189, 248, s);
    let dark = |s: &str| rgb(71, 85, 105, s);

    println!();
    println!("           {}", white("███████████╗") + &dark("      ") + &cyan_glow("◢███◤  ◢███◤"));
    println!("           {}", white("██╔════════╝") + &dark("     ") + &cyan_glow("◢███◤  ◢███◤"));
    println!("           {}", white("██║  ██████╗") + &dark("    ") + &cyan_glow("◢███◤  ◢███◤"));
    println!("           {}", white("██║  ╚═══██║") + &dark("   ") + &cyan_glow("◢███◤  ◢███◤"));
    println!("           {}", silver("███████████║") + &dark("  ") + &cyan_glow("◢███◤  ◢███◤"));
    println!("           {}", silver("╚══════════╝") + &dark(" ") + &cyan_glow("◢███◤  ◢███◤"));
    println!("                        {}", cyan_glow("▀▀▀▀   ▀▀▀▀"));

    println!();
    print!("  {}", silver("⚡ ── "));
    print!("{}", white("G R O K"));
    print!(" ");
    print!("{}", cyan_glow("C L I"));
    println!(" {}", silver("── ⚡"));

    println!(
        "  {}",
        dark("  • xAI MAXIMUM TRUTH-SEEKING • UNFILTERED REAL-TIME ENGINE •")
    );
    println!();
}

pub fn render_all_logos() {
    println!("================================================================================");
    println!("                     LDRAGO CLI — LOGO GALLERY SHOWCASE                         ");
    println!("================================================================================");
    render_ldrago_dragon();
    println!("--------------------------------------------------------------------------------");
    render_ag_logo();
    println!("--------------------------------------------------------------------------------");
    render_claude_logo();
    println!("--------------------------------------------------------------------------------");
    render_grok_logo();
    println!("================================================================================");
}

pub fn execute_logo(target: Option<LogoTarget>, all: bool) -> Result<()> {
    if all {
        render_all_logos();
        return Ok(());
    }

    match target.unwrap_or(LogoTarget::Ldrago) {
        LogoTarget::Ldrago => render_ldrago_dragon(),
        LogoTarget::Claude => render_claude_logo(),
        LogoTarget::Grok => render_grok_logo(),
        LogoTarget::Ag => render_ag_logo(),
        LogoTarget::All => render_all_logos(),
    }

    Ok(())
}
