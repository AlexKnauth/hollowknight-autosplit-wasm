# wasm-hollowknight-autosplit

An auto splitter for Hollow knight.

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
target/wasm32-wasi/release/wasm_hollowknight_autosplit.wasm
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

Right-click -> `Edit Layout...` and you should see a Layout
Editor with components like `Title`, `Splits`, `Timer`, etc.
If it does not have a component named `Auto Splitting Runtime`,
add one using the `+` Plus button -> `Control` -> `Auto Splitting Runtime`.
Once that's there, click `Layout Settings` -> `Auto Splitting Runtime`,
and next to `Script Path`, click `Browse...`,
then navigate to the compiled `wasm` file found at
`target/wasm32-wasi/release/wasm_hollowknight_autosplit.wasm`
of this repository.
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

Finally, do not manually split, skip, or undo splits while running with this autosplitter.
The autosplitter will not know that you did that, and the autosplitter's state will be out of sync with LiveSplit's state.

## Instructions for livesplit-one-desktop

Note: The main `livesplit-one-desktop` repository might not
be up-to-date enough to run this autosplitter.
- If https://github.com/CryZe/livesplit-one-desktop has a commit from December 2023 or later,
  then that's probably going to be up-to-date enough for this.
- However, if the most recent commit is still from July 2023,
  then you'll need to use a more up-to-date version,
  such as my fork https://github.com/AlexKnauth/livesplit-one-desktop,
  on branch `deps-3`.
- Or if another repository, such as https://github.com/CryZe/livesplit-one-druid,
  has a commit from December of 2023 or later, you may be able to try that out,
  but I don't have specific instructions for how to use that yet.

Clone `livesplit-one-desktop` from https://github.com/CryZe/livesplit-one-desktop,
or, if you want to clone my fork on branch `deps-3` directly, you can do so with
```sh
git clone -b deps-3 https://github.com/AlexKnauth/livesplit-one-desktop.git
```

In the `livesplit-one-desktop` repository, modify the `config.yaml` file so that it contains
```yaml
general:
  splits: <path-to-splits.lss>
  auto-splitter: <path-to-wasm_hollowknight_autosplit.wasm>
```
where you replace `<path-to-splits.lss>` with the path to your splits file, and you replace `<path-to-wasm_hollowknight_autosplit.wasm>` with a path to the compiled `wasm` file found at `target/wasm32-wasi/release/wasm_hollowknight_autosplit.wasm` of this repository.

When you run either `livesplit-one-desktop` or the `asr-debugger`, it needs to have permission to read memory of other processes.
On Mac, that might require running it under `sudo`.
For example in the `livesplit-one-desktop` repository, you can run
```sh
cargo build --release
sudo ./target/release/livesplit-one
```

Finally, do not manually split, skip, or undo splits while running with this autosplitter.
The autosplitter will not know that you did that, and the autosplitter's state will be out of sync with `livesplit-one-desktop`'s state.

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
