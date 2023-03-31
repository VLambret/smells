use crate::analysis::public_interface::smells;

mod formatters;
mod metrics;
mod analysis;

fn main() {
    smells();
}
