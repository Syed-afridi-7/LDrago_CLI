use assert_cmd::Command;
use predicates::prelude::*;

#[test]
fn test_cli_ldrago_beyblade_dragon_logo() {
    let mut cmd = Command::cargo_bin("ldrago").unwrap();
    cmd.arg("ldrago");
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("L - D R A G O"))
        .stdout(predicate::str::contains("B E Y B L A D E"))
        .stdout(predicate::str::contains("DRAGON EMPEROR"))
        .stdout(predicate::str::contains("LEFT-SPIN"))
        .stdout(predicate::str::contains("Golden Horns"))
        .stdout(predicate::str::contains("Crimson Fangs"))
        .stdout(predicate::str::contains("Electric Azure Aura"));
}

#[test]
fn test_cli_grok_logo() {
    let mut cmd = Command::cargo_bin("ldrago").unwrap();
    cmd.arg("grok");
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("G R O K"))
        .stdout(predicate::str::contains("xAI"));
}

#[test]
fn test_cli_claude_logo() {
    let mut cmd = Command::cargo_bin("ldrago").unwrap();
    cmd.arg("claude");
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("C L A U D E"))
        .stdout(predicate::str::contains("ANTHROPIC"));
}

#[test]
fn test_cli_ag_anti_gravity_logo() {
    let mut cmd = Command::cargo_bin("ldrago").unwrap();
    cmd.arg("ag");
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("A N T I - G R A V I T Y"))
        .stdout(predicate::str::contains("QUANTUM LEVITATION"));
}

#[test]
fn test_cli_logo_all_gallery_showcase() {
    let mut cmd = Command::cargo_bin("ldrago").unwrap();
    cmd.arg("logo").arg("--all");
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("LOGO GALLERY SHOWCASE"))
        .stdout(predicate::str::contains("L - D R A G O"))
        .stdout(predicate::str::contains("A N T I - G R A V I T Y"))
        .stdout(predicate::str::contains("C L A U D E"))
        .stdout(predicate::str::contains("G R O K"));
}
