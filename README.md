[![Build and Test](https://github.com/twitchax/augre/actions/workflows/build.yml/badge.svg)](https://github.com/twitchax/augre/actions/workflows/build.yml)
[![codecov](https://codecov.io/gh/twitchax/augre/branch/main/graph/badge.svg?token=35MZN0YFZF)](https://codecov.io/gh/twitchax/augre)
[![Version](https://img.shields.io/crates/v/augre.svg)](https://crates.io/crates/augre)
[![Crates.io](https://img.shields.io/crates/d/augre?label=crate)](https://crates.io/crates/augre)
[![GitHub all releases](https://img.shields.io/github/downloads/twitchax/augre/total?label=binary)](https://github.com/twitchax/augre/releases)
[![Rust](https://img.shields.io/badge/rust-nightly-blue.svg?maxAge=3600)](https://github.com/twitchax/augre)
[![License:MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

# augre

An LLM-powered (CodeLlama or OpenAI) local diff code review tool.
This particular fork is mainly focused on OpenAi, not a lot will work with Llama

## Binary Usage

### Dependencies :

You need cargo installed to compile the code: https://itsfoss.com/install-rust-cargo-ubuntu-linux/

Also, you need openssl:

Ubuntu: sudo apt install libssl-dev

Arch: sudo pacman -S openssl (not tested)

Windows: are you on windows ? haha use wsl2 :smile:

### Install

STEP 1: Clone the repo

STEP 2: run the script install.sh (it will ask for sudo password it is only to copy the files to /opt/) (feel free to check the script) 


### Help Docs

```bash
$ augre -h
Usage: augre [OPTIONS] [COMMAND]

Commands:
  review          Performs a code review from parent_branche(arg1) to child_branch(arg2)
  pr-description  Performs description for making PR from parent_branche(arg1) to child_branch(arg2)
  pr-and-review   Performs description for making PR from parent_branche(arg1) to child_branch(arg2) + reviews it
  commit-message  Gives you a comment for the last changes for your future commit
  ask             Gives a response to the specified prompt
  stop            Stop all of the background services
  help            Print this message or the help of the given subcommand(s)

Options:
  -d, --data-path <DATA_PATH>  The path to the data directory [default: .augre]
  -m, --mode <MODE>            The default operation mode [default: openai]
  -y, --yes                    Whether to skip the confirmation prompt
  -h, --help                   Print help
  -V, --version                Print version
```

For the commands review, pr-description, pr-and-review and commit-message, you can specify the version of chatgpt to use. By default it is gpt-3.5-turbo.

I highly recommand to use the command "augre review help" to help you (or any other command than "review")

## Example Config (if you use llama but remember this is not a fork that uses llama)

```toml
mode = "LocalGpu"
model_url = "https://huggingface.co/TheBloke/CodeLlama-13B-Instruct-GGML/resolve/main/codellama-13b-instruct.ggmlv3.Q3_K_M.bin"
cria_port = 3000
```

## Problems:

"An error (type: tokens) occurred on the API backend: Request too large for gpt-4 in organization org-WwKW6fn63kZPGicsCZIPrMJY on tokens per min (TPM): Limit Y, Requested X. The input or output tokens must be reduced in order to run successfully. Visit https://platform.openai.com/account/rate-limits to learn more."
In this case, too bad, no models exists yet with as much token as needed. Try a model with a bigger amount of token available

## License

MIT

I am the license
