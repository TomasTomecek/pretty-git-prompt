#!/usr/bin/python3
"""
TODO: show colorful demo by default (enable changing config files via env var)
"""
import os
import sys
import shutil
import subprocess
import tempfile
import time

import pexpect
import py


d = tempfile.mkdtemp()


class Slacker:
    """ give a command time to start & finish """
    def __enter__(self):
        time.sleep(0.05)

    def __exit__(self, *args):
        time.sleep(0.1)


def init_repo(z):
    z.sendline("git init -q .")

def add_file(z, filename):
    z.sendline("git add {}".format(filename))

def commit(z):
    z.sendline("git commit -q -m mesáž")

def add_remote(z, name, path):
    z.sendline("git remote add {} {}".format(name, path))

def add_remote_origin(z, path):
    add_remote(z, "origin", path)

def push(z, remote, branch, with_tracking=True):
    z.sendline("git push -q {} {} {} >/dev/null".format("-u" if with_tracking else "", remote, branch))

def fetch(z):
    z.sendline("git fetch -a")

def reset_hard(z, ref):
    z.sendline("git reset -q --hard {} --".format(ref))

def checkout_ref(z, ref):
    z.sendline("git checkout {} --".format(ref))

def checkout_b(z, branch_name):
    z.sendline("git checkout -b {}".format(branch_name))

def create_file(filename, content):
    with open(filename, "w") as fd:
        fd.write(content + "\r\n")

def append_file(filename, content):
    with open(filename, "a") as fd:
        fd.write(content + "\r\n")


class G():
    """ wrapper on top of pexpect.spawn to create arbitrary git repositories """

    def __init__(self, tmpdir, shell_name):
        self.tmpdir = tmpdir
        self.repo = tmpdir.mkdir("repo")
        self.origin = tmpdir.mkdir("origin")
        subprocess.check_output(["git", "init", "--bare", str(self.origin.realpath())])
        self.upstream = tmpdir.mkdir("upstream")
        subprocess.check_output(["git", "init", "--bare", str(self.upstream.realpath())])
        subprocess.check_output(["git", "config", "--global", "user.email", "pretty-git-prompt@example.com"])
        subprocess.check_output(["git", "config", "--global", "user.name", "Git \"Pretty\" Prompter"])
        self.cwd = self.repo.chdir()
        self.z = pexpect.spawn("{} -i".format(shell_name), encoding=sys.getdefaultencoding())
        self.s = Slacker()

    def __enter__(self):
        self.do()  # first __init__, then __enter__
        return self

    def __exit__(self, *args):
        os.chdir(str(self.cwd))

    def do(self):
        """ run commands, wait to finish, prepapre for output """
        # we need to either give the shell time to process, or call exit and force EOF then
        # with Slacker():
        self.run()
        self.z.sendline("exit")

    def prepare(self):
        raise NotImplemented()

    def expect(self, p):
        """ get output from shell """
        return self.z.expect(p)


class BareRepo(G):
    def run(self):
        with self.s:
            init_repo(self.z)


class SimpleUntrackedFilesRepo(BareRepo):
    def run(self):
        super().run()
        create_file("file.txt", "text")


class SimpleChangedFilesRepo(SimpleUntrackedFilesRepo):
    def run(self):
        super().run()
        add_file(self.z, "file.txt")


class SimpleRepo(SimpleChangedFilesRepo):
    def run(self):
        super().run()
        with self.s:
            commit(self.z)


class SimpleDirtyWithCommitRepo(SimpleRepo):
    def run(self):
        super().run()
        create_file("file.txt", "text2")


class RepoWithOrigin(SimpleRepo):
    def run(self):
        super().run()
        with self.s:
            add_remote_origin(self.z, str(self.origin.realpath()))


class RWOWithoutTracking(RepoWithOrigin):
    def run(self):
        super().run()
        with self.s:
            push(self.z, "origin", "master", with_tracking=False)
        create_file("file.txt", "text3")
        with self.s:
            add_file(self.z, "file.txt")
        with self.s:
            commit(self.z)


class RWOLocalCommits(RepoWithOrigin):
    def run(self):
        super().run()
        with self.s:
            push(self.z, "origin", "master")
        create_file("file.txt", "text3")
        with self.s:
            add_file(self.z, "file.txt")
        with self.s:
            commit(self.z)


class RWORemoteCommits(RepoWithOrigin):
    def run(self):
        super().run()
        create_file("file.txt", "text4")
        with self.s:
            add_file(self.z, "file.txt")
        with self.s:
            commit(self.z)
        with self.s:
            push(self.z, "origin", "master")
        with self.s:
            reset_hard(self.z, "HEAD^")
        # fetch(self.z)


class RWODetached(RWOLocalCommits):
    def run(self):
        super().run()
        time.sleep(.5)
        self.co_commit = subprocess.check_output(["git", "rev-parse", "HEAD^"]).decode("utf-8").rstrip()
        checkout_ref(self.z, self.co_commit)


class MergeConflict(RWOLocalCommits):
    def run(self):
        super().run()
        checkout_b(self.z, "branch")
        reset_hard(self.z, "HEAD^")
        create_file("file.txt", "text5")
        add_file(self.z, "file.txt")
        commit(self.z)
        checkout_ref(self.z, "master")
        self.z.sendline("git merge --ff branch")
        self.z.interact()


class Demo(RWORemoteCommits):
    def run(self):
        super().run()
        with self.s:
            create_file("file.txt", "text5")
        with self.s:
            add_file(self.z, "file.txt")
        with self.s:
            commit(self.z)
        with self.s:
            add_remote(self.z, "upstream", str(self.upstream.realpath()))
        with self.s:
            push(self.z, "upstream", "master", with_tracking=False)
        with self.s:
            append_file("file.txt", "text6")
        with open("file.txt", "a") as f:
            f.write("text6")
        with self.s:
            add_file(self.z, "file.txt")
        with self.s:
            commit(self.z)
        with self.s:
            append_file("file.txt", "text7")
        with self.s:
            add_file(self.z, "file.txt")
        with self.s:
            append_file("file.txt", "text8")
        with self.s:
            create_file("file2.txt", "text7")
        self.z.sendline()
        self.z.interact()


def demo():
    d = tempfile.mkdtemp()
    l = py.path.local(d)

    try:
        shell_name = sys.argv[1]
    except IndexError:
        print("Usage:  ./demo.py <SHELL_NAME>")
        sys.exit(1)

    defult_config_path = os.path.expanduser("~/.config/pretty-git-prompt.yml")
    os.makedirs(os.path.dirname(defult_config_path), exist_ok=True)
    shutil.copy2("/app/files/pretty-git-prompt.yml.{}".format(shell_name), defult_config_path)

    try:
        with Demo(l, shell_name) as g:
            pass
    finally:
        shutil.rmtree(d)


if __name__ == "__main__":
    demo()
