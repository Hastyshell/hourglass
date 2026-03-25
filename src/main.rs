mod args;
mod inline;
mod progress;
mod theme;
mod tui;

use std::io;

pub(crate) const FILLED: &str = "━";
pub(crate) const EMPTY:  &str = "─";
pub(crate) const HEAD:   &str = "╸";

fn main() -> io::Result<()> {
    let args = args::parse_args();
    if args.watch {
        tui::run_tui(&args.theme)
    } else {
        inline::print_inline(&args.theme)
    }
}
