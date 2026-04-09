use std::collections::{HashMap, HashSet};

use i3_lsp::{ConfigLine, I3Configuration};
use tower_lsp::lsp_types::{Diagnostic, DiagnosticSeverity, Position, Range};

fn line_count(line: &ConfigLine) -> usize {
    match line {
        ConfigLine::Mode(inner_cfg) => {
            1 + inner_cfg.lines.iter().map(line_count).sum::<usize>() + 1
        }
        _ => 1,
    }
}

fn collect_bindings_with_offset(
    lines: &[ConfigLine],
    offset: usize,
    scope: &str,
    out: &mut Vec<(String, usize)>,
) {
    let mut current_line = offset;
    for line in lines.iter() {
        match line {
            ConfigLine::Binding { modifiers, key, .. } => {
                let binding_key = format!("{}:{}+{}", scope, modifiers.join("+"), key);
                out.push((binding_key, current_line));
            }
            ConfigLine::Mode(inner_cfg) => {
                collect_bindings_with_offset(
                    &inner_cfg.lines,
                    current_line + 1,
                    &format!("mode:{}", scope),
                    out,
                );
            }
            _ => {}
        }
        current_line += line_count(line);
    }
}

pub fn check_for_duplicate_bindings(cfg: &I3Configuration) -> Vec<Diagnostic> {
    let mut all_bindings: Vec<(String, usize)> = Vec::new();
    collect_bindings_with_offset(&cfg.lines, 0, "global", &mut all_bindings);

    let mut seen: HashMap<String, usize> = HashMap::new();
    let mut diagnostics: Vec<Diagnostic> = Vec::new();

    for (binding_key, line_num) in &all_bindings {
        if let Some(first_line) = seen.get(binding_key) {
            diagnostics.push(Diagnostic {
                range: Range {
                    start: Position::new(*line_num as u32, 0),
                    end: Position::new(*line_num as u32, binding_key.len() as u32),
                },
                severity: Some(DiagnosticSeverity::ERROR),
                message: format!(
                    "Duplicate key binding: '{}' (first seen on line {})",
                    binding_key,
                    first_line + 1
                ),
                ..Default::default()
            });
        } else {
            seen.insert(binding_key.clone(), *line_num);
        }
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
            ConfigLine::Mode(_) => continue,
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

        let errors = check_variables(&cfg);

        assert_eq!(errors.len(), 1);
    }

    #[test]
    fn mode_duplicate_test() {
        let config = r#"# i3 config
                        mode resize {
                            set $mod Mod4
                            bindsym $mod+r mode "default"
                        }

                        bindsym $mod+r mode "resize"
        "#;

        let (_, cfg) = parse_config(config).unwrap();

        let errors = check_for_duplicate_bindings(&cfg);
        assert_eq!(errors.len(), 0);
    }
}
