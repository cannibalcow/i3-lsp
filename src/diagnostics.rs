use std::collections::HashMap;

use i3_lsp::{ConfigLine, I3Configuration};
use tower_lsp::lsp_types::{Diagnostic, DiagnosticSeverity, Position, Range};

pub fn check_for_duplicate_bindings(cfg: &I3Configuration) -> Vec<Diagnostic> {
    let mut bindings: HashMap<String, usize> = HashMap::new();
    let mut digag: Vec<Diagnostic> = Vec::new();

    for (line_num, line) in cfg.lines.iter().enumerate() {
        if let ConfigLine::Binding {
            modifiers,
            key,
            command: _command,
        } = line
        {
            let binding_key = format!("{}+{}", modifiers.join("+"), key);
            if bindings.contains_key(&binding_key) {
                digag.push(Diagnostic {
                    range: Range {
                        start: Position::new(line_num as u32, 0),
                        end: Position::new(line_num as u32, key.len() as u32),
                    },
                    severity: Some(DiagnosticSeverity::ERROR),
                    message: format!("Duplicate binding: '{}'", key),
                    ..Default::default()
                });
            }

            bindings
                .entry(binding_key)
                .and_modify(|v| *v += 1)
                .or_insert(0);
        }
    }

    return digag;
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
