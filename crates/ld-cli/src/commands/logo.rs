use anyhow::Result;

// TrueColor RGB styling helper
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

pub fn execute_logo() -> Result<()> {
    render_ldrago_dragon();
    Ok(())
}
