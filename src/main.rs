mod formatters;
mod metrics;
mod analysis;

use crate::analysis::smells;

fn main() {
    smells();
}
