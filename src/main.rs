mod cli;
mod formatters;
mod metrics;

use cli::cli_lib;

fn main() {
    cli_lib::smells();
}
