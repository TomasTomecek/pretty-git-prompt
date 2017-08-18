import os
import shutil
import subprocess
import tempfile

def d():
    """ debug via running shell; need to run py.test with -s """
    subprocess.call(["zsh", "-i"])


def g(a):
    subprocess.check_call(["git"] + a)

def init_repo():
    g(["init", "."])

def add_file(filename):
    g(["add", "-v", filename])

def commit():
    g(["commit", "-m", "mesáž"])

def stash():
    g(["stash"])

def add_remote(name, path):
    g(["remote", "add", name, path])

def add_remote_origin(path):
    add_remote("origin", path)

def add_remote_upstream(path):
    add_remote("upstream", path)

def push(remote, branch, with_tracking=True):
    c = ["push"]
    if with_tracking:
        c.append("-u")
    c += [remote, branch]
    g(c)

def fetch():
    g(["fetch", "-a"])

def reset_hard(ref):
    g(["reset", "--hard", ref, "--"])

def checkout_ref(ref):
    g(["checkout", ref, "--"])

def checkout_b(branch_name):
    g(["checkout", "-b", branch_name])

def create_file(filename, content):
    with open(filename, "w") as fd:
        fd.write(content + "\r\n")

def append_file(filename, content):
    with open(filename, "a") as fd:
        fd.write(content + "\r\n")


class G():
    def __init__(self, tmpdir):
        self.tmpdir = tmpdir
        self.repo = tmpdir.mkdir("repo")
        self.origin = tmpdir.mkdir("origin")
        subprocess.check_output(["git", "init", "--bare", str(self.origin.realpath())])
        self.upstream = tmpdir.mkdir("upstream")
        subprocess.check_output(["git", "init", "--bare", str(self.upstream.realpath())])
        subprocess.check_output(["git", "config", "--global", "user.email", "pretty-git-prompt@example.com"])
        subprocess.check_output(["git", "config", "--global", "user.name", "Git \"Pretty\" Prompter"])
        self.cwd = self.repo.chdir()

    def __enter__(self):
        self.do()  # first __init__, then __enter__
        return self

    def __exit__(self, *args):
        os.chdir(str(self.cwd))

    def prepare(self):
        raise NotImplemented()

    def run(self, custom_config_content=None):
        """
        custom_config_content: string

        run program, return output
        """
        cmd = ["pretty-git-prompt"]
        if custom_config_content:
            tmpdir_path = os.path.join(str(self.tmpdir), "config")
            with open(tmpdir_path, "w") as fd:
                fd.write(custom_config_content)
            cmd += ["--config", tmpdir_path]
        return subprocess.check_output(cmd).decode("utf-8").rstrip()


class BareRepo(G):
    def do(self):
        init_repo()


class SimpleUntrackedFilesRepo(BareRepo):
    def do(self):
        super().do()
        create_file("file.txt", "text")


class SimpleChangedFilesRepo(SimpleUntrackedFilesRepo):
    def do(self):
        super().do()
        add_file("file.txt")


class SimpleRepo(SimpleChangedFilesRepo):
    def do(self):
        super().do()
        commit()


class SimpleRepoWithStashedContent(SimpleRepo):
    def do(self):
        super().do()
        create_file("file.txt", "stashed-content")
        stash()


class SimpleDirtyWithCommitRepo(SimpleRepo):
    def do(self):
        super().do()
        create_file("file.txt", "text2")


class RepoWithOrigin(SimpleRepo):
    def do(self):
        super().do()
        add_remote_origin(str(self.origin.realpath()))


class RWOAndUpstream(RepoWithOrigin):
    def do(self):
        super().do()
        add_remote_upstream(str(self.upstream.realpath()))
        push("origin", "master", with_tracking=True)
        create_file("file.txt", "text4")
        add_file("file.txt")
        commit()
        push("upstream", "master", with_tracking=False)
        reset_hard("HEAD^")
        create_file("file.txt", "text5")
        add_file("file.txt")
        commit()


class RWOWithoutTracking(RepoWithOrigin):
    def do(self):
        super().do()
        push("origin", "master", with_tracking=False)
        create_file("file.txt", "text3")
        add_file("file.txt")
        commit()


class RWOLocalCommits(RepoWithOrigin):
    def do(self):
        super().do()
        push("origin", "master")
        create_file("file.txt", "text3")
        add_file("file.txt")
        commit()


class RWORemoteCommits(RepoWithOrigin):
    def do(self):
        super().do()
        create_file("file.txt", "text4")
        add_file("file.txt")
        commit()
        push("origin", "master")
        reset_hard("HEAD^")


class RWODetached(RWOLocalCommits):
    def do(self):
        super().do()
        self.co_commit = subprocess.check_output(["git", "rev-parse", "HEAD^"]).decode("utf-8").rstrip()
        checkout_ref(self.co_commit)


class MergeConflict(RWOLocalCommits):
    def do(self):
        super().do()
        checkout_b("branch")
        reset_hard("HEAD^")
        create_file("file.txt", "text5")
        add_file("file.txt")
        commit()
        checkout_ref("master")
        subprocess.call(["git", "merge", "--ff", "branch"])


if __name__ == "__main__":
    # used in functional test
    d = tempfile.mkdtemp(dir=os.environ["HOME"])
    l = py.path.local(d)
    try:
        with MergeConflict(l) as g:
            pass
    finally:
        shutil.rmtree(d)
