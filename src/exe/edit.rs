use std::borrow::Cow;
use std::env::Args;
use std::fs;
use std::io::{self, Write};

use plandustry::block::{build_registry, BlockRegistry, Rotation};
use plandustry::data::dynamic::DynData;
use plandustry::data::schematic::{Placement, ResizeError, Schematic, SchematicSerializer};
use plandustry::data::{base64, DataRead, DataWrite, GridPos, Serializer};
use plandustry::registry::RegistryEntry;

use crate::args::{self, ArgCount, ArgOption, OptionHandler};
use crate::print::print_schematic;
use crate::print_err;

struct State<'l> {
    reg: &'l BlockRegistry<'l>,
    schematic: Option<Schematic<'l>>,
    unsaved: bool,
    subregion: Option<Schematic<'l>>,
    quit: bool,
}

pub fn main(mut args: Args, arg_off: usize) {
    let mut handler = OptionHandler::default();
    let opt_file = handler
        .add(ArgOption::new(
            Some('f'),
            Some(Cow::Borrowed("file")),
            ArgCount::Required(1),
        ))
        .unwrap();
    if let Err(e) = args::parse(&mut args, &mut handler, arg_off) {
        print_err!(e, "Command error");
        return;
    }

    // try to load a schematic from the file argument or as base64
    let reg = build_registry();
    let mut ss = SchematicSerializer(&reg);
    let mut state = State {
        reg: &reg,
        schematic: None,
        unsaved: false,
        subregion: None,
        quit: false,
    };
    if let Some(path) = handler.get_value(opt_file).get_value() {
        match fs::read(path) {
            Ok(data) => match ss.deserialize(&mut DataRead::new(&data)) {
                Ok(s) => {
                    println!("Loaded schematic from {path}");
                    state.schematic = Some(s);
                }
                Err(e) => print_err!(e, "Could not read schematic from {path}"),
            },
            Err(e) => print_err!(e, "Could not read file {path:?}"),
        }
    } else if let Some(b64) = handler.get_literals().first() {
        match ss.deserialize_base64(b64) {
            Ok(s) => {
                println!("Loaded schematic from CLI");
                state.schematic = Some(s);
            }
            Err(e) => print_err!(e, "Could not read schematic"),
        }
    }
    if state.schematic.is_none() {
        println!(r#"No active schematic, use "new" or "load" to begin editing."#);
    }
    println!(r#"Type "help" for a list of available commands."#);

    // the main command interpreter loop
    let mut line_buff = String::new();
    let stdin = io::stdin();
    while !state.quit {
        line_buff.clear();
        print!("> ");
        if let Err(e) = io::stdout().flush() {
            // what the print & println macros would do
            panic!("failed printing to stdout: {e}");
        }
        match stdin.read_line(&mut line_buff) {
            Ok(..) => interpret(&mut state, line_buff.trim_start()),
            Err(e) => {
                print_err!(e, "Failed to read next command");
                if state.unsaved {
                    // special case because we wouldn't be able to read a path from stdin
                    match ss.serialize_base64(state.schematic.as_ref().unwrap()) {
                        Ok(curr) => println!("Current schematic: {curr}"),
                        Err(e) => print_err!(e, "Could not serialize schematic"),
                    }
                    state.unsaved = false;
                }
                break;
            }
        }
    }

    // give the user a chance to save their work
    if state.unsaved {
        let mut data = DataWrite::default();
        match ss.serialize(&mut data, state.schematic.as_ref().unwrap()) {
            Ok(()) => {
                let data = data.get_written();
                // SAFETY: base64 output is always valid ASCII
                let buff = unsafe { line_buff.as_mut_vec() };
                buff.resize(4 * ((data.len() + 2) / 3), 0);
                match base64::encode(data, buff) {
                    Ok(len) => println!("Current schematic: {}", &line_buff[..len]),
                    Err(e) => print_err!(e, "Could not convert schematic to base-64"),
                }

                println!(
                    "You have unsaved work. Please type a path to save to or press enter to quit."
                );
                loop {
                    line_buff.clear();
                    match stdin.read_line(&mut line_buff) {
                        Ok(..) => {
                            let path = line_buff.trim();
                            if path.is_empty() {
                                break;
                            }
                            match fs::write(path, data) {
                                Ok(()) => println!("Saved schematic to {path}"),
                                Err(e) => print_err!(e, "Could not write file {path:?}"),
                            }
                        }
                        Err(e) => {
                            print_err!(e, "Failed to read save path");
                            return;
                        }
                    }
                }
            }
            Err(e) => print_err!(e, "Could not serialize schematic"),
        }
    }
}

struct Tokenizer<'l>(Option<&'l str>);

impl<'l> Tokenizer<'l> {
    fn skip_ws(&mut self) {
        if let Some(curr) = self.0 {
            let curr = curr.trim_start();
            self.0 = if curr.is_empty() { None } else { Some(curr) };
        }
    }

    fn next(&mut self) -> Option<&'l str> {
        self.skip_ws();
        if let Some(curr) = self.0 {
            if curr.len() >= 2 && (curr.as_bytes()[0] == b'"' || curr.as_bytes()[0] == b'\'') {
                match (curr[1..]).find(curr.as_bytes()[0] as char) {
                    None => {
                        self.0 = None;
                        Some(&curr[1..])
                    }
                    Some(end) => {
                        let rest = &curr[(end + 2)..];
                        self.0 = if rest.is_empty() { None } else { Some(rest) };
                        Some(&curr[1..end])
                    }
                }
            } else {
                match curr.find(char::is_whitespace) {
                    None => {
                        self.0 = None;
                        Some(curr)
                    }
                    Some(end) => {
                        let rest = &curr[end..];
                        self.0 = if rest.is_empty() { None } else { Some(rest) };
                        Some(&curr[..end])
                    }
                }
            }
        } else {
            None
        }
    }

    fn remainder(&mut self) -> Option<&'l str> {
        self.skip_ws();
        if let Some(curr) = self.0 {
            let bytes = curr.as_bytes();
            if bytes.len() >= 2
                && (bytes[0] == b'"' || bytes[0] == b'\'')
                && bytes[bytes.len() - 1] == bytes[0]
            {
                self.0 = None;
                return Some(&curr[1..(curr.len() - 1)]);
            }
        }
        let curr = self.0;
        self.0 = None;
        curr
    }
}

