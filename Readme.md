# Stacked branches and github PR's

> [!CAUTION]
> Heavy work in progress

G-stack is a CLI util to simplify creating stacked branches and pull requests on github.

## Installation

`brew tap bendzae/gstack` and then `brew install gstack`

## Configuration

To be able to create and modify github prs a personal access token is needed.
To specify it create a config file with the following content in `$HOME/.gstack/config.toml`

```toml
personal_access_token = "<GITHUB_PERSONAL_ACCESS_TOKEN>"

```

## Usage

_Show available commands_
`gs help`
