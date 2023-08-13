use super::renderer::*;
use super::GridPos;
use crate::block::{Block, Rotation};
use bobbin_bits::U4;

#[cfg(test)]
macro_rules! dir {
    (^) => {
        crate::block::Rotation::Up
    };
    (v) => {
        crate::block::Rotation::Down
    };
    (<) => {
        crate::block::Rotation::Left
    };
    (>) => {
        crate::block::Rotation::Right
    };
}
#[cfg(test)]
macro_rules! conv {
    (_) => {
        None
    };
    ($dir:tt) => {
        Some((
            &crate::block::distribution::CONVEYOR,
            crate::data::autotile::dir!($dir),
        ))
    };
}
#[cfg(test)]
macro_rules! define {
    ($a:tt,$b:tt,$c:tt,$d:tt) => {
        [
            crate::data::autotile::conv!($a),
            crate::data::autotile::conv!($b),
            crate::data::autotile::conv!($c),
            crate::data::autotile::conv!($d),
        ]
    };
}

#[cfg(test)]
pub(crate) use conv;
#[cfg(test)]
pub(crate) use define;
#[cfg(test)]
pub(crate) use dir;

pub type Cross<'l> = [Option<(&'l Block, Rotation)>; 4];
/// holds the 4 bordering blocks
#[derive(Copy, Clone)]
pub struct RenderingContext<'l> {
    pub cross: Cross<'l>,
    pub position: PositionContext,
}

/// holds positions
#[derive(Copy, Clone, Eq, PartialEq)]
pub struct PositionContext {
    pub position: GridPos,
    pub width: usize,
    pub height: usize,
}

impl std::fmt::Debug for PositionContext {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "PC<{:?} ({}/{})>",
            self.position, self.width, self.height
        )
    }
}

#[cfg(test)]
fn print_crosses(v: Vec<Cross<'_>>, height: usize) -> String {
    let mut s = String::new();
    for c in v.chunks(height) {
        for c in c {
            s.push(c[0].map_or('_', |(_, r)| r.ch()));
            for c in &c[1..] {
                s.push(',');
                s.push(c.map_or('_', |(_, r)| r.ch()));
            }
            s.push(' ');
        }
        s.push('\n');
    }
    s
}

pub fn tile(ctx: &RenderingContext<'_>, name: &str, rot: Rotation, s: Scale) -> ImageHolder<4> {
    rotations2tile(mask2rotations(mask(ctx, rot, name), rot), name, s)
}

