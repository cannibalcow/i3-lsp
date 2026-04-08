use std::collections::{HashMap, HashSet};

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

pub fn check_variables(cfg: &I3Configuration) -> Vec<Diagnostic> {
    let variables: HashSet<&String> = cfg
        .lines
        .iter()
        .filter_map(|line| {
            if let ConfigLine::SetVar { name, .. } = line {
                Some(name)
            } else {
                None
            }
        })
        .collect();

    println!("vars: {:?}", variables);

    let mut diagnostics: Vec<Diagnostic> = Vec::new();

    for (line_num, line) in cfg.lines.iter().enumerate() {
        match line {
            ConfigLine::Comment(_) => continue,
            ConfigLine::SetVar {
                name: _name,
                value: _value,
            } => continue,
            ConfigLine::Binding {
                modifiers,
                key,
                command,
            } => {
                println!(
                    "Modfiires: {:?}, Key: {:?} Command: {:?}",
                    modifiers, key, command
                );
                let mut vars: Vec<&str> = modifiers
                    .iter()
                    .filter(|f| f.starts_with('$'))
                    .map(|it| &it[1..])
                    .collect();

                if key.starts_with('$') {
                    vars.push(&key[1..]);
                }

                let c = format!("{}+{} {}", modifiers.join("+"), key, command);

                println!("binding vars: {:?}", vars);

                vars.iter().for_each(|item| {
                    if !variables.contains(&item.to_string()) {
                        diagnostics.push(Diagnostic {
                            range: Range {
                                start: Position::new(line_num as u32, 0),
                                end: Position::new(line_num as u32, c.len() as u32),
                            },
                            severity: Some(DiagnosticSeverity::ERROR),
                            message: format!("Unknown variable '{}'", item).into(),
                            ..Default::default()
                        });
                    }
                });
            }
            ConfigLine::ExecCmd(_) => continue,
            ConfigLine::RawLine(_) => continue,
            ConfigLine::EmptyLine => continue,
        }
    }

    diagnostics
}

#[cfg(test)]
mod test {
    use crate::{
        diagnostics::{check_for_duplicate_bindings, check_variables},
        parser::parse_config,
    };

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

    #[test]
    fn check_variables_test() {
        let config = r#"# i3 config
                        set $mod Mod4
                        set $term alacritty

                        bindsym $mo+Return exec $term
                        bindsym $mod+Return exec korv
                        exec --no-startup-id nm-applet
                        font pango:monospace 10"#;

        let (_, cfg) = parse_config(config).unwrap();

        let dups = check_variables(&cfg);

        assert_eq!(dups.len(), 1);
    }
}
