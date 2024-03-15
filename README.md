# hollowknight-autosplit-wasm

An auto splitter for Hollow Knight that supports Windows, Mac, and Linux.

## Installation

Download the `hollowknight_autosplit_wasm.wasm` file from the [Latest Release](https://github.com/AlexKnauth/hollowknight-autosplit-wasm/releases/latest).

Or follow the steps in [Compilation](#compilation) and use `target/wasm32-wasi/release/hollowknight_autosplit_wasm.wasm`.

To configure LiveSplit or a LiveSplit One prototype to use this, see:
 - [Instructions for LiveSplit Windows](#instructions-for-livesplit-windows)
 - [Instructions for obs-livesplit-one](#instructions-for-obs-livesplit-one)
 - [Instructions for livesplit-one-druid](#instructions-for-livesplit-one-druid)
 - [Instructions for livesplit-one-desktop](#instructions-for-livesplit-one-desktop)

## Compilation

This auto splitter is written in Rust. In order to compile it, you need to
install the Rust compiler: [Install Rust](https://www.rust-lang.org/tools/install).

Afterwards install the WebAssembly target:
```sh
rustup target add wasm32-wasi --toolchain stable
```

The auto splitter can now be compiled:
```sh
cargo release
```

The auto splitter is then available at:
```
target/wasm32-wasi/release/hollowknight_autosplit_wasm.wasm
```

Make sure too look into the [API documentation](https://livesplit.org/asr/asr/) for the `asr` crate.

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

## Instructions for LiveSplit Windows

LiveSplit connects to this autosplitter and its settings
via a LiveSplit Layout (`.lsl`) file.
Make sure to use a different copy of your Layout for every
different splits file you run with this autosplitter.

For hit-counter mode, best to use a layout that formats
times and deltas without the decimals, such as the
[splits/hits/layout-direct.lsl](splits/hits/layout-direct.lsl)
file included in this repository.

Right-click -> `Edit Layout...` and you should see a Layout
Editor with components like `Title`, `Splits`, `Timer`, etc.
If it does not have a component named `Auto Splitting Runtime`,
add one using the `+` Plus button -> `Control` -> `Auto Splitting Runtime`.
Once that's there, click `Layout Settings` -> `Auto Splitting Runtime`,
and next to `Script Path`, click `Browse...`,
then navigate to the `hollowknight_autosplit_wasm.wasm` file.
Then click `Import Splits` and select your splits file.

For hit-counter mode, change the `Timing Method` setting to
either `Hits / dream falls` or `Hits / damage`.

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

Finally, do not manually split or skip while running with this autosplitter,
unless either it's explicitly marked as `ManualSplit` or it's the end-split.
Don't manually split, skip, or undo splits in any other situation.
The autosplitter will not know that you did that, and the autosplitter's state will be out of sync with LiveSplit's state.

## Instructions for obs-livesplit-one

Make sure to use `obs-livesplit-one` release v0.3.5 or later.

Go to the [obs-livesplit-one releases](https://github.com/LiveSplit/obs-livesplit-one/releases) page,
and under the `Assets` section, download the one that matches your architecture and operating system.
Follow the instructions in the
[`How to install`](https://github.com/LiveSplit/obs-livesplit-one/blob/master/README.md#how-to-install)
section of the `obs-livesplit-one` README file.
On Windows, extract the `obs-livesplit-one.dll` to either
`C:\Program Files\obs-studio\obs-plugins\64bit` or
`C:\Program Files (x86)\obs-studio\obs-plugins\64bit`.

For hit-counter mode, modify your splits file,
near the end after `</Splits>` but before `</AutoSplitterSettings>`,
so that it contains `<TimingMethod>HitsDreamFalls</TimingMethod>`
or `<TimingMethod>HitsDamage</TimingMethod>`.

Add OBS Source Livesplit One.

Properties:
- Splits: Select your splits file
- Use local autosplitter: Check
- Local Auto Splitter file: Select the `hollowknight_autosplit_wasm.wasm` file
- Custom auto splitter settings: Select `Import Splits`
- Select a file: Select your splits file

## Instructions for livesplit-one-druid

Note: The main `livesplit-one-druid` repository might not
be up-to-date enough to run this autosplitter.
- If https://github.com/CryZe/livesplit-one-druid has a commit from December 2023 or later,
  then that's probably going to be up-to-date enough for this.
- However, if the most recent commit is still from April 2023,
  then you'll need to use a more up-to-date version,
  such as my fork https://github.com/AlexKnauth/livesplit-one-druid.

You can clone my fork with
```sh
git clone https://github.com/AlexKnauth/livesplit-one-druid.git
```

Create a config file if it's not there already.

On Windows, the config file should be at
```
C:\Users\<name>\AppData\Local\LiveSplit\LiveSplit One\data\config.yml
```
When running `livesplit-one-druid` for the first time,
you should be able to get it to initialize the config file itself,
possibly after changing one of the settings, opening a file, etc.

On Mac, the config file should be at
```
/Users/<name>/Library/Application Support/org.LiveSplit.LiveSplit-One/config.yml
```
In my experience, you have to create a file here yourself,
running `livesplit-one-druid` doesn't initialize it on Mac.

On Linux, the config file should be at
```
/home/<name>/.local/share/livesplitone/config.yml
```

Modify the `config.yml` file so that it contains
```yaml
splits:
  current: <path-to-splits.lss>
general:
  timing-method: GameTime
  auto-splitter: <path-to-hollowknight_autosplit_wasm.wasm>
```
where you replace `<path-to-splits.lss>` with the path to your splits file,
and you replace `<path-to-hollowknight_autosplit_wasm.wasm>`
with a path to the `hollowknight_autosplit_wasm.wasm` file.

To configure it with a layout file, modify the `config.yml` file so that it contains
```yaml
general:
  layout: <path-to-layout.ls1l>
```
where you replace `<path-to-layout.ls1l>` with a path to your layout file.
I recommend you use `.ls1l` layout file for `livesplit-one-druid`, not a `.lsl` layout file.
You can make a `.ls1l` file in the LiveSplit One Web version at https://one.livesplit.org/,
or you can use the `layout-web.ls1l` file included in this repository as a starting point.

To configure hotkeys, modify the `config.yml` file so that it contains
```yaml
hotkeys:
  split: Numpad1
  reset: Numpad3
  undo: Numpad8
  skip: Numpad2
  pause: null
  undo_all_pauses: null
  previous_comparison: Numpad4
  next_comparison: Numpad6
  toggle_timing_method: null
```
Where you can replace those hotkey values with variants from
[`livesplit_hotkey::KeyCode`](https://docs.rs/livesplit-hotkey/latest/livesplit_hotkey/enum.KeyCode.html).

When you run `livesplit-one-druid`,
it needs to have permission to read memory of other processes.
On Mac, that might require running it under `sudo`.
For example in the `livesplit-one-druid` repository, you can run
```sh
cargo build --release
sudo ./target/release/livesplit-one
```

For hit-counter mode, modify your splits file,
near the end after `</Splits>` but before `</AutoSplitterSettings>`,
so that it contains `<TimingMethod>HitsDreamFalls</TimingMethod>`
or `<TimingMethod>HitsDamage</TimingMethod>`.

Finally, do not manually split or skip while running with this autosplitter,
unless either it's explicitly marked as `ManualSplit` or it's the end-split.
Don't manually split, skip, or undo splits in any other situation.
The autosplitter will not know that you did that,
and the autosplitter's state will be out of sync with `livesplit-one-druid`'s state.

## Instructions for livesplit-one-desktop

Note: The main `livesplit-one-desktop` repository might not
be up-to-date enough to run this autosplitter.
- If https://github.com/CryZe/livesplit-one-desktop has a commit from December 2023 or later,
  then that's probably going to be up-to-date enough for this.
- However, if the most recent commit is still from July 2023,
  then you'll need to use a more up-to-date version,
  such as my fork https://github.com/AlexKnauth/livesplit-one-desktop.

You can clone my fork with
```sh
git clone https://github.com/AlexKnauth/livesplit-one-desktop.git
```

According to the [`rust_minifb` Build Instructions](https://github.com/emoon/rust_minifb#build-instructions),
on Linux you may need to install these dependencies first:
```sh
sudo apt install libxkbcommon-dev libwayland-cursor0 libwayland-dev
```

In the `livesplit-one-desktop` repository, modify the `config.yaml` file so that it contains
```yaml
general:
  splits: <path-to-splits.lss>
  timing-method: GameTime
  auto-splitter: <path-to-hollowknight_autosplit_wasm.wasm>
```
where you replace `<path-to-splits.lss>` with the path to your splits file,
and you replace `<path-to-hollowknight_autosplit_wasm.wasm>`
with a path to the `hollowknight_autosplit_wasm.wasm` file.

To configure it with a layout file, modify the `config.yaml` file so that it contains
```yaml
general:
  layout: <path-to-layout.lsl>
```
where you replace `<path-to-layout.lsl>` with a path to your layout file.
You can use either a `.lsl` or a `.ls1l` layout with `livesplit-one-desktop`.

To configure hotkeys, modify the `config.yaml` file so that it contains
```yaml
hotkeys:
  split: Numpad1
  reset: Numpad3
  undo: Numpad8
  skip: Numpad2
  pause: null
  undo_all_pauses: null
  previous_comparison: Numpad4
  next_comparison: Numpad6
  toggle_timing_method: null
```
Where you can replace those hotkey values with variants from
[`livesplit_hotkey::KeyCode`](https://docs.rs/livesplit-hotkey/latest/livesplit_hotkey/enum.KeyCode.html).

When you run `livesplit-one-desktop`,
it needs to have permission to read memory of other processes.
On Mac, that might require running it under `sudo`.
For example in the `livesplit-one-desktop` repository, you can run
```sh
cargo build --release
sudo ./target/release/livesplit-one
```

For hit-counter mode, modify your splits file,
near the end after `</Splits>` but before `</AutoSplitterSettings>`,
so that it contains `<TimingMethod>HitsDreamFalls</TimingMethod>`
or `<TimingMethod>HitsDamage</TimingMethod>`.

Finally, do not manually split or skip while running with this autosplitter,
unless either it's explicitly marked as `ManualSplit` or it's the end-split.
Don't manually split, skip, or undo splits in any other situation.
The autosplitter will not know that you did that,
and the autosplitter's state will be out of sync with `livesplit-one-desktop`'s state.

The keyboard shortcuts of `livesplit-one-desktop` assume the Qwerty keyboard layout,
so you may need to press where the key would be if you were using Qwerty.
For example to save splits is "Control S" on Qwerty, but on a Dvorak keyboard,
the Qwerty key "S" is where the Dvorak key "O" is, so use Dvorak "Control O" to save instead.

How to keep it on top over a Full Screen game on Mac:
Start dragging the livesplit-one-desktop window,
and while dragging it, do a multi-finger swipe motion
to switch screens to the Full Screen game,
and then stop dragging.
It will stay on top of the Full Screen game temporarily,
as long as you don't do any more screen-switching after
that.