pub fn mask2rotations(mask: U4, rot: Rotation) -> (u8, u8, u8) {
    use U4::{
        B0000, B0001, B0010, B0011, B0100, B0101, B0110, B0111, B1000, B1001, B1010, B1011, B1100,
        B1101, B1110, B1111,
    };
    macro_rules! p {
        ($image:literal, $rotation:literal) => {
            ($image, $rotation, 0)
        };
        ($image:literal, $rotation:literal, $flipping:expr) => {
            ($image, $rotation, $flipping)
        };
    }

    match mask {
        // from left
        B0001 => match rot {
            Rotation::Down => p!(1, 1, FLIP_Y), // ┐
            Rotation::Right => p!(0, 0),        // ─
            Rotation::Up => p!(1, 3),           // ┘
            Rotation::Left => unreachable!(),
        },
        // from below
        B0010 => match rot {
            Rotation::Left => p!(1, 2),  // ┐
            Rotation::Right => p!(1, 1), // ┌
            Rotation::Up => p!(0, 3),    // │
            Rotation::Down => unreachable!(),
        },
        // from bottom + left
        B0011 => match rot {
            Rotation::Right => p!(2, 0),               // ┬
            Rotation::Up => p!(2, 3, FLIP_Y | FLIP_X), // ┤
            _ => unreachable!(),
        },
        // from right
        B0100 => match rot {
            Rotation::Left => p!(0, 2),       // ─
            Rotation::Down => p!(1, 1),       // ┌
            Rotation::Up => p!(1, 1, FLIP_X), // └
            Rotation::Right => unreachable!(),
        },
        // from sides
        B0101 => match rot {
            Rotation::Up => p!(4, 3),   // ┴
            Rotation::Down => p!(4, 1), // ┬
            _ => unreachable!(),
        },
        // from right + down
        B0110 => match rot {
            Rotation::Up => p!(2, 3),           // ├,
            Rotation::Left => p!(2, 0, FLIP_X), // ┬
            _ => unreachable!(),
        },
        // from right + down + left
        B0111 => match rot {
            Rotation::Up => p!(3, 3), // ┼
            _ => unreachable!(),
        },
        // from above
        B1000 => match rot {
            Rotation::Down => p!(0, 1),         // │
            Rotation::Left => p!(1, 0, FLIP_X), // ┘
            Rotation::Right => p!(1, 0),        // └
            Rotation::Up => unreachable!(),
        },
        // from top and left
        B1001 => match rot {
            Rotation::Right => p!(2, 0, FLIP_Y), // ┴
            Rotation::Down => p!(2, 1),          // ┤
            _ => unreachable!(),
        },
        // from top sides
        B1010 => match rot {
            Rotation::Right => p!(4, 0), // ├
            Rotation::Left => p!(4, 3),  // ┤
            _ => unreachable!(),
        },
        // from top, left, bottom
        B1011 => match rot {
            Rotation::Right => p!(3, 0), // ┼
            _ => unreachable!(),
        },
        // from top and right
        B1100 => match rot {
            Rotation::Down => p!(2, 3, FLIP_X), // ├
            Rotation::Left => p!(2, 2),         // ┴
            _ => unreachable!(),
        },
        // from top, left, right
        B1101 => match rot {
            Rotation::Down => p!(3, 1), // ┼
            _ => unreachable!(),
        },
        // from top, right, bottom
        B1110 => match rot {
            Rotation::Left => p!(3, 0, FLIP_X), // ┼
            _ => unreachable!(),
        },
        B0000 => (
            0,
            match rot {
                Rotation::Left => 2,
                Rotation::Right => 0,
                Rotation::Down => 1,
                Rotation::Up => 3,
            },
            0,
        ),
        B1111 => unreachable!(),
    }
}

pub const FLIP_X: u8 = 1;
pub const FLIP_Y: u8 = 2;

/// # Safety
///
/// `with` must be square
pub unsafe fn flrot(flip: u8, rot: u8, with: &mut ImageHolder<4>) {
    if (flip & FLIP_X) != 0 {
        with.flip_h();
    }
    if (flip & FLIP_Y) != 0 {
        with.flip_v();
    }
    with.rotate(rot);
}

/// TODO figure out if a flip is cheaper than a `rotate_270`
pub fn rotations2tile(
    (index, rot, flip): (u8, u8, u8),
    name: &str,
    scale: Scale,
) -> ImageHolder<4> {
    let mut p = match index {
        0 => {
            load!(concat 0 => name which is ["reinforced-conduit" | "armored-duct" | "pulse-conduit" | "plated-conduit" | "conduit" | "conveyor" | "titanium-conveyor" | "armored-conveyor" | "duct"], scale)
        }
        1 => {
            load!(concat 1 => name which is ["reinforced-conduit" | "armored-duct" | "pulse-conduit" | "plated-conduit" | "conduit" | "conveyor" | "titanium-conveyor" | "armored-conveyor" | "duct"], scale)
        }
        2 => {
            load!(concat 2 => name which is ["reinforced-conduit" | "armored-duct" | "pulse-conduit" | "plated-conduit" | "conduit" | "conveyor" | "titanium-conveyor" | "armored-conveyor" | "duct"], scale)
        }
        3 => {
            load!(concat 3 => name which is ["reinforced-conduit" | "armored-duct" | "pulse-conduit" | "plated-conduit" | "conduit" | "conveyor" | "titanium-conveyor" | "armored-conveyor" | "duct"], scale)
        }
        _ => {
            load!(concat 4 => name which is ["reinforced-conduit" | "armored-duct" | "pulse-conduit" | "plated-conduit" | "conduit" | "conveyor" | "titanium-conveyor" | "armored-conveyor" | "duct"], scale)
        }
    };
    unsafe { flrot(flip, rot, &mut p) };
    p
}

