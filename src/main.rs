extern crate cursive;
extern crate cyberphage;
extern crate rand;

fn main() {
    let siv = &mut cursive::Cursive::default();
    let seed: u32 = rand::random();
    cyberphage::build_ui(siv, seed);
    siv.run();
}
