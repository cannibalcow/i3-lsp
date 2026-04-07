use std::fmt::Display;

#[derive(Debug, PartialEq, Clone)]
pub struct I3Configuration {
    pub lines: Vec<ConfigLine>,
}

#[derive(Debug, PartialEq, Clone)]
pub enum ConfigLine {
    Comment(String),
    SetVar {
        name: String,
        value: String,
    },
    Binding {
        modifiers: Vec<String>,
        key: String,
        command: String,
    },
    ExecCmd(String),
    RawLine(String),
    EmptyLine,
}

impl Display for I3Configuration {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for line in &self.lines {
            match line {
                ConfigLine::Comment(c) => writeln!(f, "{}", c)?,
                ConfigLine::SetVar { name, value } => writeln!(f, "set {}={}", name, value)?,
                ConfigLine::Binding {
                    modifiers,
                    key,
                    command,
                } => writeln!(f, "{} {} {}", key, modifiers.join(","), command)?,
                ConfigLine::ExecCmd(cmd) => writeln!(f, "{}", cmd)?,
                ConfigLine::RawLine(raw) => writeln!(f, "{}", raw)?,
                ConfigLine::EmptyLine => writeln!(f, "")?,
            }
        }

        Ok(())
    }
}
