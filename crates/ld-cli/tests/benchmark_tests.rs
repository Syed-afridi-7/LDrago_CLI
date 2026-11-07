use assert_cmd::Command;
use predicates::prelude::*;

#[test]
fn test_cli_benchmark_help() {
    let mut cmd = Command::cargo_bin("ldrago").unwrap();
    cmd.arg("benchmark").arg("--help");
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Benchmark inference throughput"))
        .stdout(predicate::str::contains("--brain"))
        .stdout(predicate::str::contains("--tokens"))
        .stdout(predicate::str::contains("--quantized"));
}

#[test]
fn test_cli_benchmark_execution() {
    let mut cmd = Command::cargo_bin("ldrago").unwrap();
    cmd.arg("benchmark")
        .arg("--brain")
        .arg("coding")
        .arg("--tokens")
        .arg("10");
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("INFERENCE BENCHMARK PROFILER"))
        .stdout(predicate::str::contains("Target Brain"))
        .stdout(predicate::str::contains("Total Latency"))
        .stdout(predicate::str::contains("Throughput"));
}
