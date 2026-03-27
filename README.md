# hourglass

Your token balance is always draining. So is everything else.

Most clocks tell you *what time it is*. This one tells you *how much is left* — of the hour, the day, the week, the month, the year. And if you want to go further, of your life.

Written during a full rebuild of a large codebase at work — the kind where you start the build, make a coffee, and still have time to vibe-code a side project. Somewhere in that waiting, it occurred to me that time doesn't feel like a clock anymore — it feels like a token quota. Allocated at birth, burning down with every second, no top-ups, no rollover.

![hourglass inline mode](https://github.com/user-attachments/assets/placeholder)

## Features

- **Inline mode** (default) — prints a bordered snapshot and exits, like a `top` entry but calmer
- **Watch mode** (`-w`) — full-screen live TUI that updates every second
- **Dark / light themes** with auto-detection via `$COLORFGBG`
- **Life indicator** — opt-in row showing how far through your expected lifespan you are

## Install

```sh
# from source
cargo install --path .
```

With Nix:

```sh
# run without installing
nix run github:Hastyshell/hourglass

# or add as a flake input to your NixOS / home-manager config
```

## Usage

```sh
hourglass                        # inline snapshot
hourglass -w                     # watch mode
hourglass --theme light          # force light theme
hourglass --birth 1990-06-15     # enable life progress indicator
```

Or via environment variables:

```sh
export HOURGLASS_BIRTH=1990-06-15
export HOURGLASS_LIFESPAN=85
hourglass
```

```
OPTIONS
  -w, --watch            live updating full-screen mode
      --theme THEME      color theme: dark | light | auto (default: auto)
      --birth YYYY-MM-DD birth date for life progress indicator
      --lifespan YEARS   expected lifespan in years (default: 80)
  -h, --help             show this help

ENVIRONMENT
  HOURGLASS_BIRTH        birth date (YYYY-MM-DD), enables life indicator
  HOURGLASS_LIFESPAN     expected lifespan in years (default: 80)
```

## Stack

- Rust 2024 edition
- [ratatui](https://github.com/ratatui-org/ratatui) for TUI rendering
- [crossterm](https://github.com/crossterm-rs/crossterm) for inline terminal output
- [chrono](https://github.com/chronotope/chrono) for time calculations

## License

MIT
