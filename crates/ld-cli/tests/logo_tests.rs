use assert_cmd::Command;
use predicates::prelude::*;
use ld_cli::commands::logo::{render_banner, strip_ansi};

#[test]
fn test_render_banner_format_and_cleanliness() {
    let banner = render_banner("Coding");
    // Assert single line technical banner format
    assert!(banner.contains(">_ LDrago"));
    assert!(banner.contains("Local"));
    assert!(banner.contains("Autonomous"));
    assert!(banner.contains("[Coding]"));

    let plain = strip_ansi(&banner);
    assert_eq!(plain, ">_ LDrago • Local • Autonomous • [Coding]");
    
    // Assert zero overhyped dragon lore
    assert!(!banner.contains("Crimson Fangs"));
    assert!(!banner.contains("Electric Azure Aura"));
    assert!(!banner.contains("Azure Aura"));
    assert!(!banner.contains("BEYBLADE"));
    assert!(!banner.contains("DRAGON EMPEROR"));
    assert!(!banner.contains("VORTEX"));
}

#[test]
fn test_cli_ldrago_command_displays_clean_banner() {
    let mut cmd = Command::cargo_bin("ldrago").unwrap();
    cmd.arg("ldrago");
    cmd.assert()
        .success()
        .stdout(predicate::str::contains(">_ LDrago"))
        .stdout(predicate::str::contains("Autonomous"))
        .stdout(predicate::str::contains("[Coding]"))
        .stdout(predicate::str::contains("Crimson Fangs").not())
        .stdout(predicate::str::contains("Azure Aura").not())
        .stdout(predicate::str::contains("BEYBLADE").not())
        .stdout(predicate::str::contains("DRAGON EMPEROR").not());
}

#[test]
fn test_cli_logo_subcommand_displays_clean_banner() {
    let mut cmd = Command::cargo_bin("ldrago").unwrap();
    cmd.arg("logo");
    cmd.assert()
        .success()
        .stdout(predicate::str::contains(">_ LDrago"))
        .stdout(predicate::str::contains("Autonomous"))
        .stdout(predicate::str::contains("[Coding]"))
        .stdout(predicate::str::contains("Crimson Fangs").not())
        .stdout(predicate::str::contains("Azure Aura").not())
        .stdout(predicate::str::contains("BEYBLADE").not());
}

#[test]
fn test_cli_version_displays_clean_banner() {
    let mut cmd = Command::cargo_bin("ldrago").unwrap();
    cmd.arg("version");
    cmd.assert()
        .success()
        .stdout(predicate::str::contains(">_ LDrago"))
        .stdout(predicate::str::contains("LDrago CLI — Local Twin AI Platform"))
        .stdout(predicate::str::contains("Crimson Fangs").not())
        .stdout(predicate::str::contains("Azure Aura").not())
        .stdout(predicate::str::contains("BEYBLADE").not())
        .stdout(predicate::str::contains("DRAGON EMPEROR").not());
}
