# Flappy Rust

A small desktop Flappy Bird–style endless runner written in Rust with [Macroquad](https://github.com/not-fl3/macroquad).

## Run

From this directory:

```bash
cargo run --release
```

**Controls:** Space or left click to flap. Start from the title screen, retry after game over. Escape quits.

Run **`cargo run` from the project root** so paths like `assets/bg.png` resolve.

## Assets

Put images in **[`assets/`](assets/)** (not the repo root):

| File | Role |
|------|------|
| `assets/bg.png` | Full-screen background inside the play area |
| `assets/frame-1.png` … `assets/frame-8.png` | Bird animation (cycled while playing / on the title screen) |

If any of these fail to load, the game uses the built-in sky color and vector bird. If your bird faces the wrong way, set `flip_x` to `true` in the `draw_bird_sprite` call in `src/main.rs`.

## Save file

Your best score is stored as JSON on disk:

- **Linux:** `~/.local/share/flappy_rust/highscore.json`
- **macOS:** `~/Library/Application Support/flappy_rust/highscore.json`
- **Windows:** `%LOCALAPPDATA%\flappy_rust\highscore.json`

If you hit permission errors when running `cargo build` (for example on a shared machine), you can use a local Cargo cache:

```bash
mkdir -p .cargo_home
CARGO_HOME=$PWD/.cargo_home cargo build --release
```
