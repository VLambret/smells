mod formatters;
mod metrics;
mod analysis;
mod data_sources;

use crate::analysis::smells;

fn main() {
    smells();
}
