pub mod commands;
pub mod config;

use clap::{Parser, Subcommand};

#[derive(Parser, Debug, Clone, PartialEq)]
#[command(
    name = "ldrago",
    author = "LDrago Team",
    version = env!("CARGO_PKG_VERSION"),
    about = "LDrago CLI — Multi-engine local AI orchestrator",
    long_about = "LDrago is a modular, production-ready AI CLI featuring two custom neural architectures built from scratch in PyTorch (Transformer & SSM) alongside local Ollama integration."
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<Commands>,
}

#[derive(Subcommand, Debug, Clone, PartialEq)]
pub enum Commands {
    /// Run diagnostic probes across all three AI engines
    Doctor,
    /// Inspect and manage registered model engines
    Model {
        #[command(subcommand)]
        action: ModelCommands,
    },
    /// Inspect workspace context, directory tree, manifest, or keyword search
    Context {
        #[command(subcommand)]
        action: commands::context::ContextCommands,
    },
    /// Generate a grounded execution plan validated against JSON schema and security policies
    Plan {
        /// High-level objective to accomplish
        objective: String,
        /// Target brain: coding (Transformer), design (SSM), or general
        #[arg(short, long, default_value = "coding")]
        brain: String,
    },
    /// Display real-time dynamic development roadmap and completion dashboard
    Progress,
    /// Benchmark inference throughput, latency, and tokens/sec for an LDrago engine
    Benchmark {
        /// Target brain: coding (Transformer), design (SSM), or general (Ollama)
        #[arg(short, long, default_value = "coding")]
        brain: String,
        /// Number of tokens to generate
        #[arg(short, long, default_value_t = 32)]
        tokens: u32,
        /// Benchmark with INT8 quantization mode
        #[arg(long)]
        quantized: bool,
    },
    /// Display the L-Drago Beyblade Dragon logo (Golden, Red & Blue)
    Logo,
    /// Display the L-Drago Beyblade Dragon logo (alias for logo)
    Ldrago,
    /// Execute a command safely in a sandboxed, environment-scrubbed subprocess
    Run {
        /// The command line string to execute
        command: String,
        /// Automatically approve execution without confirmation prompt
        #[arg(short, long)]
        yes: bool,
        /// Timeout limit in seconds
        #[arg(short, long, default_value_t = 30)]
        timeout: u64,
    },
    /// Inspect and atomically apply a patch with dry-run and backup safety
    Patch {
        /// Target file path to patch
        file: String,
        /// Replacement content
        #[arg(short, long)]
        content: String,
        /// Preview diff without writing any changes to disk
        #[arg(short, long)]
        dry_run: bool,
        /// Automatically approve patch without confirmation prompt
        #[arg(short, long)]
        yes: bool,
    },
    /// Roll back a file to its automatic backup (.bak) created during patching
    Rollback {
        /// Target file path to roll back
        file: String,
    },
    /// Create, list, or restore workspace snapshot checkpoints
    Checkpoint {
        #[command(subcommand)]
        action: commands::checkpoint::CheckpointAction,
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
    /// Train an LDrago brain model using local PyTorch training harness
    Train {
        /// Target brain: coding (Transformer) or design (SSM)
        brain: String,
        /// Run micro-training on a tiny batch for verification
        #[arg(long)]
        micro: bool,
        /// Number of training steps
        #[arg(short, long, default_value_t = 10)]
        steps: u32,
    },
    /// Print LDrago version and system architecture information
    Version,
}

#[derive(Subcommand, Debug, Clone, PartialEq)]
pub enum ModelCommands {
    /// List all registered model engines and their status
    List,
}