enum Command {
    Help,
    New,
    Input,
    Load,
    Place,
    Rotate,
    Mirror,
    Move,
    Resize,
    Remove,
    Sub,
    Print,
    Dump,
    Save,
    Quit,
}

impl Command {
    fn print_help(&self, indent: usize) {
        match self {
            Self::Help => println!(
                "{:<indent$}Prints a list of available commands",
                "\"help\":"
            ),
            Self::New => println!(
                "{:<indent$}Creates a new schematic, erasing the currently loaded one",
                "\"new\":"
            ),
            Self::Input => println!(
                "{:<indent$}Loads a new schematic from a base-64 encoded string",
                "\"input\":"
            ),
            Self::Load => println!("{:<indent$}Loads a new schematic from a file", "\"load\":"),
            Self::Place => println!(
                "{:<indent$}Places a block if enough space is available",
                "\"place\":"
            ),
            Self::Rotate => println!(
                "{:<indent$}Rotates the schematic (CCW) in increments of 90 degrees",
                "\"rotate\":"
            ),
            Self::Mirror => println!(
                "{:<indent$}Mirrors the schematic horizontally or vertically",
                "\"mirror\":"
            ),
            Self::Move => println!(
                "{:<indent$}Moves all blocks by a certain offset",
                "\"move\":"
            ),
            Self::Resize => println!(
                "{:<indent$}Resizes the schematic and offsets it",
                "\"resize\":"
            ),
            Self::Remove => println!(
                "{:<indent$}Removes blocks at a position or within a region",
                "\"remove\":"
            ),
            Self::Sub => println!(
                "{:<indent$}Various commands for editing subregions",
                "\"sub\":"
            ),
            Self::Print => println!(
                "{:<indent$}Prints the schematic in a visual representation",
                "\"print\":"
            ),
            Self::Dump => println!(
                "{:<indent$}Prints the schematic as a base-64 encoded string",
                "\"dump\":"
            ),
            Self::Save => println!("{:<indent$}Saves the schematic to a file", "\"save\":"),
            Self::Quit => println!(
                "{:<indent$}Offers to save unsaved work and exits the program",
                "\"quit\":"
            ),
        }
        self.print_usage(indent);
    }

