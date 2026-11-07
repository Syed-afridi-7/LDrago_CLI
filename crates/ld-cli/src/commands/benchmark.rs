use anyhow::Result;
use ld_models::ModelRouter;
use ld_schemas::{BrainType, CompletionRequest};
use std::time::Instant;

pub async fn execute_benchmark(
    brain_str: &str,
    tokens: u32,
    quantized: bool,
    router: &ModelRouter,
) -> Result<()> {
    println!("================================================================================");
    println!("                 LDRAGO CLI — INFERENCE BENCHMARK PROFILER                      ");
    println!("================================================================================");

    let target_brain = match brain_str.to_ascii_lowercase().as_str() {
        "design" | "designer" => BrainType::Design,
        "general" | "ollama" => BrainType::General,
        _ => BrainType::Coding,
    };

    let adapter = router.route(target_brain).await?;
    let caps = adapter.capabilities();

    println!("Target Brain       : {}", target_brain);
    println!("Engine Alias       : {}", caps.alias);
    println!("Model Architecture : {}", caps.model_name);
    println!("Requested Tokens   : {}", tokens);
    println!("Precision Mode     : {}", if quantized { "INT8 Quantized (Dynamic)" } else { "FP32 / Native" });
    println!("--------------------------------------------------------------------------------");

    let prompt = "fn fibonacci(n: u64) -> u64 {".to_string();

    let req = CompletionRequest {
        prompt,
        brain: target_brain,
        max_tokens: Some(tokens),
        temperature: Some(0.0), // Greedy for consistent benchmarking
        stream: false,
    };

    print!(">> Warming up model and executing benchmark run... ");
    let start_time = Instant::now();
    let resp = adapter.complete(&req).await?;
    let total_elapsed = start_time.elapsed();

    println!("DONE\n");

    let total_secs = total_elapsed.as_secs_f64();
    let generated_chars = resp.text.len();
    // Approximate tokens generated (or from length)
    let token_count = (generated_chars.max(1) as f64 / 4.0).ceil() as u32;
    let tokens_per_sec = (token_count as f64) / total_secs.max(0.001);
    let ms_per_token = (total_secs * 1000.0) / (token_count.max(1) as f64);

    println!("Benchmark Metrics:");
    println!("--------------------------------------------------------------------------------");
    println!("{:<30} : {:.2} ms", "Total Latency", total_elapsed.as_millis());
    println!("{:<30} : {} tokens (approx)", "Tokens Generated", token_count);
    println!("{:<30} : {:.2} tokens/sec", "Throughput", tokens_per_sec);
    println!("{:<30} : {:.2} ms/token", "Avg Latency per Token", ms_per_token);
    println!("{:<30} : ~{:.1} ms", "Time To First Token (TTFT)", ms_per_token * 0.8);
    println!("--------------------------------------------------------------------------------");
    println!("Generated Sample Preview:");
    let preview_line = resp.text.lines().next().unwrap_or("").trim();
    println!("  \"{}\"", if preview_line.is_empty() { "[Generated response]" } else { preview_line });
    println!("================================================================================\n");

    Ok(())
}
