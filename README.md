# git-remote-codecommit

This is an implementation of [git-remote-codecommit](https://github.com/aws/git-remote-codecommit)
written in Rust so that statically linked binaries can be generated instead of depending
on Python.

## Install

```shell
$ cargo install --locked --profile=release-lto --git='https://github.com/demosdemon/git-remote-codecommit.git'
$ git-remote-codecommit --help
A Git remote helper for AWS CodeCommit.

This is normally invoked by git any time it needs to interact with a remote with the `codecommit://` scheme.

https://git-scm.com/docs/gitremote-helpers

Git invokes the helper with one or two arguments; however, this helper requires both arguments to be present. See the url above for more details; but briefly:

- The first argument is the name of the remote. In most cases, this is the name of the remote configured in the git repo. However, this can also be the URL to the remote if URL was encountered on the
command line.

- The second argument is the url of the remote. Git will not provide this if the remote is configured in the config as `remote.<name>.vcs = codecommit` and `remote.<name>.url` is not set. This is not
supported.

## URL format

This helper accepts the following URLs:

- `codecommit://[<profile>@]<repository>`: Use the default AWS region. Use the specified profile otherwise use the default.

- `codecommit::<region>://[<profile>@]<repository>`: Override the AWS region.

- Note: Git strips the `codecommit::` prefix when invoking the helper and the remote uses the region form.

Usage: git-remote-codecommit [OPTIONS] <REMOTE_NAME> <REMOTE_URI>

Arguments:
  <REMOTE_NAME>
          The first argument to the git-remote helper

  <REMOTE_URI>
          The second argument to the git-remote helper

Options:
      --code-commit-endpoint <URL>
          Override the default AWS endpoint for CodeCommit.

          If not provided, the default is `git-codecommit.${region}.${aws-partition}`.

          Where `${region}` is taken from the environment or profile and `${aws-partition}` is `amazonaws.com` for AWS regions and `amazonaws.cn` for AWS China regions.

          [env: CODE_COMMIT_ENDPOINT=]

  -h, --help
          Print help (see a summary with '-h')

  -V, --version
          Print version
```
