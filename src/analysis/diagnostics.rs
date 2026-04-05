use std::collections::HashMap;

use tower_lsp::lsp_types::{Diagnostic, DiagnosticSeverity, Position, Range};

pub enum Statement {
    Set {
        name: String,
        value: String,
        line: usize,
    },
    Bindsym {
        key: String,
        command: String,
        line: usize,
    },
    Exec {
        command: String,
        line: usize,
    },
    Include {
        path: String,
        line: usize,
    },
}

pub fn analyze(ast: &[Statement]) -> Vec<Diagnostic> {
    let mut diagnostics = Vec::new();

    diagnostics.extend(check_duplicate_binds(ast));

    diagnostics
}

fn check_duplicate_binds(ast: &[Statement]) -> Vec<Diagnostic> {
    let mut seen: HashMap<String, Vec<&Statement>> = HashMap::new();

    // 👇 HÄR grupperar du per key
    for stmt in ast {
        if let Statement::Bindsym { key, .. } = stmt {
            let normalized = key.to_lowercase();
            seen.entry(normalized).or_default().push(stmt);
        }
    }

    let mut diagnostics = Vec::new();

    for (key, binds) in seen {
        if binds.len() > 1 {
            for stmt in binds {
                if let Statement::Bindsym { line, .. } = stmt {
                    diagnostics.push(Diagnostic {
                        range: Range {
                            start: Position::new(*line as u32, 0),
                            end: Position::new(*line as u32, 100),
                        },
                        severity: Some(DiagnosticSeverity::WARNING),
                        message: format!("Duplicate keybinding: {}", key),
                        ..Default::default()
                    });
                }
            }
        }
    }

    diagnostics
}
