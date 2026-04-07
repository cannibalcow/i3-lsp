use i3_lsp::{ConfigLine, I3Configuration};

use nom::{
    IResult, Parser,
    branch::alt,
    bytes::complete::{tag, take_till, take_while},
    character::complete::{char, line_ending, not_line_ending, space0, space1},
    combinator::opt,
    multi::many0,
    sequence::preceded,
};

/**
 * Parse comment
 **/
fn parse_comment(input: &str) -> IResult<&str, ConfigLine> {
    let (input, _) = preceded(space0, char('#')).parse(input)?;
    let (input, text) = not_line_ending(input)?;
    Ok((input, ConfigLine::Comment(text.trim().to_string())))
}

fn identifier(input: &str) -> IResult<&str, &str> {
    take_while(|c: char| c.is_alphanumeric() || c == '_' || c == '-')(input)
}

fn parse_set(input: &str) -> IResult<&str, ConfigLine> {
    let (input, _) = (space0, tag("set"), space1).parse(input)?;
    let (input, name) = preceded(char('$'), identifier).parse(input)?;
    let (input, _) = space1(input)?;
    let (input, value) = not_line_ending(input)?;

    Ok((
        input,
        ConfigLine::SetVar {
            name: name.to_string(),
            value: value.trim().to_string(),
        },
    ))
}

fn parse_binding(input: &str) -> IResult<&str, ConfigLine> {
    let (input, _) = (space0, alt((tag("bindsym"), tag("bindcode"))), space1).parse(input)?;
    let (input, combo) = take_till(|c: char| c == ' ' || c == '\n')(input)?;
    let (input, _) = space1(input)?;
    let (input, command) = not_line_ending(input)?;

    let parts: Vec<String> = combo.split('+').map(|s| s.to_string()).collect();
    let combo_string = combo.to_string();
    let (key, modifiers) = parts.split_last().unwrap_or((&combo_string, &[]));

    Ok((
        input,
        ConfigLine::Binding {
            modifiers: modifiers.to_vec(),
            key: key.to_string(),
            command: command.trim().to_string(),
        },
    ))
}

fn parse_exec(input: &str) -> IResult<&str, ConfigLine> {
    let (input, _) = (space0, tag("exec")).parse(input)?;
    let (input, _) = opt((space1, tag("--no-startup-id"))).parse(input)?;
    let (input, _) = space1(input)?;
    let (input, cmd) = not_line_ending(input)?;

    Ok((input, ConfigLine::ExecCmd(cmd.trim().to_string())))
}

fn parse_raw(input: &str) -> IResult<&str, ConfigLine> {
    let (input, line) = not_line_ending(input)?;
    Ok((input, ConfigLine::RawLine(line.trim().to_string())))
}

fn parse_line(input: &str) -> IResult<&str, ConfigLine> {
    if input.is_empty() {
        return Err(nom::Err::Error(nom::error::Error::new(
            input,
            nom::error::ErrorKind::Eof,
        )));
    }

    if input.starts_with('\n') || input.starts_with("\r\n") {
        let (input, _) = line_ending(input)?;
        return Ok((input, ConfigLine::EmptyLine));
    }

    let (input, line) = alt((
        parse_comment,
        parse_set,
        parse_binding,
        parse_exec,
        parse_raw,
    ))
    .parse(input)?;

    let (input, _) = opt(line_ending).parse(input)?;
    Ok((input, line))
}

pub fn parse_config(document: &str) -> IResult<&str, I3Configuration> {
    let (document, lines) = many0(parse_line).parse(document)?;

    Ok((document, I3Configuration { lines: lines }))
}

#[cfg(test)]
mod test {
    use i3_lsp::{ConfigLine, I3Configuration};

    use crate::parser::{
        parse_binding, parse_comment, parse_config, parse_exec, parse_raw, parse_set,
    };

    #[test]
    fn comment_test() {
        let cfg = "# i3 configuration";

        let (_, cfg_line) = parse_comment(cfg).unwrap();

        assert_eq!(
            cfg_line,
            ConfigLine::Comment("i3 configuration".to_string())
        );
    }

    #[test]
    fn set_test() {
        let config = "set $mod Mod4";

        let (_, cfg_line) = parse_set(config).unwrap();

        assert_eq!(
            cfg_line,
            ConfigLine::SetVar {
                name: "mod".to_string(),
                value: "Mod4".to_string()
            }
        )
    }

    #[test]
    fn binding_test() {
        let (_, cfg) = parse_binding("bindsym $mod+Shift+F11 shutdown").unwrap();

        assert_eq!(
            cfg,
            ConfigLine::Binding {
                modifiers: vec!["$mod".into(), "Shift".into()],
                key: "F11".into(),
                command: "shutdown".into()
            }
        )
    }

    #[test]
    fn exec_test() {
        let (_, cfg) = parse_exec("exec --no-startup-id cowsay hello").unwrap();

        assert_eq!(cfg, ConfigLine::ExecCmd("cowsay hello".into()))
    }

    #[test]
    fn raw_test() {
        let (_, cfg) = parse_raw("font pango:monospace 10").unwrap();

        assert_eq!(cfg, ConfigLine::RawLine("font pango:monospace 10".into()));
    }

    #[test]
    fn empty_line() {
        let (_, cfg) = parse_raw("").unwrap();

        assert_eq!(cfg, ConfigLine::RawLine("".into()));
    }

    #[test]
    fn starts_with_linebreak() {
        let (_, cfg) = parse_raw("\n").unwrap();

        assert_eq!(cfg, ConfigLine::RawLine("".into()));
    }

    // parse_raw,
    #[test]
    fn full_test() {
        let config = r#"# i3 config
                        set $mod Mod4
                        set $term alacritty

                        bindsym $mod+Return exec $term
                        exec --no-startup-id nm-applet
                        font pango:monospace 10"#;

        let (_, cfg) = parse_config(config).unwrap();

        assert_eq!(
            cfg,
            I3Configuration {
                lines: vec![
                    ConfigLine::Comment("i3 config".into()),
                    ConfigLine::SetVar {
                        name: "mod".into(),
                        value: "Mod4".into()
                    },
                    ConfigLine::SetVar {
                        name: "term".into(),
                        value: "alacritty".into()
                    },
                    ConfigLine::EmptyLine,
                    ConfigLine::Binding {
                        modifiers: vec!["$mod".into()],
                        key: "Return".into(),
                        command: "exec $term".into()
                    },
                    ConfigLine::ExecCmd("nm-applet".into()),
                    ConfigLine::RawLine("font pango:monospace 10".into())
                ]
            }
        );
    }
}
