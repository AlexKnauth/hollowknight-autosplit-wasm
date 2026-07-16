# hollowknight-autosplit-wasm

An auto splitter for Hollow Knight that supports Windows, Mac, and Linux.

## Installation

Download the `hollowknight_autosplit_wasm_stable.wasm` file from the [Latest Release](https://github.com/AlexKnauth/hollowknight-autosplit-wasm/releases/latest).

Or follow the steps in [Compilation](#compilation) and use `target/wasm32-wasip1/release/hollowknight_autosplit_wasm.wasm`.

### LiveSplit (Windows)

The original LiveSplit is Windows-only. If you're on Mac or Linux, see other options below.

LiveSplit connects to this autosplitter and its settings
via a LiveSplit Layout (`.lsl`) file.
Make sure to use a different copy of your Layout for every
different splits file you run with this autosplitter.

Right-click -> `Edit Layout...` and you should see a Layout
Editor with components like `Title`, `Splits`, `Timer`, etc.
If it does not have a component named `Auto Splitting Runtime`,
add one using the `+` Plus button -> `Control` -> `Auto Splitting Runtime`.
Once that's there, click `Layout Settings` -> `Auto Splitting Runtime`,
and next to `Script Path`, click `Browse...`,
then navigate to the `hollowknight_autosplit_wasm_stable.wasm` file.
Then click `Import Splits` and select your splits file.

Click `Ok` and and save the layout with `Save Layout As...`,
with a name specific to the splits you're running with.

Deactivate the existing Hollow Knight autosplitter by Right-click -> `Edit Splits...`
then next to `Configurable Load Remover / Auto Splitter. (By DevilSquirrel)`,
click `Deactivate`.

Then add this autosplitter via the Layout file you saved earlier.
In the same Splits Editor from Right-click -> `Edit Splits...`,
below where `Configurable Load Remover / Auto Splitter. (By DevilSquirrel)` was,
check the `Use Layout` checkbox, click `Browse` next to that,
and navigate to the Layout file from before.
Select it and click `Ok`.

### LiveSplit One Druid (Windows, Linux, Mac)

Go to the [LiveSplit One Druid Latest Release](https://github.com/AlexKnauth/livesplit-one-druid/releases/latest) page,
and under the `Assets` section, download the one for your architecture and operating system.

When you run LiveSplitOne, it needs to have permission to read memory of other processes.
- Windows: no additional steps required.
- Linux: set the capabilities to include `CAP_SYS_PTRACE`, with a command like `sudo setcap CAP_SYS_PTRACE=+eip LiveSplitOne` to run once after downloading LiveSplitOne.
- Mac: you have to run it under `sudo`, with a command like `sudo ./LiveSplitOne` to run every time you want to open it.

Right-click or Control-click for the context menu:
- Splits, Open... : Select your `.lss` splits file. Go to [HKSplitMaker](https://hksplitmaker.com/?game=hollowknight) to generate and download `.lss` splits files.
- Open Auto-splitter... : Select the `hollowknight_autosplit_wasm_stable.wasm` file. Go to the [hollowknight-autosplit-wasm Latest Release](https://github.com/AlexKnauth/hollowknight-autosplit-wasm/releases/latest) to download that.
- Compare Against: Game Time.
- Hotkeys: Configure the hotkeys you want. The default hotkeys use numpad, so if your computer doesn't have a numpad, configure them differently.

### OBS LiveSplit One (Windows, Linux)

Go to the [OBS LiveSplit One Latest Release](https://github.com/AlexKnauth/obs-livesplit-one/releases/latest) page,
and under the `Assets` section, download the one for your architecture and operating system.
Follow the instructions in [How to install](https://github.com/AlexKnauth/obs-livesplit-one?tab=readme-ov-file#how-to-install):
- Windows: Extract the `obs-livesplit-one.dll` to `C:\Program Files\obs-studio\obs-plugins\64bit` or equivalent install directory.
- Linux: Ensure the plugins folder exists with `mkdir -p $HOME/.config/obs-studio/plugins`, then extract with a command like `tar -zxvf obs-livesplit-one-*-x86_64-unknown-linux-gnu.tar.gz -C $HOME/.config/obs-studio/plugins/`.

When you run OBS, it needs to have permission to read memory of other processes.
- Windows: no additional steps required.
- Linux: set the capabilities to include `CAP_SYS_PTRACE`, with a command like `sudo setcap CAP_SYS_PTRACE=+eip /usr/bin/obs` to run once after downloading OBS.

Add OBS Source Livesplit One.

Properties:
- Splits: Select your splits file. Go to [HKSplitMaker](https://hksplitmaker.com/?game=hollowknight) to generate and download `.lss` splits files.
- Use local autosplitter: Check
- Local Auto Splitter file: Select the `hollowknight_autosplit_wasm_stable.wasm` file. Go to the [hollowknight-autosplit-wasm Latest Release](https://github.com/AlexKnauth/hollowknight-autosplit-wasm/releases/latest) to download that.
- Custom auto splitter settings: Select `Import Splits`
- Select a file: Select your splits file

Open the OBS Settings from File, Settings:
- Go to the Hotkeys section and scroll down until you find LiveSplit One.
- Set a hotkey for `Toggle Timing Method`, and hit Ok.
- Hit that hotkey once to switch from the default, Real Time, to Game Time.

## Custom Variables: hits

If you have the Hit Counter setting turned on, you can show the number of hits with Edit Layout:
- Plus, Information, Text
- Layout settings, Text:
  - check the box for Custom Variable
  - Custom Variable Name: `hits`

You can also send hits to HitCounterManager via the [LiveSplit.HitCounterManagerConnector](https://github.com/topeterk/LiveSplit.HitCounterManagerConnector) component.

## Compilation

This auto splitter is written in Rust. In order to compile it, you need to
install the Rust compiler: [Install Rust](https://www.rust-lang.org/tools/install).

Afterwards install the WebAssembly target:
```sh
rustup target add wasm32-wasip1 --toolchain stable
```

The auto splitter can now be compiled:
```sh
cargo release
```

The auto splitter is then available at:
```
target/wasm32-wasip1/release/hollowknight_autosplit_wasm.wasm
```

Make sure too look into the [API documentation](https://livesplit.org/asr/asr/) for the `asr` crate.

## Development

You can use the [debugger](https://github.com/LiveSplit/asr-debugger) while
developing the auto splitter to more easily see the log messages, statistics,
dump memory, step through the code and more.

The repository comes with preconfigured Visual Studio Code tasks. During
development it is recommended to use the `Debug` launch action to run the
`asr-debugger`. You need to install the `CodeLLDB` extension to run it.

You can then use the `Build Auto Splitter (Debug)` task to manually build the
auto splitter. This will automatically hot reload the auto splitter in the
`asr-debugger`.

Alternatively you can install the [`cargo
watch`](https://github.com/watchexec/cargo-watch?tab=readme-ov-file#install)
subcommand and run the `Watch Auto Splitter` task for it to automatically build
when you save your changes.

The debugger is able to step through the code. You can set breakpoints in VSCode
and it should stop there when the breakpoint is hit. Inspecting variables
currently does not work all the time.

## Deploying a new release

My approach to deploying a new release looks like this:
1. Review Pull Requests, and merge those that are good and ready to the master branch.
2. Update `splits.json` with the command `make examples/splits.json`. If `make` says it's up-to-date and you know it isn't, `touch src/splits.rs` before running `make` again.
3. Update `Cargo.toml` with the new version number, following [Semantic Versioning](https://semver.org/). Given `MAJOR.MINOR.PATCH`:
   - Increment `PATCH` when just releasing bug-fixes that don't add any new settings or splits.
   - Increment `MINOR` when the release includes new features, new settings, or new splits.
   - Increment `MAJOR` if there've been incompatible settings changes, but like... try to avoid those if possible.
4. Run both `cargo b` and `cargo release`, each in both `--no-default-features` and default-features mode. Check that `Cargo.lock` has been updated.
5. Commit those changes, which should include `splits.json`, `Cargo.toml`, and `Cargo.lock`, with a commit name starting with `Release` and then the new version number.
6. Add a tag with the new version number on the Release commit, and push both master and the tag.
7. Check the CI to make sure all jobs pass. Sometimes there's a data race between jobs for `legacy` vs `stable` vs `unknown`, where 2 of them try to create their own release at the same time, so a release with only one of them is created as the other fails. When this happens, re-run the failed jobs to ensure that the release contains all 3 of the variants: `legacy`, `stable`, and `unknown`.
