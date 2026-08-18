/* Rendering the prompt into a terminal: either for the current repository or
 * for a set of made up repository states.
 */

use std::collections::HashMap;
use std::io::{self, Write};

use backend::{Backend,DemoData};
use colors::{Shell,render};
use conf::Conf;
use constants::*;
use models::DisplayMaster;

use yaml_rust::Yaml;

fn statuses(pairs: &[(&str, u32)]) -> HashMap<String, u32> {
    let mut h: HashMap<String, u32> = HashMap::new();
    for &(key, count) in pairs {
        h.insert(key.to_string(), count);
    }
    h
}

// repository states worth looking at while tuning a config file
fn scenarios() -> Vec<(&'static str, DemoData)> {
    let mut result: Vec<(&'static str, DemoData)> = Vec::new();

    result.push(("clean repository", DemoData::new("master")));

    let mut dirty = DemoData::new("master");
    dirty.file_statuses = statuses(&[(NEW_KEY, 3), (CHANGED_KEY, 2), (STAGED_KEY, 1)]);
    result.push(("new, changed and staged files", dirty));

    let mut diverged = DemoData::new("feature");
    diverged.ahead = 2;
    diverged.behind = 1;
    result.push(("diverged from the remote branch", diverged));

    let mut no_remote = DemoData::new("feature");
    no_remote.remote_name = None;
    result.push(("branch without a remote counterpart", no_remote));

    let mut tagged = DemoData::new("master");
    tagged.tag = Some(String::from("0.3.0"));
    result.push(("tag pointing at HEAD", tagged));

    let mut conflict = DemoData::new("master");
    conflict.repository_state = String::from("merge");
    conflict.file_statuses = statuses(&[(CONFLICTS_KEY, 1), (CHANGED_KEY, 1)]);
    result.push(("conflict during a merge", conflict));

    let mut stashed = DemoData::new("master");
    stashed.stash_count = 2;
    result.push(("stashed changes", stashed));

    let mut detached = DemoData::new("2a4b9c1");
    detached.remote_name = None;
    detached.file_statuses = statuses(&[(CHANGED_KEY, 1)]);
    result.push(("detached HEAD", detached));

    result
}

fn render_config(yaml: &Yaml, backend: Backend, debug: bool) -> String {
    let dm: DisplayMaster = DisplayMaster::new(backend, debug);
    let mut conf = Conf::new(yaml.clone(), dm);
    conf.populate_values()
}

// print the prompt for every demo scenario
pub fn preview_demo<W: Write>(out: &mut W, yaml: &Yaml, shell: Shell, colors: bool, debug: bool)
        -> io::Result<()> {
    let all = scenarios();
    let label_width = all.iter().map(|&(label, _)| label.chars().count()).max().unwrap_or(0);
    for (label, demo) in all {
        let backend = Backend::new_demo(demo, debug);
        let value = render_config(yaml, backend, debug);
        writeln!(out, "  {:width$}  {}", label, render(&value, shell, colors), width = label_width)?;
    }
    Ok(())
}

// print the prompt for the repository we are in
pub fn preview_repo<W: Write>(out: &mut W, yaml: &Yaml, backend: Backend, shell: Shell,
                              colors: bool, debug: bool) -> io::Result<()> {
    let value = render_config(yaml, backend, debug);
    writeln!(out, "{}", render(&value, shell, colors))
}


#[cfg(test)]
mod tests {
    use colors::Shell;
    use preview::preview_demo;
    use yaml_rust::YamlLoader;

    static CONFIG: &'static str = "version: '1'
values:
    - type: repository_state
      pre_format: '%{%F{red}%}'
      post_format: '%{%f%}'
    - type: separator
      display: surrounded
      pre_format: '│'
      post_format: ''
    - type: remote_difference
      display_if_uptodate: true
      pre_format: ''
      post_format: ''
      values:
        - type: name
          pre_format: '%{%F{blue}%}<LOCAL_BRANCH>'
          post_format: '%{%f%}'
        - type: ahead
          pre_format: '↑'
          post_format: ''
        - type: behind
          pre_format: '↓'
          post_format: ''
    - type: changed
      pre_format: 'Δ'
      post_format: ''
    - type: stash
      pre_format: '☐'
      post_format: ''";

    // (label, rendered prompt) for every line of the preview
    fn rendered_scenarios(colors: bool) -> Vec<(String, String)> {
        let docs = YamlLoader::load_from_str(CONFIG).unwrap();
        let mut buf: Vec<u8> = Vec::new();
        preview_demo(&mut buf, &docs[0], Shell::Zsh, colors, false).unwrap();
        let out = String::from_utf8(buf).unwrap();
        out.lines().map(|line| {
            let mut parts = line.trim().splitn(2, "  ");
            let label = parts.next().unwrap().trim().to_string();
            let value = parts.next().unwrap_or("").trim().to_string();
            (label, value)
        }).collect()
    }

    #[test]
    fn test_preview_demo_renders_every_scenario() {
        // no repository is needed to render these
        let expected = vec!(
            ("clean repository", "master"),
            ("new, changed and staged files", "masterΔ2"),
            ("diverged from the remote branch", "feature↑2↓1"),
            ("branch without a remote counterpart", "feature"),
            ("tag pointing at HEAD", "master"),
            ("conflict during a merge", "merge│masterΔ1"),
            ("stashed changes", "master☐2"),
            ("detached HEAD", "2a4b9c1Δ1"),
        );
        let rendered = rendered_scenarios(false);
        assert_eq!(rendered.len(), expected.len());
        for (idx, &(label, value)) in expected.iter().enumerate() {
            assert_eq!(rendered[idx], (label.to_string(), value.to_string()));
        }
    }

    #[test]
    fn test_preview_demo_translates_colors() {
        let docs = YamlLoader::load_from_str(CONFIG).unwrap();
        let mut buf: Vec<u8> = Vec::new();
        preview_demo(&mut buf, &docs[0], Shell::Zsh, true, false).unwrap();
        let out = String::from_utf8(buf).unwrap();
        assert!(out.contains("\x1b[38;5;4mmaster\x1b[39m"));
        assert!(!out.contains("%{"));
    }
}