pub fn mask(ctx: &RenderingContext, rot: Rotation, n: &str) -> U4 {
    macro_rules! c {
        ($in: expr, $srot: expr, $name: expr, $at: expr) => {{
            if let Some((b, rot)) = $in {
                if b.name() == $name {
                    // if they go down, we must not go up
                    (rot == $at && rot.mirrored(true, true) != $srot) as u8
                } else {
                    0
                }
            } else {
                0
            }
        }};
    }
    use Rotation::{Down, Left, Right, Up};
    let mut x = 0b0000;

    x |= 8 * c!(ctx.cross[0], rot, n, Down);
    x |= 4 * c!(ctx.cross[1], rot, n, Left);
    x |= 2 * c!(ctx.cross[2], rot, n, Up);
    x |= c!(ctx.cross[3], rot, n, Right);
    U4::from(x)
}

pub trait RotationState {
    fn get_rotation(&self) -> Option<Rotation>;
}
pub trait BlockState<'l> {
    fn get_block(&'l self) -> Option<&'l Block>;
}
pub trait Crossable {
    fn cross(&self, j: usize, c: &PositionContext) -> Cross;
}

#[test]
fn test_cross() {
    let mut reg = crate::block::BlockRegistry::default();
    crate::block::distribution::register(&mut reg);
    let mut ss = super::schematic::SchematicSerializer(&reg);
    macro_rules! test {
        ($schem: literal => $($a:tt,$b:tt,$c:tt,$d:tt)*) => {
            let s = ss.deserialize_base64($schem).unwrap();
            let mut c = vec![];
                println!("{:#?}", s.blocks);

            for (position, _) in s.block_iter() {
                let pctx = PositionContext {
                    position,
                    width: s.width,
                    height: s.height,
                };
                c.push(s.cross(&pctx));
            }
            let n = s.tags.get("name").map_or("<unknown>", |x| &x);
            let cc: Vec<Cross> = vec![
                $(define!($a,$b,$c,$d),)*
            ];
            if cc != c {
                let a = print_crosses(cc, s.height as usize);
                let b = print_crosses(c, s.height as usize);
                for diff in diff::lines(&a, &b) {
                    match diff {
                        diff::Result::Left(l)    => println!("\x1b[38;5;1m{}", l),
                        diff::Result::Right(r)   => println!("\x1b[38;5;2m{}", r),
                        diff::Result::Both(l, _) => println!("\x1b[0m{}", l),
                    }
                }
                print!("\x1b[0m");
                /*
                for diff in diff::slice(&c.into_iter().enumerate().collect::<Vec<_>>(), &cc.into_iter().enumerate().collect::<Vec<_>>()) {
                    match diff {
                        diff::Result::Left((i, l))    => println!("\x1b[38;5;1m- {l:?} at {i}"),
                        diff::Result::Right((i, r))   => println!("\x1b[38;5;2m+ {r:?} at {i}"),
                        diff::Result::Both((i, l), _) => println!("\x1b[0m  {l:?} at {i}"),
                    }
                }
                */
                panic!("test {n} \x1b[38;5;1mfailed\x1b[0m")
            }
            println!("test {n} \x1b[38;5;2mpassed\x1b[0m");
        };
    }
    // crosses go from bottom left -> top left -> bottom left + 1 -> top left + 1...
    // the symbols are directions (> => Right...), which mean the neighbors pointing direction
    // _ = no block

    // the basic test
    // ─┐
    // ─┤
    test!("bXNjaAF4nGNgYmBiZmDJS8xNZWBNSizOTGbgTkktTi7KLCjJzM9jYGBgy0lMSs0pZmCNfr9gTSwjA0dyfl5ZamV+EVCOhQEBGGEEM4hiZGAGAOb+EWA=" =>
    //  (0, 0)  (0, 1)
    //  n e s w borders (west void for first row)
        >,v,_,_ _,v,>,_
    //  (1, 0)  (1, 1)
        v,_,_,> _,_,v,>
    );
    // the loop test
    // ─│─
    // ─┼┐
    // ─└┘
    test!("bXNjaAF4nDWK4QqAIBCDd6dE0SNGP8zuh2CeaAS9fZk0xvjGBgNjYJM7BDaqy5h3qb6EfAZNAIboNokVvKyE0Wu65NbyDhM+cQv6mTtTM/WFYfqLm6m3lx9MAg7n" =>
        >,^,_,_ <,>,<,_ _,v,>,_
        >,<,_,< v,v,^,> _,>,>,<
        v,_,_,^ >,_,<,> _,_,v,v
    );
    // the snek test
    // └┐
    // ─┘
    test!("bXNjaAF4nGNgYmBiZmDJS8xNZWApzkvNZuBOSS1OLsosKMnMz2NgYGDLSUxKzSlmYIqOZWTgSM7PK0utzC8CSrAwIAAjEIIQhGJkYAIARA0Ozg==" =>
        ^,^,_,_ _,<,>,_
        <,_,_,> _,_,^,^
    );

    // the notile test
    test!("bXNjaAF4nCWJQQqAIBREx69E0Lq994oWph8STEMj6fZpzcDjDQMCSahoDsZsdN1TYB25aucz28uniMlxsdmf3wCGYDYOBbSsAqNN8eYn5XYofJEdAtSB31tfaoIVGw==" =>
        <,>,_,_ _,^,v,_
        ^,_,_,v _,_,>,<
    );
    // the asymmetrical test
    // <───
    // ───>
    test!("bXNjaAF4nEXJwQqAIBAE0HGVCPrE6GC2B0HdcCPw78MKnMMwj4EFWbjiM8N5bRnLwRpqPK8oBcCU/M5JQetmMAcpNzep/cCIAfX69yv6RF0PFy0O4Q==" =>
        <,>,_,_ _,<,>,_
        <,>,_,> _,<,>,<
        // <,_,_,> _,_,>,<
        <,_,_,> _,_,>,<
    );

    // the complex test
    // ─┬─│││─
    // ─┤─┘─┘─
    // ─┤┌─│─┐
    // ─┼┘─┴─│
    test!("bXNjaAF4nEWOUQ7CIBBEh2VZTbyCx/A2xg9a+WiC0LTGxNvb7Wjk5wEzb7M4QCO05UdBqj3PF5zuZR2XaX5OvQGwmodSV8j1FnAce3uVd1+24Iz/CYQQ8fcVHYEQIjqEXWEm9LwgX9kR+PLSbm2BMlN6Sk/3LhJnJu6S6CVmxl2MntEzv38AchUPug==" =>
        >,v,_,_ >,v,>,_ >,v,>,_ _,v,>,_
        v,<,_,> v,v,v,> v,>,v,> _,<,v,>
        v,>,_,v >,<,<,v <,^,v,v _,v,>,v
        <,>,_,< ^,v,>,v v,>,<,> _,^,^,<
        v,<,_,> >,>,>,< ^,^,v,^ _,^,>,v
        >,v,_,> ^,v,<,v ^,>,>,> _,>,^,^
        // <,_,_,< ^,_,>,v v,_,<,> _,_,^,<
        // v,_,_,> >,_,>,< ^,_,v,^ _,_,>,v
        // >,_,_,> ^,_,<,v ^,_,>,> _,_,^,^
        v,_,_,< >,_,v,> >,_,v,^ _,_,>,^
    );
}

#[test]
fn test_mask() {
    macro_rules! assert {
        ($a:tt,$b:tt,$c:tt,$d:tt => $rot: tt => $expect: expr) => {
            assert_eq!(mask!(define!($a, $b, $c, $d), $rot), $expect)
        };
    }
    macro_rules! mask {
        ($cross:expr, $rot: tt) => {
            mask(
                &RenderingContext {
                    position: PositionContext {
                        position: GridPos(5, 5),
                        width: 10,
                        height: 10,
                    },
                    cross: $cross,
                },
                dir!($rot),
                "conveyor",
            )
        };
    }
    assert!(_,_,_,_ => ^ => U4::B0000);
    assert!(v,_,_,_ => > => U4::B1000);
    assert!(v,v,_,_ => v => U4::B1000);
    assert!(_,v,>,_ => > => U4::B0000);
    assert!(v,>,<,> => ^ => U4::B0001);
    assert!(v,>,>,_ => > => U4::B1000);
}
