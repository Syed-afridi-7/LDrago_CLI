use clap::Parser;
use ld_cli::config::AppConfig;
use ld_cli::{commands, Cli, Commands, ModelCommands};
use ld_context::CitationExtractor;
use ld_models::{CustomBrainAdapter, ModelAdapter, ModelRouter, OllamaAdapter};
use ld_policy::{SafetyGate, SafetyVerdict};
use ld_schemas::{BrainType, CompletionRequest, HealthStatus};
use std::env;
use std::sync::Arc;
use std::time::Instant;

fn build_router(cfg: &AppConfig) -> ModelRouter {
    let mut router = ModelRouter::new();

    let coder = Arc::new(CustomBrainAdapter::new(
        &cfg.coding_endpoint,
        BrainType::Coding,
    ));
    let designer = Arc::new(CustomBrainAdapter::new(
        &cfg.design_endpoint,
        BrainType::Design,
    ));
    let ollama = Arc::new(OllamaAdapter::new(
        &cfg.ollama_endpoint,
        &cfg.ollama_model,
    ));

    router.register(coder);
    router.register(designer);
    router.register(ollama);

    router
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    let config = AppConfig::load(None).unwrap_or_default();
    let router = build_router(&config);

    match cli.command {
        Some(Commands::Version) | None => {
            commands::logo::render_ldrago_dragon();
            println!("========================================================");
            println!("  LDrago CLI — Local Twin AI Platform (v{})", env!("CARGO_PKG_VERSION"));
            println!("========================================================");
            println!("Architecture:");
            println!("  • Custom Architecture #1 (Coding): Pure PyTorch Transformer (GQA + RoPE + SwiGLU)");
            println!("  • Custom Architecture #2 (Design): Pure PyTorch State-Space Model (Mamba Selective Scan)");
            println!("  • Local Engine #3 (General): Ollama local runtime adapter");
            println!("  • Context Engine: High-throughput ignore scanner, symbol indexer & citation engine");
            println!("  • Policy Engine: Zero-trust safety gate, risk classifier & JSONL audit trail");
            println!("  • Core CLI: Rust (tokio + clap + serde + figment)");
        }

        Some(Commands::Logo) | Some(Commands::Ldrago) => {
            commands::logo::execute_logo()?;
        }

        Some(Commands::Doctor) => {
            println!("LDrago System Diagnostic Report");
            println!("--------------------------------------------------------");
            let coder = CustomBrainAdapter::new(&config.coding_endpoint, BrainType::Coding);
            let designer = CustomBrainAdapter::new(&config.design_endpoint, BrainType::Design);
            let ollama = OllamaAdapter::new(&config.ollama_endpoint, &config.ollama_model);

            print!("  [1] Coding Brain (Transformer @ {}) ... ", config.coding_endpoint);
            match coder.health_check().await {
                Ok(HealthStatus::Healthy) => println!("ONLINE (Healthy)"),
                Ok(HealthStatus::Degraded) => println!("DEGRADED"),
                _ => println!("OFFLINE (Run 'uv run uvicorn src.serve.server:app --port 8741' in training/)"),
            }

            print!("  [2] Design Brain (SSM/Mamba @ {}) ... ", config.design_endpoint);
            match designer.health_check().await {
                Ok(HealthStatus::Healthy) => println!("ONLINE (Healthy)"),
                Ok(HealthStatus::Degraded) => println!("DEGRADED"),
                _ => println!("OFFLINE (Run 'uv run uvicorn src.serve.server:app --port 8741' in training/)"),
            }

            print!("  [3] General Runtime (Ollama @ {}) ... ", config.ollama_endpoint);
            match ollama.health_check().await {
                Ok(HealthStatus::Healthy) => println!("ONLINE (Healthy)"),
                Ok(HealthStatus::Degraded) => println!("DEGRADED"),
                _ => println!("OFFLINE (Ollama service not detected at {})", config.ollama_endpoint),
            }
            println!("--------------------------------------------------------");
        }

        Some(Commands::Model { action }) => match action {
            ModelCommands::List => {
                println!("Registered LDrago Engines:");
                println!("--------------------------------------------------------------------------------");
                println!("{:<16} {:<32} {:<10} {:<15}", "ALIAS", "MODEL NAME", "BRAIN", "ENDPOINT");
                println!("--------------------------------------------------------------------------------");
                for cap in router.list_capabilities() {
                    let brain_str = cap.brain.map(|b| b.to_string()).unwrap_or_else(|| "general".to_string());
                    println!("{:<16} {:<32} {:<10} {:<15}", cap.alias, cap.model_name, brain_str, cap.endpoint);
                }
                println!("--------------------------------------------------------------------------------");
            }
        },

        Some(Commands::Context { action }) => {
            commands::context::execute_context(action)?;
        }

        Some(Commands::Plan { objective, brain }) => {
            commands::plan::execute_plan(&objective, &brain, &router).await?;
        }

        Some(Commands::Progress) => {
            commands::progress::execute_progress(&config.coding_endpoint, &config.design_endpoint).await?;
        }

        Some(Commands::Benchmark { brain, tokens, quantized }) => {
            commands::benchmark::execute_benchmark(&brain, tokens, quantized, &router).await?;
        }

        Some(Commands::Ask {
            prompt,
            brain,
            max_tokens,
            temperature,
        }) => {
            let safety = SafetyGate::default();
            if let SafetyVerdict::Violation { reason, risk } = safety.check_prompt(&prompt) {
                eprintln!("\n[SECURITY BLOCK] Safety Gate Intercepted Prompt:");
                eprintln!("  Reason: {}", reason);
                eprintln!("  Risk Level: {}", risk);
                anyhow::bail!("Prompt rejected by security policy.");
            }

            let target_brain = match brain.to_ascii_lowercase().as_str() {
                "coding" | "coder" => BrainType::Coding,
                "design" | "designer" => BrainType::Design,
                "general" | "ollama" => BrainType::General,
                _ => {
                    let lower = prompt.to_ascii_lowercase();
                    if lower.contains("design") || lower.contains("flow") || lower.contains("ui") || lower.contains("ux") {
                        BrainType::Design
                    } else {
                        BrainType::Coding
                    }
                }
            };

            println!(">> Routing instruction to [{target_brain}] brain...");
            let start = Instant::now();
            let adapter = router.route(target_brain).await?;
            let caps = adapter.capabilities();
            println!(">> Selected Engine: {} ({})", caps.alias, caps.model_name);

            let req = CompletionRequest {
                prompt: prompt.clone(),
                brain: target_brain,
                max_tokens: Some(max_tokens),
                temperature: Some(temperature),
                stream: false,
            };

            match adapter.complete(&req).await {
                Ok(resp) => {
                    let elapsed = start.elapsed();
                    println!("\nResponse ({} ms):", elapsed.as_millis());
                    println!("--------------------------------------------------------");
                    println!("{}", resp.text);
                    println!("--------------------------------------------------------");

                    // Check for file citations in response
                    if let Ok(cwd) = env::current_dir() {
                        let extractor = CitationExtractor::new(&cwd);
                        let citations = extractor.extract_and_verify(&resp.text);
                        if !citations.is_empty() {
                            println!("\nVerified Source Citations ({}):", citations.len());
                            for c in citations {
                                let status = if c.verified { "VERIFIED" } else { "UNVERIFIED" };
                                println!("  • [{}] {} (L{}-L{})", status, c.relative_path, c.start_line, c.end_line);
                            }
                        }
                    }
                }
                Err(err) => {
                    eprintln!("\nError from [{target_brain}] engine: {err}");
                    eprintln!("Tip: Verify the local backend server is running with 'ldrago doctor'.");
                }
            }
        }

        Some(Commands::Train { brain, micro, steps }) => {
            commands::train::execute_train(&brain, micro, steps)?;
        }
    }

    Ok(())
}
