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
cargo b
```

The auto splitter is then available at:
```
target/wasm32-wasi/release/wasm_hollowknight_autosplit.wasm
```

Make sure too look into the [API documentation](https://livesplit.org/asr/asr/) for the `asr` crate.

You can use the [debugger](https://github.com/CryZe/asr-debugger) while
developing the auto splitter to more easily see the log messages, statistics,
dump memory and more.

## Instructions for livesplit-one-desktop

Clone `livesplit-one-desktop` from https://github.com/CryZe/livesplit-one-desktop

In the `livesplit-one-desktop` repository, modify the `config.yaml` file so that it contains
```yaml
general:
  splits: <path-to-splits.lss>
  auto-splitter: <path-to-wasm_hollowknight_autosplit.wasm>
```
where you replace `<path-to-splits.lss>` with the path to your splits file, and you replace `<path-to-wasm_hollowknight_autosplit.wasm>` with a path to the compiled `wasm` file found at `target/wasm32-wasi/release/wasm_hollowknight_autosplit.wasm` of this repository.

If you're running anything other than the specific placeholder splits in the `src/splits.json` file of this repository, you should modify that file to have the splits you want, in the order you want, and then re-compile this repository with
```sh
cargo b
```

When you run either `livesplit-one-desktop` or the `asr-debugger`, it needs to have permission to read memory of other processes.
On Mac, that might require running it under `sudo`.
For example in the `livesplit-one-desktop` repository, you can run
```sh
sudo cargo run --release
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
