use std::collections::HashMap;

use i3_lsp::{ConfigLine, I3Configuration};
use tower_lsp::lsp_types::{Diagnostic, DiagnosticSeverity, Position, Range};

pub fn check_for_duplicate_bindings(cfg: &I3Configuration) -> Vec<Diagnostic> {
    let mut seen_bindings: HashMap<String, usize> = HashMap::new();
    let mut diagnostics: Vec<Diagnostic> = Vec::new();

    for (line_num, line) in cfg.lines.iter().enumerate() {
        let ConfigLine::Binding { modifiers, key, .. } = line else {
            continue;
        };

        let binding_key = format!("{}+{}", modifiers.join("+"), key);

        if seen_bindings.contains_key(&binding_key) {
            diagnostics.push(Diagnostic {
                range: Range {
                    start: Position::new(line_num as u32, 0),
                    end: Position::new(line_num as u32, binding_key.len() as u32),
                },
                severity: Some(DiagnosticSeverity::ERROR),
                message: format!("Duplicate key binding: '{}'", binding_key),
                ..Default::default()
            });
        }

        seen_bindings
            .entry(binding_key)
            .and_modify(|v| *v += 1)
            .or_insert(0);
    }
    diagnostics
}

#[cfg(test)]
mod test {
    use crate::{diagnostics::check_for_duplicate_bindings, parser::parse_config};

    #[test]
    fn duplicate_test() {
        let config = r#"# i3 config
                        set $mod Mod4
                        set $term alacritty

                        bindsym $mod+Return exec $term
                        bindsym $mod+Return exec korv
                        exec --no-startup-id nm-applet
                        font pango:monospace 10"#;

        let (_, cfg) = parse_config(config).unwrap();

        let dups = check_for_duplicate_bindings(&cfg);

        assert_eq!(dups.len(), 1);
    }
}
