pub enum FormatType {
    Zsh,
    NoColor
}

pub struct FormatEntry {
    pub text: String,
    pub color: String
}

fn format_for_zsh(f: FormatEntry) -> String {
    // {% ... %} correct word wrap with zsh
    if f.text.is_empty() || f.color.is_empty() {
        f.text
    } else {
        format!("%{{%F{{{color}}}%}}{s}%{{%f%}}", color=f.color, s=f.text)
    }
}

fn format_without_color(f: FormatEntry) -> String {
    f.text
}

pub struct Format {
    pub format_type: FormatType
}

impl Format {
    pub fn format(&self, f: FormatEntry) -> String {
        match self.format_type {
            FormatType::Zsh => format_for_zsh(f),
            FormatType::NoColor => format_without_color(f),
        }
    }
}


#[test]
fn test_format_for_zsh() {
    let fe = FormatEntry{text: String::from("asd"), color: String::from("white")};
    let result = format_for_zsh(fe);
    assert_eq!(result, "%{%F{white}%}asd%{%f%}");
}


#[test]
fn test_format_f_zsh() {
    let fe = FormatEntry{text: String::from("asd"), color: String::from("white")};
    let f = Format{format_type: FormatType::Zsh};

    let result = f.format(fe);
    assert_eq!(result, "%{%F{white}%}asd%{%f%}");
}


#[test]
fn test_format_without_color() {
    let fe = FormatEntry{text: String::from("asd"), color: String::from("")};
    let result = format_without_color(fe);
    assert_eq!(result, "asd");
}


#[test]
fn test_format_f_no_color() {
    let fe = FormatEntry{text: String::from("asd"), color: String::from("")};
    let f = Format{format_type: FormatType::NoColor};

    let result = f.format(fe);
    assert_eq!(result, "asd");
}


#[test]
fn test_zsh_color_blank_str() {
    let fe = FormatEntry{text: String::from(""), color: String::from("white")};
    let f = Format{format_type: FormatType::Zsh};

    let result = f.format(fe);
    assert_eq!(result, "");
}
