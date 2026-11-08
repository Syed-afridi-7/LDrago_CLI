//! REPL session banner rendering matching Claude Code terminal styling.

use crate::commands::logo::render_banner;
use ld_schemas::BrainType;

/// Render the formatted banner string for a given brain type.
pub fn session_banner_text(brain: BrainType) -> String {
    let label = match brain {
        BrainType::Coding => "Coding",
        BrainType::Design => "Design",
        BrainType::General => "General",
    };
    let banner_line = render_banner(label);
    let border = "\x1b[38;2;75;85;99m"; // slate gray
    let reset = "\x1b[0m";
    let cyan = "\x1b[38;2;0;220;255m";
    let green = "\x1b[38;2;34;197;94m";
    let dim = "\x1b[38;2;156;163;175m";
    let bold_white = "\x1b[1;37m";

    format!(
        "{border}╭────────────────────────────────────────────────────────────────────────────╮{reset}\n\
         {border}│{reset}  {cyan}●{reset} {bold_white}LDrago CLI{reset} {dim}(v{}){reset}\n\
         {border}│{reset}  {dim}Local Twin AI • Autonomous Code & Design Engine{reset}\n\
         {border}│{reset}\n\
         {border}│{reset}  {banner_line}\n\
         {border}│{reset}\n\
         {border}│{reset}  {dim}Active Engine:{reset} {green}● {label}{reset}   {dim}Workspace:{reset} {cyan}{}{reset}\n\
         {border}│{reset}  {dim}Commands:{reset}      {cyan}/model{reset} {dim}(switch){reset}  {cyan}/status{reset}  {cyan}/help{reset}  {cyan}/clear{reset}  {cyan}/exit{reset}\n\
         {border}╰────────────────────────────────────────────────────────────────────────────╯{reset}",
        env!("CARGO_PKG_VERSION"),
        std::env::current_dir().unwrap_or_default().display()
    )
}

/// Print the clean technical banner for a REPL session.
pub fn print_session_banner(brain: BrainType) {
    println!("\n{}\n", session_banner_text(brain));
}
