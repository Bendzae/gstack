![build](https://github.com/Bendzae/gstack/actions/workflows/rust.yml/badge.svg)

# Stacked branches and github PR's

> [!CAUTION]
> Heavy work in progress

G-stack is a CLI util to simplify creating stacked branches and pull requests on github.

## Installation

```bash
brew tap bendzae/gstack
```

and then

```bash
brew install gstack
```

## Configuration

To be able to create and modify github prs a personal access token is needed.
Check [the official github docs](https://docs.github.com/en/authentication/keeping-your-account-and-data-secure/managing-your-personal-access-tokens)
and make sure the token has read/write access to pull requests.
Then create a config file with the following content in `$HOME/.gstack/config.toml`

```toml
personal_access_token = "<GITHUB_PERSONAL_ACCESS_TOKEN>"
```

## Usage

### Show available commands

```bash
gs help
```

### Create a new stack

Creates a new stack with the current branch as a base and checks out the new branch

```bash
gs new
```

### Add a new stack branch

Stacks a new branch on top of the current stac

```bash
gs add
```

### Moving through stack branches

Move up and down trough stack branches with:

```bash
gs up
gs down
```

or interactively select a stack branch with

```bash
gs change
#or
gs c
```
