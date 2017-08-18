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
