use plandustry::block::build_registry;
use plandustry::data::renderer::Renderer;
use plandustry::data::schematic::SchematicSerializer;
use std::env::Args;

use crate::args::{self, OptionHandler};
use crate::print_err;

pub fn main(mut args: Args, arg_off: usize) {
    let mut handler = OptionHandler::default();
    if let Err(e) = args::parse(&mut args, &mut handler, arg_off) {
        print_err!(e, "Command error");
        return;
    }

    let reg = build_registry();
    let mut ss = SchematicSerializer(&reg);
    let mut first = true;
    let mut need_space = false;

    // process schematics from command line
    for curr in handler.get_literals() {
        match ss.deserialize_base64(curr) {
            Ok(s) => {
                if !first || need_space {
                    println!();
                }
                Renderer::render(&s).save("x.png").unwrap();
            }
            // continue processing literals & maybe interactive mode
            Err(e) => {
                if need_space {
                    println!();
                }
                first = false;
                need_space = false;
                print_err!(e, "Could not read schematic");
            }
        }
    }
}
