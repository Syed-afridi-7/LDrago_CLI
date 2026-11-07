//! REPL command parsing and command definitions.

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ReplCommand {
    Model(Option<String>),
    Status,
    Clear,
    Help,
    Exit,
    Prompt(String),
}

/// Parse user REPL input into a typed command.
pub fn parse_repl_command(input: &str) -> ReplCommand {
    let trimmed = input.trim();
    if trimmed.is_empty() {
        return ReplCommand::Prompt(String::new());
    }

    if trimmed == "exit" || trimmed == "quit" || trimmed == "/exit" || trimmed == "/quit" {
        return ReplCommand::Exit;
    }
    if trimmed == "/status" {
        return ReplCommand::Status;
    }
    if trimmed == "/clear" {
        return ReplCommand::Clear;
    }
    if trimmed == "/help" {
        return ReplCommand::Help;
    }
    if trimmed == "/model" {
        return ReplCommand::Model(None);
    }
    if let Some(rest) = trimmed.strip_prefix("/model") {
        if rest.starts_with(char::is_whitespace) {
            let model_arg = rest.trim();
            if model_arg.is_empty() {
                return ReplCommand::Model(None);
            } else {
                return ReplCommand::Model(Some(model_arg.to_string()));
            }
        }
    }

    ReplCommand::Prompt(trimmed.to_string())
}
