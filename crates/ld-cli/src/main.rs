use clap::{Parser, Subcommand};
use ld_models::{CustomBrainAdapter, ModelAdapter, ModelRouter, OllamaAdapter};
use ld_schemas::{BrainType, CompletionRequest, HealthStatus};
use std::sync::Arc;
use std::time::Instant;

mod config;
use config::AppConfig;

#[derive(Parser, Debug)]
#[command(
    name = "ldrago",
    author = "LDrago Team",
    version = env!("CARGO_PKG_VERSION"),
    about = "LDrago CLI — Multi-engine local AI orchestrator",
    long_about = "LDrago is a modular, production-ready AI CLI featuring two custom neural architectures built from scratch in PyTorch (Transformer & SSM) alongside local Ollama integration."
)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Run diagnostic probes across all three AI engines
    Doctor,
    /// Inspect and manage registered model engines
    Model {
        #[command(subcommand)]
        action: ModelCommands,
    },
    /// Send an instruction or prompt to a brain
    Ask {
        /// Text instruction or prompt to evaluate
        prompt: String,
        /// Target brain: coding (Transformer), design (SSM), general (Ollama), or auto
        #[arg(short, long, default_value = "auto")]
        brain: String,
        /// Maximum number of tokens to generate
        #[arg(short, long, default_value_t = 64)]
        max_tokens: u32,
        /// Sampling temperature (0.0 to 1.0)
        #[arg(short, long, default_value_t = 0.7)]
        temperature: f32,
    },
    /// Print LDrago version and system architecture information
    Version,
}

#[derive(Subcommand, Debug)]
enum ModelCommands {
    /// List all registered model engines and their status
    List,
}

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
            println!("========================================================");
            println!("  LDrago CLI — Local Twin AI Platform (v{})", env!("CARGO_PKG_VERSION"));
            println!("========================================================");
            println!("Architecture:");
            println!("  • Custom Architecture #1 (Coding): Pure PyTorch Transformer (GQA + RoPE + SwiGLU)");
            println!("  • Custom Architecture #2 (Design): Pure PyTorch State-Space Model (Mamba Selective Scan)");
            println!("  • Local Engine #3 (General): Ollama local runtime adapter");
            println!("  • Core CLI: Rust (tokio + clap + serde + figment)");
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

        Some(Commands::Ask {
            prompt,
            brain,
            max_tokens,
            temperature,
        }) => {
            let target_brain = match brain.to_ascii_lowercase().as_str() {
                "coding" | "coder" => BrainType::Coding,
                "design" | "designer" => BrainType::Design,
                "general" | "ollama" => BrainType::General,
                _ => {
                    // Auto-route heuristic: if prompt mentions UI, flow, or design -> Design Brain, else Coding Brain
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
                }
                Err(err) => {
                    eprintln!("\nError from [{target_brain}] engine: {err}");
                    eprintln!("Tip: Verify the local backend server is running with 'ldrago doctor'.");
                }
            }
        }
    }

    Ok(())
}
