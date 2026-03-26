# Flappy Rust

A small desktop Flappy Bird–style endless runner written in Rust with [Macroquad](https://github.com/not-fl3/macroquad).

<img width="617" height="840" alt="Screenshot_2026-03-26_16-43-24" src="https://github.com/user-attachments/assets/3d229549-2e70-4ea8-a97a-1a7beed8bb4c" />

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

## Web (WASM) and GitHub Pages

The workflow [.github/workflows/pages.yml](.github/workflows/pages.yml) builds **`wasm32-unknown-unknown`**, copies [`web/index.html`](web/index.html), `flappy_rust.wasm`, and **`assets/`** into a static site, and deploys it with **GitHub Actions → Pages**.

**One-time repo setup**

1. On GitHub: **Settings → Pages → Build and deployment → Source:** choose **GitHub Actions** (not “Deploy from a branch”).
2. Push to **`main`**; check **Actions** for the “Deploy Pages” run. The site URL will be  
   `https://<user>.github.io/<repo>/`  
   (for example `https://curious-keeper.github.io/flappy-rust/`).

**High score in the browser** uses **`localStorage`** via [`web/storage_plugin.js`](web/storage_plugin.js) and Miniquad’s `miniquad_add_plugin` API. Do **not** use `web-sys` / **wasm-bindgen** for saves: Macroquad’s loader (`mq_js_bundle.js`) is not compatible with wasm-bindgen imports (black screen / instantiate errors).

**Try WASM locally** (static server; same folder must include `storage_plugin.js`):

```bash
rustup target add wasm32-unknown-unknown
cargo build --release --target wasm32-unknown-unknown

mkdir -p site
cp target/wasm32-unknown-unknown/release/flappy_rust.wasm site/
cp web/index.html site/
cp web/storage_plugin.js site/
cp -r assets site/
cd site && python -m http.server 8080
```

Open `http://localhost:8080/`. See also [Macroquad WASM notes](https://github.com/not-fl3/macroquad#wasm).
