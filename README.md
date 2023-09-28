# About this repository

This repository demonstrates how to use the new Discord select menu API introduced by
[twilight-rs/twilight#2219](https://github.com/twilight-rs/twilight/pull/2219) and, now that this PR is merged,
how the API introduced by [twilight-rs/twilight#2281](https://github.com/twilight-rs/twilight/pull/2281) is used.

Most notably, the code in this repository is a _quick and dirty_ example and should not be used in production. Its sole
purpose is to provide an example that allows others to test the new API with a real Discord bot. The bot doesn't
implement any useful behaviour (outside testing).

## Required intents and permissions

The bot user requires the following permissions to read and send messages in a specific channel:

- View Channel
- Send Messages

## Usage

The bot expects the parent directory to contain a clone of twilight. Specifically, it searches in
`../twilight/twilight-{gateway,http,model}` for the respective crates. The clone should be checked out at the most
recent commit of the Pull Request mentioned above. At the time of writing this README, that's commit
`c9a1a7ff4eda4149d49e80724171484a252c102a`.

If you have cloned your code in a different directory, simply change the `path` fields in `Cargo.toml` accordingly.

Next, run the bot with
```shell
DISCORD_TOKEN='YoUrDisCoRdToKeN' cargo run
```

You can then use `/select` in any channel the bot user can access.

## Licence

Copyright © 2023 Archer <archer@nefarious.dev>

This bot is permissively licensed under the terms and conditions of the Apache License, version 2.0. You can find the
full licence in [COPYING](COPYING) or online at <https://www.apache.org/licenses/LICENSE-2.0>.
