import os
import shutil
import subprocess
import tempfile


class G():
    """ methods for creating git repositories """

    def __init__(self, tmpdir):
        self.tmpdir = tmpdir
        self.repo = tmpdir.mkdir("repo")
        self.origin = tmpdir.mkdir("origin")
        subprocess.check_output(["git", "init", "--bare", str(self.origin.realpath())])
        self.upstream = tmpdir.mkdir("upstream")
        subprocess.check_output(["git", "init", "--bare", str(self.upstream.realpath())])
        subprocess.check_output(["git", "config", "--global", "user.email", "pretty-git-prompt@example.com"])
        subprocess.check_output(["git", "config", "--global", "user.name", "Git \"Pretty\" Prompter"])

        self.cwd = None

    def __enter__(self):
        self.cwd = self.repo.chdir()
        self.do()  # first __init__, then __enter__
        return self

    def __exit__(self, *args):
        os.chdir(str(self.cwd))

    def prepare(self):
        raise NotImplemented()

    def run(self):
        """ run program, return output """
        binary_path = os.path.join(str(self.cwd), "target/debug/pretty-git-prompt")
        if not os.path.exists(binary_path):
            binary_path = os.path.join(str(self.cwd), "target/release/pretty-git-prompt")
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
        subprocess.check_call(["git", "remote", "add", "origin", str(self.origin.realpath())])


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
            f.write("text3")
        subprocess.check_call(["git", "add", "file.txt"])
        subprocess.check_call(["git", "commit", "-m", "msg2"])
        subprocess.check_call(["git", "push", "-u", "origin", "master"])
        subprocess.check_call(["git", "reset", "--hard", "HEAD^"])


class RWODetached(RWOLocalCommits):
    def do(self):
        super().do()
        self.co_commit = subprocess.check_output(["git", "rev-parse", "HEAD^"]).decode("utf-8").rstrip()
        subprocess.check_call(["git", "checkout", self.co_commit, "--"])


class MergeConflict(RWOLocalCommits):
    def do(self):
        super().do()

        subprocess.check_call(["git", "checkout", "-b", "branch"])
        subprocess.check_call(["git", "reset", "--hard", "HEAD^", "--"])
        with open("file.txt", "w") as f:
            f.write("text3")
        subprocess.check_call(["git", "add", "file.txt"])
        subprocess.check_call(["git", "commit", "-m", "this-will-conflict"])
        subprocess.check_call(["git", "checkout", "master"])
        subprocess.call(["git", "merge", "--ff", "branch"])


class Everything(RWORemoteCommits):
    def do(self):
        super().do()
        with open("file.txt", "a") as f:
            f.write("\ntext4")
        subprocess.check_call(["git", "add", "file.txt"])
        subprocess.check_call(["git", "commit", "-m", "msg4"])
        subprocess.check_call(["git", "remote", "add", "upstream", str(self.upstream.realpath())])
        subprocess.check_call(["git", "push", "upstream", "master"])
        with open("file.txt", "a") as f:
            f.write("\ntext5")
        subprocess.check_call(["git", "add", "file.txt"])
        subprocess.check_call(["git", "commit", "-m", "msg5"])
        with open("file.txt", "a") as f:
            f.write("\ntext6")
        subprocess.check_call(["git", "add", "file.txt"])
        with open("file.txt", "a") as f:
            f.write("\ntext7")
        with open("file2.txt", "w") as f:
            f.write("text")


if __name__ == "__main__":
    # used in functional test
    d = tempfile.mkdtemp(dir=os.environ["HOME"])
    import py
    l = py.path.local(d)
    try:
        with Everything(l) as g:
            subprocess.check_call(["zsh", "-i"])
    finally:
        shutil.rmtree(d)
