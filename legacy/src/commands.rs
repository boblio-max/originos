use std::process::Command;

pub struct OriginShell;

impl OriginShell {
    pub fn new() -> Self {
        Self
    }

    pub fn command_loader(&self, input: &str) -> Vec<String> {
        let command = input.trim();
        let mut output: Vec<String> = Vec::new();

        if command.starts_with("open ") {
            let target = command[5..].trim();

            let _ = Command::new("xdg-open")
                .arg(target)
                .spawn();
        }

        else if command.starts_with("close ") {
            let process = command[6..].trim();

            let _ = Command::new("pkill")
                .arg(process)
                .spawn();
        }

        else if command.starts_with("restart ") {
            let process = command[8..].trim();

            let _ = Command::new("pkill")
                .arg(process)
                .spawn();

            let _ = Command::new(process)
                .spawn();
        }

        else if command == "help" {
            output.push("OriginOS Commands:".to_string());
            output.push("help".to_string());
            output.push("clear".to_string());
            output.push("exit".to_string());
        }

        output
    }
}