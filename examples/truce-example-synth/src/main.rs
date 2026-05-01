use truce_example_synth::Plugin;

fn main() {
    truce_standalone::run::<Plugin>(truce_standalone::baked_defaults!());
}
