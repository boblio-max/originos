#[derive(Debug, PartialEq)]
pub enum Command {
    Open(String), Close(String), Restart(String),
    Echo(String), Help, Clear, Exit, Empty,
    Host(String),
}

pub fn parse(line: &str) -> Command {
    if line.trim().is_empty() {
        return Command::Empty;
    }
    else {
        let parts: Vec<&str> = line.trim().split_whitespace().collect();
        match parts[0].to_ascii_lowercase().as_str() {
            "open" => Command::Open(parts[1..].join(" ")),
            "close" => Command::Close(parts[1..].join(" ")),
            "restart" => Command::Restart(parts[1..].join(" ")),
            "echo" => Command::Echo(parts[1..].join(" ")),
            "help" => Command::Help,
            "clear" => Command::Clear,
            "exit" => Command::Exit,
            "host" => Command::Host(line.trim().to_string()),
        }
    }
}

#[cfg(test)]
mod tests {

}