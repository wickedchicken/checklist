# checklist
a checklist app to run through processes

[![Build Status](https://travis-ci.org/wickedchicken/checklist.svg?branch=master)](https://travis-ci.org/wickedchicken/checklist)
[![Dependabot Status](https://api.dependabot.com/badges/status?host=github&repo=wickedchicken/checklist)](https://dependabot.com)

This is an application designed to help you go through repeatable processes. `checklist`
will run through whatever `.checklist.yml` is in the current directory. Right now, there
only one checklist used and that is called 'committing.'

```yaml
---
schema_version: 2
committing:
  automated:
    - git log origin/master..HEAD --oneline | grep -v WIP  # abort if WIP comment is seen
    - cargo test
  manual:
    - checked that the version number has been bumped
    - checked that the schema version number has been bumped
```

This will run `git log [...]` and `cargo test` automatically, exiting if they fail. It
will then ask you if you're performed the other two tasks. Note that the automated tasks
are passed to the shell using [duct_sh](https://docs.rs/duct_sh/0.1.0/duct_sh/), meaning
they can use piping and anything else your shell (either `/bin/sh` or `cmd.exe`) supports.
Of particular note: in the example above, the `#` is interpreted as a comment by YAML
itself, stripped before the command is passed to the shell.

The spirit of `checklist` is that some checks are too crude to be on CI, and other checks
require a human brain to perform. We should still document and run these, not letting the
perfect be the enemy of the good.

It is recommended to put `.checklist.yml` in your `.gitignore` or `.git/info/exclude`, and
treat them as personal and not repository files. You may want to use a program like
[gnu stow](https://www.gnu.org/software/stow/) to keep these files in source control, but
in their own repository. There are
[several](http://brandon.invergo.net/news/2012-05-26-using-gnu-stow-to-manage-your-dotfiles.html)
[tutorials](https://alexpearce.me/2016/02/managing-dotfiles-with-stow/) on how to do this.

The advantage of keeping `.checklist` files to yourself is you can put whatever hacky
scripts or hints to yourself that you need. It should live inside an ecosystem of
code review and proper CI, not take the place of one or the other.