    fn print_usage(&self, indent: usize) {
        match self {
            Self::Help => (),
            Self::New => println!(r#"{:indent$}  Usage: "new" <width> [<height>]"#, ""),
            Self::Input => println!(r#"{:indent$}  Usage: "input" <base64>"#, ""),
            Self::Load => println!(r#"{:indent$}  Usage: "load" <load path>"#, ""),
            Self::Place => {
                println!(
                    r#"{:indent$}  Usage: "place" <x> <y> <block name> [<rotation> [<replace>]]"#,
                    ""
                );
                println!(
                    r#"{:indent$}  Rotation is one of right, up, left, down or compass angles"#,
                    ""
                )
            }
            Self::Rotate => println!(r#"{:indent$}  Usage: "rotate" <angle>"#, ""),
            Self::Mirror => println!(r#"{:indent$}  Usage: "mirror" <axis>"#, ""),
            Self::Move => println!(r#"{:indent$}  Usage: "move" <dx> <dy>"#, ""),
            Self::Resize => println!(
                r#"{:indent$}  Usage: "resize" <width> <height> [<dx> <dy>]"#,
                ""
            ),
            Self::Remove => println!(r#"{:indent$}  Usage: "remove" <x0> <y0> [<x1> <y1>]"#, ""),
            Self::Sub => println!(r#"{:indent$}  Usage: "sub" ... (see "sub help")"#, ""),
            Self::Print | Self::Dump => (),
            Self::Save => println!(r#"{:indent$}  Usage: "save" <save path>"#, ""),
            Self::Quit => (),
        }
    }
}

macro_rules! parse_num {
    ($cmd:path, $name:expr, <$type:ty>::from($val:expr)) => {
        match $val {
            None => {
                eprintln!("Missing argument: {}", $name);
                $cmd.print_usage(0);
                return;
            }
            Some(s) => match s.parse::<$type>() {
                Ok(v) => v,
                Err(e) => {
                    print_err!(e, "Could not parse {}", $name);
                    $cmd.print_usage(0);
                    return;
                }
            },
        }
    };
    ($cmd:path, $tokens:expr, $name:expr, $type:ty) => {
        parse_num!($cmd, $name, <$type>::from($tokens.next()))
    };
}

fn interpret(state: &mut State, cmd: &str) {
    let mut tokens = Tokenizer(Some(cmd));
    match tokens.next() {
        None => println!(r#"Empty command, type "quit" to exit"#),
        Some("help") => {
            println!(r#"List of available commands:"#);
            const INDENT: usize = 12;
            Command::Help.print_help(INDENT);
            Command::New.print_help(INDENT);
            Command::Input.print_help(INDENT);
            Command::Load.print_help(INDENT);
            Command::Place.print_help(INDENT);
            Command::Rotate.print_help(INDENT);
            Command::Mirror.print_help(INDENT);
            Command::Remove.print_help(INDENT);
            Command::Sub.print_help(INDENT);
            Command::Print.print_help(INDENT);
            Command::Dump.print_help(INDENT);
            Command::Save.print_help(INDENT);
            Command::Quit.print_help(INDENT);
            println!();
            println!("Legend: \"literal (excluding quotes)\" <required> [<optional>]");
            println!("Arguments are delimited by whitespace, unless surrounded with ' or \"");
            if tokens.remainder().is_some() {
                eprintln!("Extra arguments are considered an error");
            } else {
                println!("Extra arguments are considered an error");
            }
        }
        Some("new") => {
            let width = parse_num!(Command::New, tokens, "width", u16);
            if width == 0 {
                eprintln!("Schematic width must be positive");
                return;
            }
            let height = parse_num!(Command::New, tokens, "height", u16);
            if height == 0 {
                eprintln!("Schematic height must be positive");
                return;
            }
            if tokens.remainder().is_some() {
                eprintln!(r#"Too many parameters for "new""#);
                Command::New.print_usage(0);
                return;
            }
            state.schematic = Some(Schematic::new(width, height));
            // it's empty, no need to save this
            state.unsaved = false;
        }
        Some("input") => {
            let schematic = match tokens.next() {
                None => {
                    eprintln!("Missing argument: base64");
                    Command::Input.print_usage(0);
                    return;
                }
                Some(b64) => match SchematicSerializer(state.reg).deserialize_base64(b64) {
                    Ok(s) => s,
                    Err(e) => {
                        print_err!(e, "Could not deserialize schematic");
                        return;
                    }
                },
            };
            if tokens.remainder().is_some() {
                eprintln!(r#"Too many parameters for "input""#);
                Command::Input.print_usage(0);
                return;
            }
            state.schematic = Some(schematic);
            state.unsaved = false;
        }
        Some("load") => {
            let schematic = match tokens.next() {
                None => {
                    eprintln!("Missing argument: load path");
                    Command::Load.print_usage(0);
                    return;
                }
                Some(path) => {
                    let data = match fs::read(path) {
                        Ok(d) => d,
                        Err(e) => {
                            print_err!(e, "Could not load from file");
                            return;
                        }
                    };
                    match SchematicSerializer(state.reg).deserialize(&mut DataRead::new(&data)) {
                        Ok(s) => s,
                        Err(e) => {
                            print_err!(e, "Could not deserialize schematic");
                            return;
                        }
                    }
                }
            };
            if tokens.remainder().is_some() {
                eprintln!(r#"Too many parameters for "load""#);
                Command::Load.print_usage(0);
                return;
            }
            state.schematic = Some(schematic);
            state.unsaved = false;
        }
        Some("place") => {
            let Some(ref mut schematic) = state.schematic
			else
			{
				eprintln!(r#"Command "place" requires an active schematic (see "help")"#);
				return;
			};
            let x = parse_num!(Command::Place, tokens, "x", u16);
            let y = parse_num!(Command::Place, tokens, "y", u16);
            if x >= schematic.get_width() || y >= schematic.get_height() {
                eprintln!(
                    "Invalid coordinate ({x} / {y}) out of bounds ({} / {})",
                    schematic.get_width(),
                    schematic.get_height()
                );
                return;
            }
            let block = match tokens.next() {
                None => {
                    eprintln!("Missing argument: block name");
                    Command::Place.print_usage(0);
                    return;
                }
                Some(name) => match state.reg.get(name) {
                    None => {
                        eprintln!("No such block {name:?}");
                        return;
                    }
                    Some(b) => b,
                },
            };
            let rot = match tokens.next() {
                None => None,
                Some("right") | Some("east") => Some(Rotation::Right),
                Some("up") | Some("north") => Some(Rotation::Up),
                Some("left") | Some("west") => Some(Rotation::Left),
                Some("down") | Some("south") => Some(Rotation::Down),
                Some(rot) => {
                    eprintln!("Invalid rotation {rot:?}");
                    return;
                }
            };
            let replace = if rot.is_some() {
                match tokens.next() {
                    None => None,
                    Some("true") | Some("yes") => Some(true),
                    Some("false") | Some("no") => Some(false),
                    Some(replace) => {
                        eprintln!("Invalid replacement {replace:?}");
                        return;
                    }
                }
            } else {
                None
            };
            if tokens.remainder().is_some() {
                eprintln!(r#"Too many parameters for "place""#);
                Command::Place.print_usage(0);
                return;
            }
            let rot = rot.unwrap_or(Rotation::Right);
            let result = if replace.unwrap_or(false) {
                schematic
                    .replace(x, y, block, DynData::Empty, rot, false)
                    .err()
            } else {
                schematic.set(x, y, block, DynData::Empty, rot).err()
            };
            if let Some(e) = result {
                print_err!(e, "Failed to place block at {x} / {y}");
            }
        }
        Some("rotate") => {
            let Some(ref mut schematic) = state.schematic
			else
			{
				eprintln!(r#"Command "rotate" requires an active schematic (see "help")"#);
				return;
			};
            let angle = parse_num!(Command::Rotate, tokens, "angle", i32);
            if angle % 90 != 0 {
                eprintln!("Rotation angle must be a multiple of 90 degrees");
                return;
            }
            if tokens.remainder().is_some() {
                eprintln!(r#"Too many parameters for "rotate""#);
                Command::Rotate.print_usage(0);
                return;
            }
            match (angle / 90) % 4 {
                0 => (),
                1 | -3 => {
                    schematic.rotate(false);
                    state.unsaved = true;
                }
                2 | -2 => {
                    schematic.rotate_180();
                    state.unsaved = true;
                }
                3 | -1 => {
                    schematic.rotate(true);
                    state.unsaved = true;
                }
                a => unreachable!("angle {angle} -> {a}"),
            }
        }
        Some("mirror") => {
            let Some(ref mut schematic) = state.schematic
			else
			{
				eprintln!(r#"Command "mirror" requires an active schematic (see "help")"#);
				return;
			};
            let (x, y) = match tokens.next() {
                None => {
                    eprintln!("Missing argument: axis");
                    Command::Mirror.print_usage(0);
                    return;
                }
                Some("x") | Some("h") | Some("horizontal") | Some("horizontally") => (true, false),
                Some("y") | Some("v") | Some("vertical") | Some("vertically") => (false, true),
                Some("both") | Some("all") => (true, true),
                Some(axis) => {
                    eprintln!("Invalid mirroring axis: {axis:?}");
                    return;
                }
            };
            if tokens.remainder().is_some() {
                eprintln!(r#"Too many parameters for "mirror""#);
                Command::Mirror.print_usage(0);
                return;
            }
            schematic.mirror(x, y);
            state.unsaved = true;
        }
        Some("move") => {
            let Some(ref mut schematic) = state.schematic
			else
			{
				eprintln!(r#"Command "move" requires an active schematic (see "help")"#);
				return;
			};
            let dx = parse_num!(Command::Move, tokens, "dx", i16);
            let dy = parse_num!(Command::Move, tokens, "dy", i16);
            if tokens.remainder().is_some() {
                eprintln!(r#"Too many parameters for "move""#);
                Command::Move.print_usage(0);
                return;
            }
            if dx != 0 && dy != 0 {
                if let Err(e) =
                    schematic.resize(dx, dy, schematic.get_width(), schematic.get_height())
                {
                    match e {
                        ResizeError::XOffset { dx, old_w, new_w } => {
                            debug_assert_eq!(old_w, new_w);
                            eprintln!("Invalid horizontal move {dx} not in ]-{old_w}, {new_w}[");
                        }
                        ResizeError::YOffset { dy, old_h, new_h } => {
                            debug_assert_eq!(old_h, new_h);
                            eprintln!("Invalid vertical move {dy} not in ]-{old_h}, {new_h}[");
                        }
                        ResizeError::Truncated {
                            right,
                            top,
                            left,
                            bottom,
                        } => {
                            eprint!("Move would truncate schematic: ");
                            let mut first = true;
                            if right > 0 {
                                eprint!("{}right: {right}", if first { "" } else { ", " });
                                first = false;
                            }
                            if top > 0 {
                                eprint!("{}top: {top}", if first { "" } else { ", " });
                                first = false;
                            }
                            if left > 0 {
                                eprint!("{}left: {left}", if first { "" } else { ", " });
                                first = false;
                            }
                            if bottom > 0 {
                                eprint!("{}bottom: {bottom}", if first { "" } else { ", " });
                                first = false;
                            }
                            if first {
                                eprintln!("<unknown>");
                            } else {
                                eprintln!();
                            }
                        }
                        _ => print_err!(e, "Unexpected resize error (for {dx} / {dy})"),
                    }
                } else {
                    state.unsaved = true;
                }
            }
        }
        Some("resize") => {
            let Some(ref mut schematic) = state.schematic
			else
			{
				eprintln!(r#"Command "resize" requires an active schematic (see "help")"#);
				return;
			};
            let w = parse_num!(Command::Resize, tokens, "width", u16);
            if w == 0 {
                eprintln!("Schematic width must be positive");
                return;
            }
            let h = parse_num!(Command::Resize, tokens, "height", u16);
            if h == 0 {
                eprintln!("Schematic height must be positive");
                return;
            }
            let (dx, dy) = if let arg @ Some(..) = tokens.next() {
                let dx = parse_num!(Command::Resize, "dx", <i16>::from(arg));
                let dy = parse_num!(Command::Resize, tokens, "dy", i16);
                (dx, dy)
            } else {
                (0, 0)
            };
            if tokens.remainder().is_some() {
                eprintln!(r#"Too many parameters for "resize""#);
                Command::Resize.print_usage(0);
                return;
            }
            if w != schematic.get_width() || h != schematic.get_height() || dx != 0 || dy != 0 {
                if let Err(e) = schematic.resize(dx, dy, w, h) {
                    print_err!(e, "Could not resize schematic");
                }
            }
        }
        Some("remove") => {
            let Some(ref mut schematic) = state.schematic
			else
			{
				eprintln!(r#"Command "remove" requires an active schematic (see "help")"#);
				return;
			};
            let x0 = parse_num!(Command::Remove, tokens, "x0", u16);
            let y0 = parse_num!(Command::Remove, tokens, "y0", u16);
            if x0 >= schematic.get_width() || y0 >= schematic.get_height() {
                eprintln!(
                    "Invalid coordinate ({x0} / {y0}) out of bounds ({} / {})",
                    schematic.get_width(),
                    schematic.get_height()
                );
                return;
            }
            let (x0, y0, x1, y1) = if let arg @ Some(..) = tokens.next() {
                let x1 = parse_num!(Command::Remove, "x1", <u16>::from(arg));
                let y1 = parse_num!(Command::Remove, tokens, "y1", u16);
                if x1 >= schematic.get_width() || y1 >= schematic.get_height() {
                    eprintln!(
                        "Invalid coordinate ({x1} / {y1}) out of bounds ({} / {})",
                        schematic.get_width(),
                        schematic.get_height()
                    );
                    return;
                }
                (x0.min(x1), y0.min(y1), x0.max(x1), y0.max(y1))
            } else {
                (x0, y0, x0, y0)
            };
            if tokens.remainder().is_some() {
                eprintln!(r#"Too many parameters for "remove""#);
                Command::Remove.print_usage(0);
                return;
            }
            if x1 > x0 || y1 > y0 {
                let mut cnt = 0u32;
                for y in y0..=y1 {
                    for x in x0..=x1 {
                        // position was already checked while parsing
                        if schematic.take(x, y).unwrap().is_some() {
                            cnt += 1;
                        }
                    }
                }
                println!("Removed {cnt} blocks in {x0} / {y0} to {x1} / {y1}");
                if cnt > 0 {
                    state.unsaved = true;
                }
            } else {
                // position was already checked while parsing
                match schematic.take(x0, y0).unwrap() {
                    None => (),
                    Some(p) => {
                        println!(
                            "Removed block {} from {x0} / {y0}",
                            p.get_block().get_name()
                        );
                        state.unsaved = true;
                    }
                }
            }
        }
        Some("sub") => interpret_sub(state, &mut tokens),
        Some("print") => {
            let Some(ref schematic) = state.schematic
			else
			{
				eprintln!(r#"Command "print" requires an active schematic (see "help")"#);
				return;
			};
            if tokens.remainder().is_some() {
                eprintln!(r#"Too many parameters for "print""#);
                Command::Print.print_usage(0);
                return;
            }
            print_schematic(schematic);
        }
        Some("dump") => {
            let Some(ref schematic) = state.schematic
			else
			{
				eprintln!(r#"Command "dump" requires an active schematic (see "help")"#);
				return;
			};
            if tokens.remainder().is_some() {
                eprintln!(r#"Too many parameters for "dump""#);
                Command::Dump.print_usage(0);
                return;
            }
            let b64 = match SchematicSerializer(state.reg).serialize_base64(schematic) {
                Ok(b64) => b64,
                Err(e) => {
                    print_err!(e, "Could not serialize schematic");
                    return;
                }
            };
            println!("Schematic: {}", b64);
        }
        Some("save") => {
            let Some(ref schematic) = state.schematic
			else
			{
				eprintln!(r#"Command "save" requires an active schematic (see "help")"#);
				return;
			};
            let Some(path) = tokens.next()
			else
			{
				eprintln!("Missing argument: save path");
				Command::Save.print_usage(0);
				return;
			};
            if tokens.remainder().is_some() {
                eprintln!(r#"Too many parameters for "save""#);
                Command::Save.print_usage(0);
                return;
            }
            let mut serial_buff = DataWrite::default();
            if let Err(e) = SchematicSerializer(state.reg).serialize(&mut serial_buff, schematic) {
                print_err!(e, "Could not serialize schematic");
                return;
            }
            if let Err(e) = fs::write(path, serial_buff.get_written()) {
                print_err!(e, "Could not write to file");
                return;
            }
            state.unsaved = false;
            println!("Saved schematic to {path}.");
        }
        Some("quit") => state.quit = true,
        Some(unknown) => eprintln!("Unknown command {unknown:?}"),
    }
}

enum SubCommand {
    Help,
    Input,
    Copy,
    Cut,
    Paste,
    Place,
    Rotate,
    Mirror,
    Move,
    Resize,
    Remove,
    Print,
    Dump,
}

impl SubCommand {
    fn print_help(&self, indent: usize) {
        match self {
            Self::Help => println!(
                "{:<indent$}Prints a list of available commands",
                "\"sub\" \"help\":"
            ),
            Self::Input => println!(
                "{:<indent$}Loads a new subregion from a base-64 encoded string",
                "\"sub\" \"input\":"
            ),
            Self::Copy => println!(
                "{:<indent$}Replaces the current subregion by copying from the schematic",
                "\"sub\" \"copy\":"
            ),
            Self::Cut => println!(
                "{:<indent$}Replaces the current subregion by removing from the schematic",
                "\"sub\" \"cut\":"
            ),
            Self::Paste => println!(
                "{:<indent$}Places a copy of the current subregion into the schematic",
                "\"sub\" \"paste\":"
            ),
            Self::Place => println!(
                "{:<indent$}Places a block if enough space is available",
                "\"sub\" \"place\":"
            ),
            Self::Rotate => println!(
                "{:<indent$}Rotates the current subregion (CCW) in increments of 90 degrees",
                "\"sub\" \"rotate\":"
            ),
            Self::Mirror => println!(
                "{:<indent$}Mirrors the current subregion horizontally or vertically",
                "\"sub\" \"mirror\":"
            ),
            Self::Move => println!(
                "{:<indent$}Moves all blocks by a certain offset",
                "\"sub\" \"move\":"
            ),
            Self::Resize => println!(
                "{:<indent$}Resizes the current subregion and offsets it",
                "\"sub\" \"resize\":"
            ),
            Self::Remove => println!(
                "{:<indent$}Removes blocks at a position or within a region",
                "\"sub\" \"remove\":"
            ),
            Self::Print => println!(
                "{:<indent$}Prints the current subregion in a visual representation",
                "\"sub\" \"print\":"
            ),
            Self::Dump => println!(
                "{:<indent$}Prints the current subregion as a base-64 encoded string",
                "\"sub\" \"dump\":"
            ),
        }
        self.print_usage(indent);
    }

    fn print_usage(&self, indent: usize) {
        match self {
            Self::Help => (),
            Self::Input => println!(r#"{:indent$}  Usage: "sub" "input" <base64>"#, ""),
            Self::Copy => {
                println!(
                    r#"{:indent$}  Usage: "sub" "copy" <x0> <y0> <x1> <y1> [<strict>]"#,
                    ""
                );
                println!(
                    r#"{:indent$}  Strictness ignores blocks which are not entirely within the bounds"#,
                    ""
                );
            }
            Self::Cut => {
                println!(
                    r#"{:indent$}  Usage: "sub" "cut" <x0> <y0> <x1> <y1> [<strict>]"#,
                    ""
                );
                println!(
                    r#"{:indent$}  Strictness ignores blocks which are not entirely within the bounds"#,
                    ""
                );
            }
            Self::Paste => println!(r#"{:indent$}  Usage: "sub" "paste" <x> <y>"#, ""),
            Self::Place => {
                println!(
                    r#"{:indent$}  Usage: "sub" "place" <x> <y> <block name> [<rotation> [<replace>]]"#,
                    ""
                );
                println!(
                    r#"{:indent$}  Rotation is one of right, up, left, down or compass angles"#,
                    ""
                )
            }
            Self::Rotate => println!(r#"{:indent$}  Usage: "sub" "rotate" <angle>"#, ""),
            Self::Mirror => println!(r#"{:indent$}  Usage: "sub" "mirror" <axis>"#, ""),
            Self::Move => println!(r#"{:indent$}  Usage: "sub" "move" <dx> <dy>"#, ""),
            Self::Resize => println!(
                r#"{:indent$}  Usage: "sub" "resize" <width> <height> [<dx> <dy>]"#,
                ""
            ),
            Self::Remove => println!(
                r#"{:indent$}  Usage: "sub" "remove" <x0> <y0> [<x1> <y1>]"#,
                ""
            ),
            Self::Print | Self::Dump => (),
        }
    }
}

fn interpret_sub(state: &mut State, tokens: &mut Tokenizer) {
    match tokens.next() {
        None => println!(r#"Empty "sub" command, type "sub help" for more info"#),
        Some("help") => {
            const INDENT: usize = 20;
            println!(r#"List of available commands for \"sub\":"#);
            SubCommand::Help.print_help(INDENT);
            SubCommand::Input.print_help(INDENT);
            SubCommand::Copy.print_help(INDENT);
            SubCommand::Cut.print_help(INDENT);
            SubCommand::Paste.print_help(INDENT);
            SubCommand::Place.print_help(INDENT);
            SubCommand::Rotate.print_help(INDENT);
            SubCommand::Mirror.print_help(INDENT);
            SubCommand::Move.print_help(INDENT);
            SubCommand::Resize.print_help(INDENT);
            SubCommand::Remove.print_help(INDENT);
            SubCommand::Print.print_help(INDENT);
            SubCommand::Dump.print_help(INDENT);
            if tokens.remainder().is_some() {
                eprintln!("Extra arguments are considered an error");
            }
        }
        Some("input") => {
            let subregion = match tokens.next() {
                None => {
                    eprintln!("Missing argument: base64");
                    SubCommand::Input.print_usage(0);
                    return;
                }
                Some(b64) => match SchematicSerializer(state.reg).deserialize_base64(b64) {
                    Ok(s) => s,
                    Err(e) => {
                        print_err!(e, "Could not deserialize schematic");
                        return;
                    }
                },
            };
            if tokens.remainder().is_some() {
                eprintln!(r#"Too many parameters for "sub input""#);
                SubCommand::Input.print_usage(0);
                return;
            }
            state.subregion = Some(subregion);
        }
        Some(op @ ("copy" | "cut")) => {
            let modify_original = op == "cut";
            let Some(ref mut schematic) = state.schematic
			else
			{
				eprintln!(r#"Command "sub {op}" requires an active schematic (see "help")"#);
				return;
			};
            let x0 = if modify_original {
                parse_num!(SubCommand::Cut, tokens, "x0", u16)
            } else {
                parse_num!(SubCommand::Copy, tokens, "x0", u16)
            };
            let y0 = if modify_original {
                parse_num!(SubCommand::Cut, tokens, "y0", u16)
            } else {
                parse_num!(SubCommand::Copy, tokens, "y0", u16)
            };
            if x0 >= schematic.get_width() || y0 >= schematic.get_height() {
                eprintln!(
                    "Invalid lower coordinate ({x0} / {y0}) out of bounds ({} / {})",
                    schematic.get_width(),
                    schematic.get_height()
                );
                return;
            }
            let x1 = if modify_original {
                parse_num!(SubCommand::Cut, tokens, "x1", u16)
            } else {
                parse_num!(SubCommand::Copy, tokens, "x1", u16)
            };
            let y1 = if modify_original {
                parse_num!(SubCommand::Cut, tokens, "y1", u16)
            } else {
                parse_num!(SubCommand::Copy, tokens, "y1", u16)
            };
            if x1 < x0 || y1 < y0 {
                eprintln!(
                    "Invalid upper coordinate ({x1} / {y1}) too low (lower bound {x0} / {y0})"
                );
                return;
            }
            if x1 >= schematic.get_width() || y1 >= schematic.get_height() {
                eprintln!(
                    "Invalid upper coordinate ({x1} / {y1}) out of bounds ({} / {})",
                    schematic.get_width(),
                    schematic.get_height()
                );
                return;
            }
            let strict = match tokens.next() {
                None => false,
                Some("true") | Some("yes") => true,
                Some("false") | Some("no") => false,
                Some(strict) => {
                    eprintln!("Invalid strictness {strict:?}");
                    return;
                }
            };
            if tokens.remainder().is_some() {
                eprintln!(r#"Too many parameters for "sub {op}""#);
                SubCommand::Copy.print_usage(0);
                return;
            }
            let mut true_x0 = x0;
            let mut true_y0 = y0;
            let mut true_x1 = x1;
            let mut true_y1 = y1;
            let mut targets = Vec::new();
            for p in schematic.block_iter() {
                let sz = p.get_block().get_size() as u16;
                let pos0 = (p.get_pos().0 - (sz - 1) / 2, p.get_pos().1 - (sz - 1) / 2);
                let pos1 = (p.get_pos().0 + sz / 2, p.get_pos().1 + sz / 2);
                let include = if strict {
                    pos0.0 >= x0 && pos0.1 >= y0 && pos1.0 <= x1 && pos1.1 <= y1
                } else {
                    pos0.0 <= x1 && pos0.1 <= y1 && pos1.0 >= x0 && pos1.1 >= y0
                };
                if include {
                    targets.push(p.get_pos());
                    if !strict {
                        if pos0.0 < true_x0 {
                            true_x0 = pos0.0;
                        }
                        if pos0.1 < true_y0 {
                            true_y0 = pos0.1;
                        }
                        if pos1.0 > true_x1 {
                            true_x1 = pos1.0;
                        }
                        if pos1.1 > true_y1 {
                            true_y1 = pos1.1;
                        }
                    }
                }
            }
            let mut subregion = Schematic::new(true_x1 - true_x0 + 1, true_y1 - true_y0 + 1);
            if !targets.is_empty() {
                if modify_original {
                    state.unsaved = true;
                }
                for GridPos(x, y) in targets {
                    let mut original: Option<Placement> = None;
                    let place = if modify_original {
                        original = schematic.take(x, y).unwrap();
                        original.as_ref().unwrap()
                    } else {
                        schematic.get(x, y).unwrap().unwrap()
                    };
                    let data = match place.get_state() {
                        None => DynData::Empty,
                        Some(d) => place.get_block().serialize_state(d).unwrap(),
                    };
                    subregion
                        .set(
                            place.get_pos().0 - true_x0,
                            place.get_pos().1 - true_y0,
                            place.get_block(),
                            data,
                            place.get_rotation(),
                        )
                        .unwrap();
                    drop(original);
                }
            }
            state.subregion = Some(subregion);
        }
        Some("paste") => {
            let Some(ref mut schematic) = state.schematic
			else
			{
				eprintln!(r#"Command "sub paste" requires an active schematic (see "help")"#);
				return;
			};
            let Some(ref subregion) = state.subregion
			else
			{
				eprintln!(r#"Command "sub paste" requires an active subregion (see "sub help")"#);
				return;
			};
            let x = parse_num!(SubCommand::Paste, tokens, "x", u16);
            let y = parse_num!(SubCommand::Paste, tokens, "y", u16);
            if subregion.get_width() > schematic.get_width()
                || subregion.get_height() > schematic.get_height()
            {
                eprintln!(
                    "Subregion ({} / {}) is larger than schematic ({} / {})",
                    subregion.get_width(),
                    subregion.get_height(),
                    schematic.get_width(),
                    schematic.get_height()
                );
                return;
            }
            if x >= schematic.get_width() - subregion.get_width() {
                let x1 = x + subregion.get_width() - 1;
                let y1 = y + subregion.get_height() - 1;
                eprintln!(
                    "Invalid coordinate ({x} / {y} to {x1} / {y1}) out of bounds ({} / {})",
                    schematic.get_width(),
                    schematic.get_height()
                );
                return;
            }
            if tokens.remainder().is_some() {
                eprintln!(r#"Too many parameters for "sub paste""#);
                SubCommand::Paste.print_usage(0);
                return;
            }
            state.unsaved = true;
            for p in subregion.block_iter() {
                let data = match p.get_state() {
                    None => DynData::Empty,
                    Some(d) => p.get_block().serialize_state(d).unwrap(),
                };
                schematic
                    .replace(
                        x + p.get_pos().0,
                        y + p.get_pos().1,
                        p.get_block(),
                        data,
                        p.get_rotation(),
                        false,
                    )
                    .unwrap();
            }
        }
        Some("place") => {
            let Some(ref mut subregion) = state.subregion
			else
			{
				eprintln!(r#"Command "sub place" requires an active subregion (see "sub help")"#);
				return;
			};
            let x = parse_num!(SubCommand::Place, tokens, "x", u16);
            let y = parse_num!(SubCommand::Place, tokens, "y", u16);
            if x >= subregion.get_width() || y >= subregion.get_height() {
                eprintln!(
                    "Invalid coordinate ({x} / {y}) out of bounds ({} / {})",
                    subregion.get_width(),
                    subregion.get_height()
                );
                return;
            }
            let block = match tokens.next() {
                None => {
                    eprintln!("Missing argument: block name");
                    SubCommand::Place.print_usage(0);
                    return;
                }
                Some(name) => match state.reg.get(name) {
                    None => {
                        eprintln!("No such block {name:?}");
                        return;
                    }
                    Some(b) => b,
                },
            };
            let rot = match tokens.next() {
                None => None,
                Some("right") | Some("east") => Some(Rotation::Right),
                Some("up") | Some("north") => Some(Rotation::Up),
                Some("left") | Some("west") => Some(Rotation::Left),
                Some("down") | Some("south") => Some(Rotation::Down),
                Some(rot) => {
                    eprintln!("Invalid rotation {rot:?}");
                    return;
                }
            };
            let replace = if rot.is_some() {
                match tokens.next() {
                    None => None,
                    Some("true") | Some("yes") => Some(true),
                    Some("false") | Some("no") => Some(false),
                    Some(replace) => {
                        eprintln!("Invalid replacement {replace:?}");
                        return;
                    }
                }
            } else {
                None
            };
            if tokens.remainder().is_some() {
                eprintln!(r#"Too many parameters for "sub place""#);
                SubCommand::Place.print_usage(0);
                return;
            }
            let rot = rot.unwrap_or(Rotation::Right);
            let result = if replace.unwrap_or(false) {
                subregion
                    .replace(x, y, block, DynData::Empty, rot, false)
                    .err()
            } else {
                subregion.set(x, y, block, DynData::Empty, rot).err()
            };
            if let Some(e) = result {
                print_err!(e, "Failed to place block at {x} / {y}");
            }
        }
        Some("rotate") => {
            let Some(ref mut subregion) = state.subregion
			else
			{
				eprintln!(r#"Command "sub rotate" requires an active subgregion (see "sub help")"#);
				return;
			};
            let angle = parse_num!(SubCommand::Rotate, tokens, "angle", i32);
            if angle % 90 != 0 {
                eprintln!("Rotation angle must be a multiple of 90 degrees");
                return;
            }
            if tokens.remainder().is_some() {
                eprintln!(r#"Too many parameters for "sub rotate""#);
                SubCommand::Rotate.print_usage(0);
                return;
            }
            match (angle / 90) % 4 {
                0 => (),
                1 | -3 => subregion.rotate(false),
                2 | -2 => subregion.rotate_180(),
                3 | -1 => subregion.rotate(true),
                a => unreachable!("angle {angle} -> {a}"),
            }
        }
        Some("mirror") => {
            let Some(ref mut subregion) = state.subregion
			else
			{
				eprintln!(r#"Command "sub mirror" requires an active subregion (see "sub help")"#);
				return;
			};
            let (x, y) = match tokens.next() {
                None => {
                    eprintln!("Missing argument: axis");
                    SubCommand::Mirror.print_usage(0);
                    return;
                }
                Some("x") | Some("h") | Some("horizontal") | Some("horizontally") => (true, false),
                Some("y") | Some("v") | Some("vertical") | Some("vertically") => (false, true),
                Some("both") | Some("all") => (true, true),
                Some(axis) => {
                    eprintln!("Invalid mirroring axis: {axis:?}");
                    return;
                }
            };
            if tokens.remainder().is_some() {
                eprintln!(r#"Too many parameters for "sub mirror""#);
                SubCommand::Mirror.print_usage(0);
                return;
            }
            subregion.mirror(x, y);
        }
        Some("move") => {
            let Some(ref mut subregion) = state.subregion
			else
			{
				eprintln!(r#"Command "sub move" requires an active subregion (see "sub help")"#);
				return;
			};
            let dx = parse_num!(SubCommand::Move, tokens, "dx", i16);
            let dy = parse_num!(SubCommand::Move, tokens, "dy", i16);
            if tokens.remainder().is_some() {
                eprintln!(r#"Too many parameters for "sub move""#);
                SubCommand::Move.print_usage(0);
                return;
            }
            if dx != 0 && dy != 0 {
                if let Err(e) =
                    subregion.resize(dx, dy, subregion.get_width(), subregion.get_height())
                {
                    match e {
                        ResizeError::XOffset { dx, old_w, new_w } => {
                            debug_assert_eq!(old_w, new_w);
                            eprintln!("Invalid horizontal move {dx} not in ]-{old_w}, {new_w}[");
                        }
                        ResizeError::YOffset { dy, old_h, new_h } => {
                            debug_assert_eq!(old_h, new_h);
                            eprintln!("Invalid vertical move {dy} not in ]-{old_h}, {new_h}[");
                        }
                        ResizeError::Truncated {
                            right,
                            top,
                            left,
                            bottom,
                        } => {
                            eprint!("Move would truncate subregion: ");
                            let mut first = true;
                            if right > 0 {
                                eprint!("{}right: {right}", if first { "" } else { ", " });
                                first = false;
                            }
                            if top > 0 {
                                eprint!("{}top: {top}", if first { "" } else { ", " });
                                first = false;
                            }
                            if left > 0 {
                                eprint!("{}left: {left}", if first { "" } else { ", " });
                                first = false;
                            }
                            if bottom > 0 {
                                eprint!("{}bottom: {bottom}", if first { "" } else { ", " });
                                first = false;
                            }
                            if first {
                                eprintln!("<unknown>");
                            } else {
                                eprintln!();
                            }
                        }
                        _ => print_err!(e, "Unexpected resize error (for {dx} / {dy})"),
                    }
                }
            }
        }
        Some("resize") => {
            let Some(ref mut subregion) = state.subregion
			else
			{
				eprintln!(r#"Command "sub resize" requires an active subregion (see "sub help")"#);
				return;
			};
            let w = parse_num!(SubCommand::Resize, tokens, "width", u16);
            if w == 0 {
                eprintln!("Subregion width must be positive");
                return;
            }
            let h = parse_num!(SubCommand::Resize, tokens, "height", u16);
            if h == 0 {
                eprintln!("Subregion height must be positive");
                return;
            }
            let (dx, dy) = if let arg @ Some(..) = tokens.next() {
                let dx = parse_num!(SubCommand::Resize, "dx", <i16>::from(arg));
                let dy = parse_num!(SubCommand::Resize, tokens, "dy", i16);
                (dx, dy)
            } else {
                (0, 0)
            };
            if tokens.remainder().is_some() {
                eprintln!(r#"Too many parameters for "sub resize""#);
                SubCommand::Resize.print_usage(0);
                return;
            }
            if w != subregion.get_width() || h != subregion.get_height() || dx != 0 || dy != 0 {
                if let Err(e) = subregion.resize(dx, dy, w, h) {
                    print_err!(e, "Could not resize subregion");
                }
            }
        }
        Some("remove") => {
            let Some(ref mut subregion) = state.subregion
			else
			{
				eprintln!(r#"Command "sub remove" requires an active subregion (see "sub help")"#);
				return;
			};
            let x0 = parse_num!(SubCommand::Remove, tokens, "x0", u16);
            let y0 = parse_num!(SubCommand::Remove, tokens, "y0", u16);
            if x0 >= subregion.get_width() || y0 >= subregion.get_height() {
                eprintln!(
                    "Invalid coordinate ({x0} / {y0}) out of bounds ({} / {})",
                    subregion.get_width(),
                    subregion.get_height()
                );
                return;
            }
            let (x0, y0, x1, y1) = if let arg @ Some(..) = tokens.next() {
                let x1 = parse_num!(SubCommand::Remove, "x1", <u16>::from(arg));
                let y1 = parse_num!(SubCommand::Remove, tokens, "y1", u16);
                if x1 >= subregion.get_width() || y1 >= subregion.get_height() {
                    eprintln!(
                        "Invalid coordinate ({x1} / {y1}) out of bounds ({} / {})",
                        subregion.get_width(),
                        subregion.get_height()
                    );
                    return;
                }
                (x0.min(x1), y0.min(y1), x0.max(x1), y0.max(y1))
            } else {
                (x0, y0, x0, y0)
            };
            if tokens.remainder().is_some() {
                eprintln!(r#"Too many parameters for "sub remove""#);
                SubCommand::Remove.print_usage(0);
                return;
            }
            if x1 > x0 || y1 > y0 {
                let mut cnt = 0u32;
                for y in y0..=y1 {
                    for x in x0..=x1 {
                        // position was already checked while parsing
                        if subregion.take(x, y).unwrap().is_some() {
                            cnt += 1;
                        }
                    }
                }
                println!("Removed {cnt} blocks in {x0} / {y0} to {x1} / {y1}");
            } else {
                // position was already checked while parsing
                match subregion.take(x0, y0).unwrap() {
                    None => (),
                    Some(p) => println!(
                        "Removed block {} from {x0} / {y0}",
                        p.get_block().get_name()
                    ),
                }
            }
        }
        Some("print") => {
            let Some(ref subregion) = state.subregion
			else
			{
				eprintln!(r#"Command "sub print" requires an active subregion (see "sub help")"#);
				return;
			};
            if tokens.remainder().is_some() {
                eprintln!(r#"Too many parameters for "sub print""#);
                SubCommand::Print.print_usage(0);
                return;
            }
            print_schematic(subregion);
        }
        Some("dump") => {
            let Some(ref subregion) = state.subregion
			else
			{
				eprintln!(r#"Command "sub dump" requires an active subregion (see "sub help")"#);
				return;
			};
            if tokens.remainder().is_some() {
                eprintln!(r#"Too many parameters for "sub dump""#);
                SubCommand::Dump.print_usage(0);
                return;
            }
            let b64 = match SchematicSerializer(state.reg).serialize_base64(subregion) {
                Ok(b64) => b64,
                Err(e) => {
                    print_err!(e, "Could not serialize subregion");
                    return;
                }
            };
            println!("Subregion: {}", b64);
        }
        Some(unknown) => eprintln!("Unknown command \"sub\" {unknown:?}"),
    }
}
