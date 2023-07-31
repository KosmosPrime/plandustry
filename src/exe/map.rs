use mindus::data::renderer::warmup;
use mindus::data::DataRead;
use mindus::{build_registry, Renderable};
use mindus::{MapSerializer, Serializer};
use std::env::Args;
use std::time::Instant;

use super::print_err;
pub fn main(args: Args) {
    let reg = build_registry();
    let mut ms = MapSerializer(&reg);

    // process schematics from command line
    println!("starting timing");
    let then = Instant::now();
    warmup();
    let warmup_took = then.elapsed();
    for curr in args {
        let Ok(s) = std::fs::read(curr) else {
            continue;
        };
        let starting_deser = Instant::now();
        match ms.deserialize(&mut DataRead::new(&s)) {
            Err(e) => print_err!(e, "fail"),
            Ok(m) => {
                let deser_took = starting_deser.elapsed();
                if let Ok(v) = std::env::var("SAVE") {
                    if v == "1" {
                        m.render().save("x.png").unwrap();
                        continue;
                    }
                }
                let starting_render = Instant::now();
                for _ in 0..10 {
                    m.render();
                }
                let renders_took = starting_render.elapsed();
                let took = then.elapsed();
                println!(
                    "μ total: {:.2}s (10 runs) (deser: {}ms, warmup: {}ms, render: {:.2}s) on map {}",
                    took.as_secs_f32() / 10.,
                    deser_took.as_millis(),
                    warmup_took.as_millis(),
                    renders_took.as_secs_f32() / 10.,
                    m.tags.get("mapname").unwrap(),
                );
            }
        }
    }
}
