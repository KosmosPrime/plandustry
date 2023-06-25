use std::env::Args;

use mindus::build_registry;
use mindus::{Schematic, SchematicSerializer};

use crate::print_err;

pub fn main(args: Args) {
    let reg = build_registry();
    let mut ss = SchematicSerializer(&reg);
    let mut first = true;
    let mut need_space = false;
    // process schematics from command line
    for curr in args {
        match ss.deserialize_base64(&curr) {
            Ok(s) => {
                if !first || need_space {
                    println!();
                }
                first = false;
                need_space = true;
                println!("Schematic: {curr}");
                print_schematic(&s);
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

pub fn print_schematic(s: &Schematic) {
    if let Some(name) = s.tags.get("name") {
        if !name.is_empty() {
            println!("Name: {name}");
        }
    }
    if let Some(desc) = s.tags.get("description") {
        if !desc.is_empty() {
            println!("Desc: {desc}");
        }
    }
    if let Some(labels) = s.tags.get("labels") {
        if !labels.is_empty() && labels != "[]" {
            println!("Tags: {:?}", labels);
        }
    }
    let (cost, sandbox) = s.compute_total_cost();
    if !cost.is_empty() {
        println!(
            "Build cost: {cost}{}",
            if sandbox { " (Sandbox only)" } else { "" }
        );
    } else if sandbox {
        println!("Can only be built in the Sandbox");
    }
    println!("\n{s}");
}
