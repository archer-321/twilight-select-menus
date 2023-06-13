# About this repository

This repository demonstrates how to use the new Discord select menu API introduced by
[twilight-rs/twilight#2219](https://github.com/twilight-rs/twilight/pull/2219).

Most notably, the code in this repository is a _quick and dirty_ example and should not be used in production. Its sole
purpose is to provide an example that allows others to test the new API with a real Discord bot. The bot doesn't
implement any useful behaviour (outside testing) and doesn't use the new recommended slash commands.

## Required intents and permissions

This bot doesn't use the new slash commands system. Instead, it uses legacy text commands (specifically, `!select`).
Thus, you'll need to run this bot with an application that has access to the
[privileged `MESSAGE_CONTENT` intent](https://discord.com/developers/docs/topics/gateway#message-content-intent).

The bot user requires the following permissions to read and send messages in a specific channel:

- View Channel
- Send Messages

## Usage

The bot expects the parent directory to contain a clone of twilight. Specifically, it searches in
`../twilight/twilight-{gateway,http,model}` for the respective crates. The clone should be checked out at the most
recent commit of the Pull Request mentioned above. At the time of writing this README, that's commit
`e8fffb2b86bff8050104afce2ea71353d526e15b`.

If you have cloned your code in a different directory, simply change the `path` fields in `Cargo.toml` accordingly.

Next, run the bot with
```shell
DISCORD_TOKEN='YoUrDisCoRdToKeN' cargo run
```

You can then use `!select` in any channel the bot user can access.

## Licence

Copyright © 2023 Archer <archer@nefarious.dev>

This bot is permissively licensed under the terms and conditions of the Apache License, version 2.0. You can find the
full licence in [COPYING](COPYING) or online at <https://www.apache.org/licenses/LICENSE-2.0>.
