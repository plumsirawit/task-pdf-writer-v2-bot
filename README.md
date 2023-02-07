# task-pdf-writer-v2-bot

Discord chatbot for `task-pdf-writer-v2`.

Note that this is just an attempt. Not sure if it works well but let's try.

## Usage

`cargo install cargo-shuttle` to install `shuttle`, then `cargo shuttle deploy`.

The current version uses [shuttle](https://www.shuttle.rs/) as a base runner, on [serenity](https://github.com/serenity-rs/serenity). Full dependencies are listed in `Cargo.toml`.

## Bot Usage

First, set up a `task-pdf-writer-v2`-compatible directory, then tell this information to the bot by using the slash command `/config`. The first argument should be the git repository (it may contain other stuffs, don't worry). The second argument is the relative path from the root directory to the contest directory. And the third argument is the private key for private repositories (leave blank for public ones).

## Setting up the key(s)

To allow the bot to access your private repository, an SSH key must be generated first, and the public key must be added to the repository beforehand. If you're using `ssh-keygen`, basically, most probably, you'll need to `ssh-keygen -t ed25519` into some directory (IMPORTANT: don't use your `~/.ssh` default directory since this key will only be used for the bot connection, and also don't utilize this key for other uses). The instructions go as follows:

1. Generate the key pair (Ed25519 recommended since GitHub doesn't allow SHA-1 anymore).
2. Add the public key to the repository. For GitHub, open the repository webpage and select `Settings`, then go to `Security > Deploy keys`, then `Add deploy key` and paste the public key into the textarea.
3. Add the private key to the bot. Use `/config` as stated before.

Note: If you use `/config` one time and want to use it again, you should give all the data again: all the 2 arguments + (1 optional), all at once. (You cannot just replace the private key without giving the first two arguments, etc.)

## PDF Generation

In Discord, setup a dedicated channel for the bot with the name exactly `task-pdf-writer-v2-bot`. Under that channel, create threads, each thread must have its name exactly the same as the problem name. After that, call `/genpdf` inside the thread. It should give you the requested PDF.

## BUG!?

In case of bugs, please report them (maybe in the issues here or in direct message to me). Note that IT IS EXPECTED to have bugs. It is normal, since I haven't tested it rigorously enough.

## Contributions

Contributions are more than welcomed. Feel free to open pull requests, but note that they will be reviewed by [me (@plumsirawit)](https://github.com/plumsirawit) first. The contributions should be aligned with the direction of the current working draft (changing languages or frameworks shouldn't be the case). In case of suggestion for changing the direction of the implementation, please contact me first.
