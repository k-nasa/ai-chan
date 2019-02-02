# ai-chan

[![CircleCI](https://circleci.com/gh/k-nasa/ai-chan.svg?style=svg)](https://circleci.com/gh/k-nasa/ai-chan)

A helpful operation bot for GitHub. It helps assign reviewers and merge pull requests.

## What is this?
I made it inspired by [popuko](https://github.com/voyagegroup/popuko)

This is an operation bot to do these things automatically for your project on GitHub.

- merge pull request
- assign a pull request to  reviewers

## Command

### ```r? @<reviewer>```

- You can call r? @<reviewer1> @<reviewer2> to assign multiple reviewers
- All user can call this command.

### ```@<botname> r+```

- You can use this to merge pull
- Now anyone can call (It is due to be fixed)

## Setup
### Build and Launch application
0. This requires [cargo](https://github.com/rust-lang/cargo).
1. Build from source
  - cargo build

2. Create config file
  - Read the config file of this path (~/.config/ai-chan/config.toml)
  - Let's cofy from [config.toml](https://github.com/k-nasa/ai-chan/blob/master/example.config.toml)

3. Start the exec binary
 - Please run the binary made by cargo build
 - Or run ```cargo run```
4. Done!

### Setup for your repository in GitHub
1. Set the account which this app uses as a collaborator for your repository.
2. Please set webhook url
  - The entry point of the request is ```http://<your_server_with_port>/github```
  - Although it is OK to send all event types, we recommend enabling only the following items
    - IssueComment
    - PullRequest
3. Done!
