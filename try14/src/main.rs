use my_cfg::Cfg;

use crate::optimizations::{remove_unneeded_assigns, remove_unneeded_locals};

mod optimizations;

fn main() {
    let mut cfg = Cfg::from_json(include_str!("../../../stable-mir-json/input.smir.json"));
    cfg.print();
    remove_unneeded_assigns(&mut cfg);
    remove_unneeded_locals(&mut cfg);
    cfg.print();
    dbg!(cfg.loopify());
}
