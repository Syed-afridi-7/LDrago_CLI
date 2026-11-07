# CLI Logo Suite Implementation Plan (L-Drago Dragon, Anti-Gravity, Claude, Grok)

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Implement high-fidelity, vibrant ANSI TrueColor terminal logos for **LDrago** (Beyblade-inspired 3-color dragon), **Anti-Gravity (AG)**, **Claude**, and **Grok**, accessible via dedicated subcommands (`ldrago logo <target>`, `ldrago ldrago`, `ldrago ag`, `ldrago claude`, `ldrago grok`) and integrated into the default CLI banner.

**Architecture:** A dedicated rendering engine in `crates/ld-cli/src/commands/logo.rs` supporting 24-bit TrueColor RGB terminal escape sequences with automatic ANSI fallback; integrated into `clap` CLI definitions in `crates/ld-cli/src/lib.rs` and `crates/ld-cli/src/main.rs`.

**Tech Stack:** Rust, ANSI 24-bit TrueColor, Unicode Geometric contour blocks, Clap v4.

---

## 1. Visual Design Specifications

### 🐉 L-Drago (Beyblade Dragon Logo)
- **Visual Motif:** Fierce 3-headed / horned dragon inspired by Ryuga's legendary *Lightning / Meteo L-Drago* from Beyblade Metal Masters.
- **Color Palette (3-Color Fusion):**
  - **Dragon Gold / Amber** (`#FFD700` / `#F59E0B`): Dragon horns, crown crest, and radiant aura.
  - **Ferocious Crimson Red** (`#FF2233` / `#DC2626`): Draconic jaws, fire breathing flame jets, and core body scales.
  - **Electric Cyan / Azure Blue** (`#00F0FF` / `#2563EB`): Glowing dragon eyes, left-spin energy vortex, and celestial aura.
- **Typography:** `L - D R A G O` with flame gradient shading and Beyblade Dragon Emperor tagline.

### 🌌 Anti-Gravity (AG Logo)
- **Visual Motif:** Levitating inverted quantum delta crystal floating in an anti-gravity field with upward energy rings.
- **Color Palette:**
  - Neon Cyan (`#00FFFF`), Celestial Purple (`#A855F7`), Deep Nebula Indigo (`#6366F1`).
- **Typography:** `A N T I - G R A V I T Y  C L I` with Google DeepMind aesthetic.

### ☀️ Claude Logo
- **Visual Motif:** Anthropic's iconic 8-pointed terracotta starburst / sunburst asterisk symbol.
- **Color Palette:**
  - Warm Terracotta / Coral (`#D97757`), Sunset Peach (`#F4A261`), Soft Amber (`#E76F51`).
- **Typography:** `C L A U D E  C L I` with Anthropic warm minimalist design.

### ⚡ Grok Logo
- **Visual Motif:** xAI's cybernetic geometric monolithic slash glyph ("//") with angular cyber lines.
- **Color Palette:**
  - Pure Stark White (`#FFFFFF`), Slate Silver (`#94A3B8`), Neon Electric Cyan Flare (`#38BDF8`).
- **Typography:** `G R O K  C L I` with xAI bold cybernetic styling.

---

## 2. Command Architecture

Users will be able to display logos in multiple intuitive ways:

1. **Direct subcommands:**
   - `ldrago grok` -> Displays Grok logo
   - `ldrago claude` -> Displays Claude logo
   - `ldrago ag` (or `ldrago antigravity`) -> Displays Anti-Gravity logo
   - `ldrago ldrago` (or `ldrago`) -> Displays L-Drago Beyblade Dragon logo

2. **Unified Logo command:**
   - `ldrago logo grok`
   - `ldrago logo claude`
   - `ldrago logo ag`
   - `ldrago logo ldrago`
   - `ldrago logo --all` (Galleries all 4 logos consecutively)

3. **Default Banner:**
   - Running `ldrago` or `ldrago version` showcases the L-Drago Dragon logo prominently at the top of the version information.

---

## 3. Implementation Tasks

### Task 1: Create Logo Rendering Engine (`crates/ld-cli/src/commands/logo.rs`)
- Implement color helpers: `rgb(r, g, b, text)` and `bold_rgb(r, g, b, text)`.
- Implement `render_ldrago_dragon()`: High-impact multi-line dragon ASCII/Unicode art fusing gold, crimson, and electric blue.
- Implement `render_ag_logo()`: Futuristic floating delta crystal with cyan/purple gradient.
- Implement `render_claude_logo()`: 8-pointed starburst in warm terracotta.
- Implement `render_grok_logo()`: Cyber slash monogram in stark silver and white.
- Implement `render_all_logos()`: Sequential showcase.

### Task 2: Update CLI Definitions (`crates/ld-cli/src/lib.rs`)
- Add `Logo` subcommand with `LogoTarget` enum (`Ldrago`, `Claude`, `Grok`, `Ag`, `All`).
- Add top-level alias commands: `Grok`, `Claude`, `Ag`, `Antigravity`, `Ldrago`.

### Task 3: Wire Execution in `crates/ld-cli/src/main.rs`
- Connect CLI matching to dispatch to corresponding logo rendering functions.
- Update default banner (`None` or `Version`) to display the L-Drago Dragon.

### Task 4: Unit & Integration Tests (`crates/ld-cli/tests/logo_tests.rs`)
- Test `ldrago grok` outputs Grok logo text and escape sequences.
- Test `ldrago claude` outputs Claude logo text.
- Test `ldrago ag` outputs Anti-Gravity logo text.
- Test `ldrago ldrago` outputs dragon art with gold/red/blue color codes.
- Test `ldrago logo --all` runs successfully.

### Task 5: Verification & Walkthrough
- Run `cargo test -p ld-cli --test logo_tests`.
- Verify visual output in terminal across all 4 commands.
- Update walkthrough documentation.
