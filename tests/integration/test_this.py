from .utils import *


def test_bare_repo(tmpdir):
    with BareRepo(tmpdir) as r:
        assert r.run() == "master"


def test_simple_untracked_files_repo(tmpdir):
    with SimpleUntrackedFilesRepo(tmpdir) as r:
        assert r.run() == "master│✚1"


def test_changed_files_repo(tmpdir):
    with SimpleChangedFilesRepo(tmpdir) as r:
        assert r.run() == "master│▶1"


def test_simple_repo(tmpdir):
    with SimpleRepo(tmpdir) as r:
        assert r.run() == "master"


def test_simple_dirty_with_commit_repo(tmpdir):
    with SimpleDirtyWithCommitRepo(tmpdir) as r:
        assert r.run() == "master│Δ1"


def test_repo_with_origin(tmpdir):
    with RepoWithOrigin(tmpdir) as r:
        assert r.run() == "master"


def test_rwo_and_upstream(tmpdir):
    with RWOAndUpstream(tmpdir) as r:
        assert r.run() == "master↑1│upstream↑1↓1"


def test_empty_repo_with_fetched_upstream(tmpdir):
    config = """\
---
version: '1'
values:
    - type: remote_difference
      remote_branch: 'upstream/master'
      display_if_uptodate: true
      pre_format: ''
      post_format: ''
      values:
        - type: name
          pre_format: '<LOCAL_BRANCH>'
          post_format: ''
        - type: ahead
          pre_format: '↑'
          post_format: ''
        - type: behind
          pre_format: '↓'
          post_format: ''"""
    with EmptyRepoWithFetchedUpstream(tmpdir) as r:
        assert r.run() == "master"
        assert r.run(custom_config_content=config) == ""


def test_rwo_no_tracking(tmpdir):
    with RWOWithoutTracking(tmpdir) as r:
        assert r.run() == "master"


def test_rwo_local_commits(tmpdir):
    with RWOLocalCommits(tmpdir) as r:
        assert r.run() == "master↑1"


def test_rwo_remote_commits(tmpdir):
    with RWORemoteCommits(tmpdir) as r:
        assert r.run() == "master↓1"


def test_rwo_detached(tmpdir):
    with RWODetached(tmpdir) as r:
        assert r.run() == r.co_commit[:7]


def test_lightweight_tag(tmpdir):
    with TaggedRepo(tmpdir) as r:
        assert r.run() == "master│#v1.0.0"


def test_annotated_tag(tmpdir):
    with AnnotatedTaggedRepo(tmpdir) as r:
        assert r.run() == "master│#v1.0.0"


def test_tag_not_on_head(tmpdir):
    with TaggedRepoWithNewerCommit(tmpdir) as r:
        assert r.run() == "master"


def test_checked_out_tag(tmpdir):
    with CheckedOutTag(tmpdir) as r:
        assert r.run() == "%s│#v1.0.0" % r.tagged_commit[:7]


def test_merge_conflict(tmpdir):
    with MergeConflict(tmpdir) as r:
        assert r.run() == "merge│master↑1│✖1"


def test_stashed(tmpdir):
    with SimpleRepoWithStashedContent(tmpdir) as r:
        assert r.run() == "master│☐1"


def test_global_separator(tmpdir):
    config = """\
---
version: '1'
values:
    - type: separator
      display: always
      pre_format: (
      post_format: ''
    - type: separator
      display: always
      pre_format: )
      post_format: ''"""
    print(config)
    with SimpleRepo(tmpdir) as r:
        assert r.run(custom_config_content=config) == "()"


def test_global_with_value(tmpdir):
    config = """\
---
version: '1'
values:
    - type: separator
      display: always
      pre_format: (
      post_format: ''
    - type: remote_difference
      display_if_uptodate: true
      pre_format: ''
      post_format: ''
      values:
        - type: name
          pre_format: '<LOCAL_BRANCH>'
          post_format: ''
        - type: ahead
          pre_format: '↑'
          post_format: ''
        - type: behind
          pre_format: '↓'
          post_format: ''
    - type: separator
      display: always
      pre_format: )
      post_format: ''"""
    print(config)
    with SimpleRepo(tmpdir) as r:
        assert r.run(custom_config_content=config) == "(master)"


def test_surrounded_separator(tmpdir):
    config = """\
---
version: '1'
values:
    - type: separator
      display: surrounded
      pre_format: (
      post_format: ''
    - type: remote_difference
      display_if_uptodate: true
      pre_format: ''
      post_format: ''
      values:
        - type: name
          pre_format: '<LOCAL_BRANCH>'
          post_format: ''
        - type: ahead
          pre_format: '↑'
          post_format: ''
        - type: behind
          pre_format: '↓'
          post_format: ''
    - type: separator
      display: surrounded
      pre_format: )
      post_format: ''"""
    print(config)
    with SimpleRepo(tmpdir) as r:
        assert r.run(custom_config_content=config) == "(master)"


def test_surrounded_separator_in_the_middle(tmpdir):
    config = """\
---
version: '1'
values:
    - type: remote_difference
      display_if_uptodate: true
      pre_format: ''
      post_format: ''
      values:
        - type: name
          pre_format: '<LOCAL_BRANCH>'
          post_format: ''
        - type: ahead
          pre_format: '\u2191'
          post_format: ''
        - type: behind
          pre_format: '\u2193'
          post_format: ''
    - type: separator
      display: surrounded
      pre_format: '|'
      post_format: ''
    - type: new
      pre_format: '+'
      post_format: ''"""
    print(config)
    with SimpleRepo(tmpdir) as r:
        # there are no untracked files, hence the separator is not displayed
        assert r.run(custom_config_content=config) == "master"


