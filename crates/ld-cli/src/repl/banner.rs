//! REPL session banner rendering.

use crate::commands::logo::render_banner;
use ld_schemas::BrainType;

/// Render the formatted banner string for a given brain type.
pub fn session_banner_text(brain: BrainType) -> String {
    let label = match brain {
        BrainType::Coding => "Coding",
        BrainType::Design => "Design",
        BrainType::General => "General",
    };
    render_banner(label)
}

/// Print the clean technical banner for a REPL session.
pub fn print_session_banner(brain: BrainType) {
    println!("\n{}\n", session_banner_text(brain));
}
