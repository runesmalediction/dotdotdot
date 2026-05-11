<div align="center">
  <h1>[...]</h1>
  <h3> Manage your dotfiles.</h3>
</div>

<div align="center">
    <img alt="GitHub License" src="https://img.shields.io/github/license/runesmalediction/dotdotdot">
    <img alt="GitHub top language" src="https://img.shields.io/github/languages/top/runesmalediction/dotdotdot">
    <img alt="GitHub last commit" src="https://img.shields.io/github/last-commit/runesmalediction/dotdotdot">
    <img alt="Rust edition" src="https://img.shields.io/badge/rust%20edition-2024-orange">
    <img alt="GitHub release" src="https://img.shields.io/github/v/release/runesmalediction/dotdotdot">
</div>


## Installation

Download the latest binary from the [releases page](https://github.com/runesmalediction/dotdotdot/releases/latest) and place it somewhere on your `PATH`:

```sh
curl -L https://github.com/runesmalediction/dotdotdot/releases/latest/download/dotdotdot -o ~/.local/bin/dotdotdot
chmod +x ~/.local/bin/dotdotdot
```

Or build from source with Cargo:

```sh
cargo install --path .
```

## Usage

Run dotdotdot to apply your dotfiles:

```sh
dotdotdot
```

On each run it will:
1. Optionally commit, pull, and push the config directory if it is a git repository (requires `git = true`)
2. Warn about any untracked top-level entries in the config directory
3. Create symlinks for all entries marked as `linked`

dotdotdot reads its config from `~/.config/dotdotdot/config.toml`, which it creates on first run.
All files are linked, so no rendering of config files is done.
This is by designto keep dotdotdot simple and flexible.
Possible variable files can be kept in a `vars/` folder in the config directory.

## Features

- **Symlink management**: Automatically create and manage symlinks for your dotfiles
- **Git integration**: Optional automatic git commits, pulls, and pushes
- **Simple configuration**: TOML-based config file with clear syntax
- **Safety checks**: Warns about conflicting symlinks and untracked files
- **Fast-forward only**: Git operations use fast-forward only to prevent merge conflicts

## Config

```toml
git = true  # enable automatic git sync (default: false)

[[managed]]
path = ".zshrc"
manage = { linked = ".zshrc" }  # ~/.config/dotdotdot/.zshrc -> ~/.zshrc

[[managed]]
path = "nvim"
manage = { linked = ".config/nvim" }  # ~/.config/dotdotdot/nvim -> ~/.config/nvim

[[managed]]
path = "scripts"
manage = "none"  # present in the config dir but not linked anywhere
```

### Manage types

| Value | Behaviour |
|---|---|
| `"none"` | Tracked in the config directory, not linked |
| `{ linked = "<path>" }` | Symlinked to `~/<path>` |

If a symlink target already exists and is not managed by dotdotdot, a warning is shown asking you to back it up and remove it before re-running.

### Git sync

When `git = true` and `~/.config/dotdotdot/` is a git repository, dotdotdot will on each run:

- Commit any local changes with the message `auto: update dotfiles`
- Pull if behind the remote (fast-forward only)
- Push if ahead of the remote
- Warn if the repository has diverged and requires manual intervention

## License

Licensed under the [Apache License 2.0](LICENSE).