def test_surrounded_separator_blank(tmpdir):
    config = """\
---
version: '1'
values:
    - type: separator
      display: surrounded
      pre_format: (
      post_format: ''
    - type: remote_difference
      remote_branch: 'upstream/master'
      display_if_uptodate: false
      pre_format: ''
      post_format: ''
      values:
        - type: name
          pre_format: '<LOCAL_BRANCH>'
          post_format: ''
        - type: ahead
          pre_format: '↑'
          post_format: ''
        - type: behind
          pre_format: '↓'
          post_format: ''
    - type: separator
      display: surrounded
      pre_format: )
      post_format: ''"""
    print(config)
    with SimpleRepo(tmpdir) as r:
        assert r.run(custom_config_content=config) == ""


REMOTE_FIRST_LETTER_CONFIG = """\
---
version: '1'
values:
    - type: remote_difference
      display_if_uptodate: true
      pre_format: ''
      post_format: ''
      values:
        - type: name
          pre_format: '<REMOTE_FIRST_LETTER>/<LOCAL_BRANCH>'
          post_format: ''
        - type: ahead
          pre_format: '↑'
          post_format: ''
        - type: behind
          pre_format: '↓'
          post_format: ''"""


def test_remote_first_letter(tmpdir):
    with RWOLocalCommits(tmpdir) as r:
        assert r.run(custom_config_content=REMOTE_FIRST_LETTER_CONFIG) == "o/master↑1"


def test_remote_first_letter_without_remote(tmpdir):
    with SimpleRepo(tmpdir) as r:
        assert r.run(custom_config_content=REMOTE_FIRST_LETTER_CONFIG) == "_/master"


def test_remote_first_letter_distinguishes_remotes(tmpdir):
    config = """\
---
version: '1'
values:
    - type: remote_difference
      display_if_uptodate: true
      pre_format: ''
      post_format: ''
      values:
        - type: name
          pre_format: '<REMOTE_FIRST_LETTER>/<LOCAL_BRANCH>'
          post_format: ''
    - type: separator
      display: always
      pre_format: '│'
      post_format: ''
    - type: remote_difference
      remote_branch: 'upstream/master'
      display_if_uptodate: true
      pre_format: ''
      post_format: ''
      values:
        - type: name
          pre_format: '<REMOTE_FIRST_LETTER>/<LOCAL_BRANCH>'
          post_format: ''"""
    with RWOAndUpstream(tmpdir) as r:
        assert r.run(custom_config_content=config) == "o/master│u/master"


def test_tracked_remote_name(tmpdir):
    config = """\
---
version: '1'
values:
    - type: remote_difference
      display_if_uptodate: true
      pre_format: ''
      post_format: ''
      values:
        - type: name
          pre_format: '<REMOTE>/<LOCAL_BRANCH>'
          post_format: ''"""
    with RWOLocalCommits(tmpdir) as r:
        assert r.run(custom_config_content=config) == "origin/master"


def test_remote_first_letter_custom_placeholder(tmpdir):
    config = """\
---
version: '1'
values:
    - type: remote_difference
      display_if_uptodate: true
      no_remote_placeholder: '∅'
      pre_format: ''
      post_format: ''
      values:
        - type: name
          pre_format: '<REMOTE_FIRST_LETTER>/<LOCAL_BRANCH>'
          post_format: ''"""
    with SimpleRepo(tmpdir) as r:
        assert r.run(custom_config_content=config) == "∅/master"


ZSH_CONFIG = """\
---
version: '1'
values:
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
      pre_format: '%{%B%F{red}%}Δ'
      post_format: '%{%b%f%}'"""


def run_outside_repo(tmpdir, args):
    return subprocess.check_output(
        ["pretty-git-prompt"] + args, cwd=str(tmpdir)).decode("utf-8")


def test_list_colors_works_outside_a_repository(tmpdir):
    out = run_outside_repo(tmpdir, ["list-colors", "--shell", "zsh"])
    assert "%{%F{blue}%}…%{%f%}" in out
    assert "%{%B%}…%{%b%}" in out
    # the color is also rendered, so that it can be seen in a terminal
    assert "\x1b[38;5;4m" in out


def test_list_colors_for_bash(tmpdir):
    out = run_outside_repo(tmpdir, ["list-colors", "--shell", "bash"])
    assert "\\[\\e[38;5;4m\\]…\\[\\e[0m\\]" in out
    assert "%{%F{blue}%}" not in out


def test_list_colors_no_color(tmpdir):
    out = run_outside_repo(tmpdir, ["list-colors", "--no-color"])
    assert "\x1b" not in out


def test_preview_demo(tmpdir):
    with SimpleRepo(tmpdir) as r:
        out = r.run(custom_config_content=ZSH_CONFIG, args=["preview", "--demo"])
    # prompt escapes are translated into terminal escape sequences
    assert "\x1b[38;5;4mmaster\x1b[39m" in out
    assert "%{" not in out
    # every scenario is rendered, including ones the repository is not in
    assert "\x1b[38;5;4mfeature\x1b[39m↑2↓1" in out


def test_preview_in_a_repository(tmpdir):
    with SimpleDirtyWithCommitRepo(tmpdir) as r:
        out = r.run(custom_config_content=ZSH_CONFIG, args=["preview"])
    assert out == "\x1b[38;5;4mmaster\x1b[39m\x1b[1m\x1b[38;5;1mΔ1\x1b[22m\x1b[39m"


def test_preview_without_colors(tmpdir):
    with SimpleDirtyWithCommitRepo(tmpdir) as r:
        out = r.run(custom_config_content=ZSH_CONFIG, args=["preview", "--no-color"])
    assert out == "masterΔ1"
