pub struct NoColorFormatter {

}

pub struct ZshColorFormatter {

}

pub trait CanFormat {
    fn format(&self, f: FormatEntry) -> String;
}

pub struct FormatEntry {
    pub text: String,
    pub color: String,
}

impl Clone for FormatEntry {
    fn clone(&self) -> Self {
        FormatEntry{text: self.text.clone(), color: self.color.clone()}
    }
}


impl CanFormat for ZshColorFormatter {
    fn format(&self, f: FormatEntry) -> String {
        // {% ... %} correct word wrap with zsh
        // TODO: figure out when color is ""
        format!("%{{%F{{{color}}}%}}{s}%{{%f%}}", color=f.color, s=f.text)
    }
}

impl CanFormat for NoColorFormatter {
    fn format(&self, f: FormatEntry) -> String {
        f.text
    }
}

