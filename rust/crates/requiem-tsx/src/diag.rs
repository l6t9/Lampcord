use std::fmt;

#[derive(Clone, Debug, PartialEq)]
pub struct Diagnostic {
    pub start: u32,
    pub end: u32,
    pub line: u32,
    pub column: u32,
    pub message: String,
    pub help: Option<String>,
}

impl Diagnostic {
    pub fn new(source: &str, start: u32, end: u32, message: impl Into<String>) -> Self {
        let before = &source[..(start as usize).min(source.len())];
        let line = before.matches('\n').count() as u32 + 1;
        let column = before.rsplit('\n').next().map_or(0, |l| l.chars().count()) as u32 + 1;
        Self {
            start,
            end,
            line,
            column,
            message: message.into(),
            help: None,
        }
    }

    pub fn help(mut self, help: impl Into<String>) -> Self {
        self.help = Some(help.into());
        self
    }

    pub fn render(&self, file: &str, source: &str) -> String {
        let text = source.lines().nth(self.line as usize - 1).unwrap_or("");
        let room = text.chars().count().saturating_sub(self.column as usize - 1).max(1);
        let width = (self.end.saturating_sub(self.start) as usize).clamp(1, room);
        let mut out = format!(
            "{file}:{}:{}: {}\n  {text}\n  {}{}",
            self.line,
            self.column,
            self.message,
            " ".repeat(self.column as usize - 1),
            "^".repeat(width),
        );
        if let Some(help) = &self.help {
            out.push_str("\n  help: ");
            out.push_str(help);
        }
        out
    }
}

impl fmt::Display for Diagnostic {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}:{}: {}", self.line, self.column, self.message)
    }
}
