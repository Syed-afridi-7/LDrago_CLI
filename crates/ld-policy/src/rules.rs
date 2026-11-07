use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum RiskLevel {
    ReadOnly = 0,
    Low = 1,
    Medium = 2,
    High = 3,
    Dangerous = 4,
}

impl std::fmt::Display for RiskLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RiskLevel::ReadOnly => write!(f, "ReadOnly"),
            RiskLevel::Low => write!(f, "Low"),
            RiskLevel::Medium => write!(f, "Medium"),
            RiskLevel::High => write!(f, "High"),
            RiskLevel::Dangerous => write!(f, "Dangerous"),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ToolPermission {
    Allow(RiskLevel),
    RequiresApproval(RiskLevel),
    Deny(RiskLevel),
}

impl std::fmt::Display for ToolPermission {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ToolPermission::Allow(r) => write!(f, "Allow ({})", r),
            ToolPermission::RequiresApproval(r) => write!(f, "Approval Required ({})", r),
            ToolPermission::Deny(r) => write!(f, "Denied ({})", r),
        }
    }
}

#[derive(Debug, Clone)]
pub struct PolicyEngine {
    brain: String,
}

impl PolicyEngine {
    pub fn for_brain(brain: &str) -> Self {
        Self {
            brain: brain.to_string(),
        }
    }

    pub fn evaluate_tool(&self, tool_name: &str) -> ToolPermission {
        match tool_name {
            "read_file" | "list_dir" | "search_code" | "get_scene_info" => {
                ToolPermission::Allow(RiskLevel::ReadOnly)
            }
            "write_file" | "edit_file" | "create_file" => {
                ToolPermission::Allow(RiskLevel::Medium)
            }
            "run_command" | "execute_bash" | "git_commit" | "git_push" => {
                ToolPermission::RequiresApproval(RiskLevel::High)
            }
            "delete_database"
            | "drop_tables"
            | "format_disk"
            | "raw_shell_root"
            | "bypass_auth" => ToolPermission::Deny(RiskLevel::Dangerous),
            _ => ToolPermission::RequiresApproval(RiskLevel::Medium),
        }
    }

    pub fn brain_name(&self) -> &str {
        &self.brain
    }
}
