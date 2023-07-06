use mindus::data::DataRead;
use mindus::{build_registry, Renderer};
use mindus::{MapSerializer, Serializer};
use std::env::Args;

use super::print_err;
pub fn main(args: Args) {
    let reg = build_registry();
    let mut ms = MapSerializer(&reg);

    // process schematics from command line
    for curr in args {
        if let Ok(s) = std::fs::read(curr) {
            match ms.deserialize(&mut DataRead::new(&s)) {
                Err(e) => print_err!(e, "fail"),
                Ok(m) => {
                    Renderer::render_map(&m).save("x.png").unwrap();
                }
            }
        }
    }
}
