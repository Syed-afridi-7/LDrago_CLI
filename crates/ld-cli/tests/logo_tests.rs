use assert_cmd::Command;
use predicates::prelude::*;

#[test]
fn test_cli_ldrago_command_displays_beyblade_dragon_logo() {
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
fn test_cli_logo_subcommand_displays_beyblade_dragon_logo() {
    let mut cmd = Command::cargo_bin("ldrago").unwrap();
    cmd.arg("logo");
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("L - D R A G O"))
        .stdout(predicate::str::contains("B E Y B L A D E"))
        .stdout(predicate::str::contains("DRAGON EMPEROR"))
        .stdout(predicate::str::contains("LEFT-SPIN"));
}

#[test]
fn test_cli_version_includes_ldrago_dragon_banner() {
    let mut cmd = Command::cargo_bin("ldrago").unwrap();
    cmd.arg("version");
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("L - D R A G O"))
        .stdout(predicate::str::contains("DRAGON EMPEROR"))
        .stdout(predicate::str::contains("LDrago CLI — Local Twin AI Platform"));
}
