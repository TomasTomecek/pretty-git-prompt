/* Listing colors and text styles, and translating shell prompt escapes into
 * terminal escape sequences so the result can be displayed in a terminal.
 *
 * The codes which go into `pre_format` and `post_format` are shell prompt
 * expansion syntax (`%{%F{blue}%}` in zsh, `\[\e[38;5;4m\]` in bash) -- a
 * terminal does not interpret those, only a shell does while it renders its
 * prompt. To show what a value looks like, they need to be translated into
 * plain terminal escape sequences first.
 *
 * This module must not depend on any other module of this crate.
 */

use std::env;
use std::io::{self, Write};

// (name known to zsh, 256-color code)
static NAMED_COLORS: [(&'static str, u8); 16] = [
    ("black", 0),
    ("red", 1),
    ("green", 2),
    ("yellow", 3),
    ("blue", 4),
    ("magenta", 5),
    ("cyan", 6),
    ("white", 7),
    ("bright black", 8),
    ("bright red", 9),
    ("bright green", 10),
    ("bright yellow", 11),
    ("bright blue", 12),
    ("bright magenta", 13),
    ("bright cyan", 14),
    ("bright white", 15),
];

// (name, zsh enable, zsh disable, terminal code)
// zsh has a prompt escape only for some of the styles: the rest can be used in
// bash only
static STYLES: [(&'static str, Option<&'static str>, Option<&'static str>, u8); 5] = [
    ("bold", Some("%B"), Some("%b"), 1),
    ("dim", None, None, 2),
    ("italic", None, None, 3),
    ("underline", Some("%U"), Some("%u"), 4),
    ("standout (reverse)", Some("%S"), Some("%s"), 7),
];

const SAMPLE: &'static str = "████";

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Shell {
    Bash,
    Zsh,
}

impl Shell {
    pub fn from_name(name: &str) -> Option<Shell> {
        match name {
            "bash" => Some(Shell::Bash),
            "zsh" => Some(Shell::Zsh),
            _ => None,
        }
    }

    // figure out which shell the user runs from $SHELL
    pub fn detect() -> Option<Shell> {
        let shell = match env::var("SHELL") {
            Ok(s) => s,
            Err(_) => return None,
        };
        match shell.rsplit('/').next() {
            Some(name) => Shell::from_name(name),
            None => None,
        }
    }

    // a config file gives away which shell it was written for
    pub fn from_config(content: &str) -> Option<Shell> {
        if content.contains("%{") {
            Some(Shell::Zsh)
        } else if content.contains("\\[") {
            Some(Shell::Bash)
        } else {
            None
        }
    }

    pub fn name(&self) -> &'static str {
        match *self {
            Shell::Bash => "bash",
            Shell::Zsh => "zsh",
        }
    }
}

// the pair of strings which sets and unsets a foreground color in a config file;
// zsh knows the names of the first eight colors only
fn color_snippet(shell: Shell, color: &str, code_256: u8) -> (String, String) {
    match shell {
        Shell::Zsh => {
            let spec = if code_256 < 8 { String::from(color) } else { code_256.to_string() };
            (format!("%{{%F{{{}}}%}}", spec), String::from("%{%f%}"))
        }
        Shell::Bash => (format!("\\[\\e[38;5;{}m\\]", code_256), String::from("\\[\\e[0m\\]")),
    }
}

// the pair of strings which enables and disables a text style in a config file
fn style_snippet(shell: Shell, zsh_on: Option<&str>, zsh_off: Option<&str>, code: u8)
        -> Option<(String, String)> {
    match shell {
        Shell::Zsh => match (zsh_on, zsh_off) {
            (Some(on), Some(off)) => Some((format!("%{{{}%}}", on), format!("%{{{}%}}", off))),
            _ => None,
        },
        Shell::Bash => Some((format!("\\[\\e[{}m\\]", code), String::from("\\[\\e[0m\\]"))),
    }
}

// resolve a zsh color spec (a name or a number) into a 256-color code
fn color_code(spec: &str) -> Option<u8> {
    let s = spec.trim();
    if let Ok(n) = s.parse::<u16>() {
        if n <= 255 {
            return Some(n as u8);
        }
        return None;
    }
    for &(name, code) in NAMED_COLORS.iter() {
        if name == s {
            return Some(code);
        }
    }
    None
}

fn sgr(code: &str) -> String {
    format!("\x1b[{}m", code)
}

fn fg(code: u8) -> String {
    format!("\x1b[38;5;{}m", code)
}

fn reset() -> String {
    String::from("\x1b[0m")
}

fn translate_zsh(s: &str) -> String {
    let mut out = String::new();
    let mut chars = s.chars().peekable();
    while let Some(c) = chars.next() {
        if c != '%' {
            out.push(c);
            continue;
        }
        let next = match chars.next() {
            Some(n) => n,
            // a trailing '%' is not an escape
            None => {
                out.push('%');
                break;
            }
        };
        match next {
            // these only tell zsh the enclosed text takes no space on screen
            '{' | '}' => (),
            '%' => out.push('%'),
            'F' | 'K' => {
                let layer = if next == 'F' { "38" } else { "48" };
                let mut spec = String::new();
                if chars.peek() == Some(&'{') {
                    chars.next();
                    for ch in chars.by_ref() {
                        if ch == '}' {
                            break;
                        }
                        spec.push(ch);
                    }
                }
                if let Some(code) = color_code(&spec) {
                    out += &sgr(&format!("{};5;{}", layer, code));
                }
            }
            'f' => out += &sgr("39"),
            'k' => out += &sgr("49"),
            'B' => out += &sgr("1"),
            'b' => out += &sgr("22"),
            'U' => out += &sgr("4"),
            'u' => out += &sgr("24"),
            'S' => out += &sgr("7"),
            's' => out += &sgr("27"),
            // prompt escapes we have no use for (%n, %m, %0m, ...) are dropped
            _ => (),
        }
    }
    out
}

fn translate_bash(s: &str) -> String {
    let chars: Vec<char> = s.chars().collect();
    let mut out = String::new();
    let mut i = 0;
    while i < chars.len() {
        if chars[i] != '\\' {
            out.push(chars[i]);
            i += 1;
            continue;
        }
        let rest = &chars[i + 1..];
        match rest.first() {
            // markers for text which takes no space on screen
            Some(&'[') | Some(&']') => i += 2,
            Some(&'e') => {
                out.push('\x1b');
                i += 2;
            }
            Some(&'\\') => {
                out.push('\\');
                i += 2;
            }
            _ => {
                if rest.len() >= 3 && rest[0] == '0' && rest[1] == '3' && rest[2] == '3' {
                    out.push('\x1b');
                    i += 4;
                } else {
                    out.push('\\');
                    i += 1;
                }
            }
        }
    }
    out
}

// drop terminal escape sequences from a string
fn strip_ansi(s: &str) -> String {
    let chars: Vec<char> = s.chars().collect();
    let mut out = String::new();
    let mut i = 0;
    while i < chars.len() {
        if chars[i] != '\x1b' {
            out.push(chars[i]);
            i += 1;
            continue;
        }
        i += 1;
        if i < chars.len() && chars[i] == '[' {
            i += 1;
            // parameter and intermediate bytes, terminated by a final byte
            while i < chars.len() && !('@'..='~').contains(&chars[i]) {
                i += 1;
            }
            if i < chars.len() {
                i += 1;
            }
        }
    }
    out
}

// turn a string formatted for a shell prompt into a string a terminal renders
// the same way; with colors disabled all the formatting is dropped instead
pub fn render(s: &str, shell: Shell, colors: bool) -> String {
    let translated = match shell {
        Shell::Zsh => translate_zsh(s),
        Shell::Bash => translate_bash(s),
    };
    if colors {
        translated
    } else {
        strip_ansi(&translated)
    }
}

// is coloring the output desirable?
pub fn colors_wanted(no_color_flag: bool) -> bool {
    !no_color_flag && env::var("NO_COLOR").is_err()
}

struct Row {
    // terminal escape sequence which formats the row
    format: String,
    label: String,
    // a block of color can't show bold or italic, the label has to carry those
    label_carries_format: bool,
    // config snippet per shell, in the order shells were requested
    snippets: Vec<Option<String>>,
}

fn write_table<W: Write>(out: &mut W, header_label: &str, shells: &[Shell], rows: &[Row],
                         colors: bool) -> io::Result<()> {
    let label_width = rows.iter().map(|r| r.label.chars().count()).max().unwrap_or(0);
    let mut snippet_widths: Vec<usize> = Vec::new();
    for (idx, shell) in shells.iter().enumerate() {
        let mut w = shell.name().chars().count();
        for row in rows {
            if let Some(ref s) = row.snippets[idx] {
                w = w.max(s.chars().count());
            }
        }
        snippet_widths.push(w);
    }

    let mut header = format!("  {:width$}  ", "", width = SAMPLE.chars().count());
    header += &format!("{:width$}  ", header_label, width = label_width);
    for (idx, shell) in shells.iter().enumerate() {
        header += &format!("{:width$}  ", shell.name(), width = snippet_widths[idx]);
    }
    writeln!(out, "{}", header.trim_end())?;

    for row in rows {
        let format = if colors { row.format.clone() } else { String::new() };
        let end = if colors { reset() } else { String::new() };
        let (sample, label) = if row.label_carries_format {
            // padding stays outside the formatting: underline and reverse would
            // stretch over the trailing spaces otherwise
            (" ".repeat(SAMPLE.chars().count()),
             format!("{}{}{}{:width$}", format, row.label, end, "",
                     width = label_width - row.label.chars().count()))
        } else {
            (format!("{}{}{}", format, SAMPLE, end),
             format!("{:width$}", row.label, width = label_width))
        };
        let mut line = format!("  {}  {}  ", sample, label);
        for (idx, _) in shells.iter().enumerate() {
            let snippet = match row.snippets[idx] {
                Some(ref s) => s.clone(),
                None => String::from("-"),
            };
            line += &format!("{:width$}  ", snippet, width = snippet_widths[idx]);
        }
        writeln!(out, "{}", line.trim_end())?;
    }
    Ok(())
}

fn color_rows(shells: &[Shell]) -> Vec<Row> {
    NAMED_COLORS.iter().map(|&(name, code)| {
        let snippets = shells.iter().map(|shell| {
            let (pre, post) = color_snippet(*shell, name, code);
            Some(format!("{}…{}", pre, post))
        }).collect();
        Row {
            format: fg(code),
            label: format!("{} ({})", name, code),
            label_carries_format: false,
            snippets: snippets,
        }
    }).collect()
}

fn style_rows(shells: &[Shell]) -> Vec<Row> {
    STYLES.iter().map(|&(name, zsh_on, zsh_off, code)| {
        let snippets = shells.iter().map(|shell| {
            style_snippet(*shell, zsh_on, zsh_off, code)
                .map(|(pre, post)| format!("{}…{}", pre, post))
        }).collect();
        Row {
            format: sgr(&code.to_string()),
            label: String::from(name),
            label_carries_format: true,
            snippets: snippets,
        }
    }).collect()
}

fn write_palette_256<W: Write>(out: &mut W, colors: bool) -> io::Result<()> {
    for chunk_start in (0..256).step_by(16) {
        let mut line = String::from("  ");
        for code in chunk_start..chunk_start + 16 {
            let code = code as u8;
            if colors {
                // dark cells need light text and the other way around
                let text = if is_dark(code) { 255 } else { 232 };
                line += &format!("\x1b[48;5;{}m\x1b[38;5;{}m {:>3} {}", code, text, code, reset());
            } else {
                line += &format!(" {:>3} ", code);
            }
        }
        writeln!(out, "{}", line)?;
    }
    Ok(())
}

// rough guess whether a 256-color code is dark enough to need light text on top
fn is_dark(code: u8) -> bool {
    match code {
        0..=7 => code != 7,
        8..=15 => false,
        // 6x6x6 color cube: judge by the green component, it dominates luminance
        16..=231 => ((code - 16) / 6) % 6 <= 2,
        // grayscale ramp
        _ => code < 244,
    }
}

pub fn list_colors<W: Write>(out: &mut W, shells: &[Shell], colors: bool) -> io::Result<()> {
    writeln!(out, "Colors")?;
    writeln!(out)?;
    write_table(out, "color", shells, &color_rows(shells), colors)?;
    writeln!(out)?;
    writeln!(out, "Text styles")?;
    writeln!(out)?;
    write_table(out, "style", shells, &style_rows(shells), colors)?;
    writeln!(out)?;
    writeln!(out, "256 colors")?;
    writeln!(out)?;
    write_palette_256(out, colors)?;
    writeln!(out)?;
    for shell in shells {
        let (pre, post) = color_snippet(*shell, "141", 141);
        writeln!(out, "  {}: any of the codes above fits in place of 141: {}…{}",
                 shell.name(), pre, post)?;
    }
    writeln!(out)?;
    writeln!(out, "Put the codes in 'pre_format' and 'post_format' of a value in your \
                   config file, then run 'pretty-git-prompt preview --demo' to see the result.")?;
    Ok(())
}


#[cfg(test)]
mod tests {
    use colors::*;

    #[test]
    fn test_shell_from_name() {
        assert_eq!(Shell::from_name("zsh"), Some(Shell::Zsh));
        assert_eq!(Shell::from_name("bash"), Some(Shell::Bash));
        assert_eq!(Shell::from_name("fish"), None);
    }

    #[test]
    fn test_shell_from_config() {
        assert_eq!(Shell::from_config("pre_format: '%{%F{blue}%}<LOCAL_BRANCH>'"), Some(Shell::Zsh));
        assert_eq!(Shell::from_config("pre_format: '\\[\\e[38;5;4m\\]x'"), Some(Shell::Bash));
        assert_eq!(Shell::from_config("pre_format: 'Δ'"), None);
    }

    #[test]
    fn test_render_zsh_colors() {
        // taken from files/pretty-git-prompt.yml.zsh
        assert_eq!(render("%{%F{blue}%}master%{%f%}", Shell::Zsh, true),
                   "\x1b[38;5;4mmaster\x1b[39m");
        assert_eq!(render("%{%B%F{red}%}Δ1%{%b%f%}", Shell::Zsh, true),
                   "\x1b[1m\x1b[38;5;1mΔ1\x1b[22m\x1b[39m");
        assert_eq!(render("%{%F{014}%}✚1%{%f%}", Shell::Zsh, true),
                   "\x1b[38;5;14m✚1\x1b[39m");
        assert_eq!(render("%{%K{4}%}x%{%k%}", Shell::Zsh, true), "\x1b[48;5;4mx\x1b[49m");
    }

    #[test]
    fn test_render_zsh_unknown_escapes() {
        assert_eq!(render("%n@%m %{%F{nope}%}x%{%f%}", Shell::Zsh, true), "@ x\x1b[39m");
        assert_eq!(render("100%% done", Shell::Zsh, true), "100% done");
        assert_eq!(render("trailing %", Shell::Zsh, true), "trailing %");
    }

    #[test]
    fn test_render_bash_colors() {
        // taken from files/pretty-git-prompt.yml.bash
        assert_eq!(render("\\[\\e[38;5;4m\\]master\\[\\e[0m\\]", Shell::Bash, true),
                   "\x1b[38;5;4mmaster\x1b[0m");
        assert_eq!(render("\\[\\033[1m\\]x\\[\\033[0m\\]", Shell::Bash, true),
                   "\x1b[1mx\x1b[0m");
        assert_eq!(render("a\\\\b", Shell::Bash, true), "a\\b");
        assert_eq!(render("50\\% \\q", Shell::Bash, true), "50\\% \\q");
    }

    #[test]
    fn test_render_keeps_plain_text_intact() {
        for shell in [Shell::Bash, Shell::Zsh].iter() {
            assert_eq!(render("master│✚1Δ1", *shell, true), "master│✚1Δ1");
            assert_eq!(render("master│✚1Δ1", *shell, false), "master│✚1Δ1");
        }
    }

    #[test]
    fn test_render_without_colors_drops_formatting() {
        assert_eq!(render("%{%B%F{red}%}Δ1%{%b%f%}", Shell::Zsh, false), "Δ1");
        assert_eq!(render("\\[\\e[38;5;4m\\]master\\[\\e[0m\\]", Shell::Bash, false), "master");
    }

    #[test]
    fn test_color_code() {
        assert_eq!(color_code("blue"), Some(4));
        assert_eq!(color_code("bright blue"), Some(12));
        assert_eq!(color_code("014"), Some(14));
        assert_eq!(color_code("255"), Some(255));
        assert_eq!(color_code("256"), None);
        assert_eq!(color_code("mauve"), None);
    }

    #[test]
    fn test_list_colors_is_plain_text_without_colors() {
        let mut buf: Vec<u8> = Vec::new();
        list_colors(&mut buf, &[Shell::Zsh, Shell::Bash], false).unwrap();
        let out = String::from_utf8(buf).unwrap();
        assert!(!out.contains('\x1b'));
        assert!(out.contains("%{%F{blue}%}…%{%f%}"));
        assert!(out.contains("\\[\\e[38;5;4m\\]…\\[\\e[0m\\]"));
        // zsh has no prompt escape for italic
        assert!(out.contains("italic"));
        assert!(out.contains("255"));
    }

    #[test]
    fn test_list_colors_emits_escapes() {
        let mut buf: Vec<u8> = Vec::new();
        list_colors(&mut buf, &[Shell::Zsh], true).unwrap();
        let out = String::from_utf8(buf).unwrap();
        assert!(out.contains("\x1b[38;5;4m"));
        // a block of color would look the same bold or not, so the label is styled
        assert!(out.contains("\x1b[1mbold"));
        assert!(out.contains("\x1b[3mitalic"));
        assert!(!out.contains("bash"));
    }
}
