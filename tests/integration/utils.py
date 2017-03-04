import os
import subprocess

def init_repo():
    """ git init """

class G():
    """ methods for creating git repositories """

    def __init__(self, tmpdir):
        self.tmpdir = tmpdir
        self.repo = tmpdir.mkdir("repo")
        self.origin = tmpdir.mkdir("origin")
        subprocess.check_output(["git", "init", "--bare", self.origin.realpath()])
        self.upstream = tmpdir.mkdir("upstream")
        subprocess.check_output(["git", "init", "--bare", self.upstream.realpath()])
        self.cwd = None

    def __enter__(self):
        self.cwd = self.repo.chdir()
        self.do()  # first __init__, then __enter__
        return self

    def __exit__(self, *args):
        os.chdir(self.cwd)

    def prepare(self):
        raise NotImplemented()

    def run(self):
        """ run program, return output """
        binary_path = os.path.join(self.cwd, "target/release/git-stats")
        return subprocess.check_output([binary_path]).decode("utf-8").rstrip()


class BareRepo(G):
    def do(self):
        subprocess.check_call(["git", "init", "."])


class SimpleUntrackedFilesRepo(BareRepo):
    def do(self):
        super().do()
        with open("file.txt", "w+") as f:
            f.write("text")


class SimpleChangedFilesRepo(SimpleUntrackedFilesRepo):
    def do(self):
        super().do()
        subprocess.check_call(["git", "add", "file.txt"])


class SimpleRepo(SimpleChangedFilesRepo):
    def do(self):
        super().do()
        subprocess.check_call(["git", "commit", "-m", "msg"])


class SimpleDirtyWithCommitRepo(SimpleRepo):
    def do(self):
        super().do()
        with open("file.txt", "w+") as f:
            f.write("text2")


class RepoWithOrigin(SimpleRepo):
    def do(self):
        super().do()
        subprocess.check_call(["git", "remote", "add", "origin", self.origin.realpath()])


class RWOLocalCommits(RepoWithOrigin):
    def do(self):
        super().do()
        subprocess.check_call(["git", "push", "-u", "origin", "master"])
        with open("file.txt", "w") as f:
            f.write("text2")
        subprocess.check_call(["git", "add", "file.txt"])
        subprocess.check_call(["git", "commit", "-m", "msg2"])
        subprocess.check_call(["git", "fetch", "origin"])


class RWORemoteCommits(RepoWithOrigin):
    def do(self):
        super().do()
        with open("file.txt", "w") as f:
            f.write("text2")
        subprocess.check_call(["git", "add", "file.txt"])
        subprocess.check_call(["git", "commit", "-m", "msg2"])
        subprocess.check_call(["git", "push", "-u", "origin", "master"])
        subprocess.check_call(["git", "reset", "--hard", "HEAD^"])


class RWODetached(RWOLocalCommits):
    def do(self):
        super().do()
        self.co_commit = subprocess.check_output(["git", "rev-parse", "HEAD^"]).decode("utf-8").rstrip()
        subprocess.check_call(["git", "checkout", self.co_commit, "--"])


