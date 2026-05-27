# asciitubing

A minimal Rust + OpenCV tool that streams your webcam directly into the terminal as full-screen colored ASCII art.

```
        .'`^":;Il!i><~+_-?][}{1)(|\/tfjrxnuvczXYUJCLQ0OZmwqpdbkhao*#MW&8%B@$
       .'`^":;Il!i><~+_-?][}{1)(|\/tfjrxnuvczXYUJCLQ0OZmwqpdbkhao*#MW&8%B@$
      .'`^":;Il!i><~+_-?][}{1)(|\/tfjrxnuvczXYUJCLQ0OZmwqpdbkhao*#MW&8%B@$
```

## What it does

- Captures webcam frames via V4L2 (`/dev/video0`)
- Resizes each frame to match your terminal dimensions
- Converts to grayscale and maps every pixel to an ASCII character using a 70-level brightness ramp
- Renders in real-time at ~30 FPS
- Press `c` to cycle through colors: **green → red → blue → yellow → orange → purple → cyan → white → rainbow**

## Dependencies

- [Rust](https://rustup.rs/)
- [OpenCV 4.x](https://opencv.org/) with development headers
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

Or after copying the binary:
```bash
./asciitubing
```

## Controls

| Key | Action |
|---|---|
| `c` | Cycle color mode |
| `q` | Quit |

## Color modes

| Mode | Description |
|---|---|
| `green` | Matrix-style terminal green |
| `red` | Crimson glow |
| `blue` | Deep electric blue |
| `yellow` | Warm amber |
| `orange` | Sunset orange |
| `purple` | Neon violet |
| `cyan` | Ice cyan |
| `white` | Clean monochrome |
| `rainbow` | Animated HSV-shifting per-pixel spectrum |

## License

MIT
