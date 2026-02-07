# sshpick 🔑

Simple CLI tool: pick an SSH host from your `~/.ssh/config`, then test the connection with `ssh -T`.

## Usage

```bash
$ sshpick

? Select SSH host:
> github-perso  → github.com  [~/.ssh/id_ed25519_perso]  (git)
  github-work   → github.com  [~/.ssh/id_ed25519_work]   (git)
  gitlab-client → gitlab.com  [~/.ssh/id_rsa_client]     (git)

🔑 ssh -T git@github-perso

Hi alice! You've successfully authenticated, but GitHub does not provide shell access.
```

## How it works

1. Parse `~/.ssh/config` to list all `Host` entries (skips wildcards)
2. Display an interactive selector showing host name, hostname, key file, and user
3. Run `ssh -T git@<selected-host>`

## Installation

```bash
cargo install --path .
```

## SSH config example

```
Host github-perso
    HostName github.com
    User git
    IdentityFile ~/.ssh/id_ed25519_perso

Host github-work
    HostName github.com
    User git
    IdentityFile ~/.ssh/id_ed25519_work
```

## License

MIT
