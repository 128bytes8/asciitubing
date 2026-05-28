# asciitubing

A Rust + OpenCV tool that streams your webcam into the terminal as full-screen colored ASCII art with many color and ramp effects.

[![asciitubing demo](https://img.youtube.com/vi/i__ff5CbVk4/0.jpg)](https://www.youtube.com/watch?v=i__ff5CbVk4)

## What it does

- Captures webcam frames via V4L2 (`/dev/video0`)
- Resizes each frame to your terminal size (minus one status row)
- Maps brightness to ASCII with multiple ramp modes (normal, dense, sparse, inverted, block, edge, dither)
- 30+ color effects (matrix, heatmap, synthwave, truecolor from the camera, face tint, and more)
- Status bar shows **dark** / **light** pixel counts (threshold 0.5 luminance) and **average luminance**
- Real-time at ~30 FPS

## Dependencies

- [Rust](https://rustup.rs/)
- [OpenCV 4.x](https://opencv.org/) with development headers (and Haar cascades for face mode)
- Linux with a V4L2-compatible webcam

On Arch Linux:

```bash
sudo pacman -S opencv rust
```

## Build

```bash
cargo build --release
```

## Run

```bash
./target/release/asciitubing
```

## Controls

| Key | Action |
|---|---|
| `c` | Cycle color mode (all effects, one by one) |
| `b` | Cycle ASCII ramp mode |
| `m` | Toggle horizontal mirror |
| `p` | Pause / resume capture |
| `i` | Invert luminance before mapping |
| `s` | Save ASCII snapshot (`asciitubing_<unix>.txt`) |
| `+` / `=` | Increase checkerboard / stripe tile size |
| `-` | Decrease tile size |
| `1`–`9` | Jump to preset colors (green, red, blue, yellow, orange, purple, cyan, white, rainbow) |
| `q` | Quit |

## Status bar

The bottom line shows live stats, for example:

```text
color:matrix | dark:12450 light:8230 avg:0.42 | ramp:normal tile:100 | c:color b:ramp ...
```

- **dark** — pixels with luminance &lt; 0.5
- **light** — pixels with luminance ≥ 0.5
- **avg** — mean luminance (0.0–1.0)

## Color modes (`c` to cycle)

| Mode | Description |
|---|---|
| `green` … `white` | Solid terminal colors |
| `rainbow` | HSV spectrum shifted over time |
| `checkerboard` | Black/white tiles (size with `+`/`-`) |
| `matrix` | Green-on-black Matrix rain style |
| `heatmap` | Blue → red heat map by brightness |
| `grayscale` | Per-pixel gray from luminance |
| `neon` | Cyan / magenta / purple bands |
| `sepia` | Warm brown tones |
| `complement` | Inverted light/dark |
| `scanlines` | Horizontal band backgrounds |
| `vignette` | Darker toward edges |
| `fire` | Orange / red flame palette |
| `ocean` | Deep blue waves |
| `synthwave` | Pink / purple sunset |
| `duotone` | Two-color split by threshold |
| `pulse` | Brightness pulsing over time |
| `wave` | Horizontal color waves |
| `glitch` | Random RGB columns |
| `strobe` | Alternating invert flash |
| `aurora` | Green / purple northern lights |
| `stripes` | Vertical colored stripes |
| `dots` | Dot grid backgrounds |
| `radial` | Center spotlight |
| `noise` | Static grain colors |
| `truecolor` | Raw webcam BGR per pixel |
| `face` | Tint detected face region (Haar cascade) |

## Ramp modes (`b` to cycle)

| Mode | Description |
|---|---|
| `normal` | Full 70-character ramp |
| `dense` | Same full ramp |
| `sparse` | Short ` .:-=+*#%@` ramp |
| `inverted` | Flip brightness before lookup |
| `block4` / `block8` | Quantized brightness levels |
| `edge` | Sobel edge emphasis |
| `dither` | 4×4 Bayer ordered dither |

## License

MIT
