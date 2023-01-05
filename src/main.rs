use std::borrow::Cow;
use std::env::Args;
use std::io::{self, Write};
use std::fs;

use crate::args::{ArgOption, OptionError, OptionHandler};
use crate::data::{DataRead, Serializer};
use crate::data::schematic::{Schematic, SchematicSerializer};

pub mod access;
pub mod args;
pub mod block;
pub mod data;
pub mod logic;

fn main()
{
	let mut args = std::env::args();
	args.next().unwrap(); // path to executable
	match args.next()
	{
		None => panic!("not enough arguments"),
		Some(s) if s == "print" => main_print(args),
		Some(s) => panic!("unknown argument {s}"),
	}
}

fn main_print(mut args: Args)
{
	let mut handler = OptionHandler::new();
	let opt_file = handler.add(ArgOption::new(Some('f'), Some(Cow::Borrowed("file")))).unwrap();
	let opt_interact = handler.add(ArgOption::new(Some('i'), Some(Cow::Borrowed("interactive")))).unwrap();
	match args::parse(&mut args, &mut handler)
	{
		Err(args::Error::Handler{pos, val: OptionError::NoSuchShort(short)}) =>
		{
			println!("Invalid argument \"-{short}\" (at #{pos})).");
			return;
		},
		Err(args::Error::Handler{pos, val: OptionError::NoSuchLong(long)}) =>
		{
			println!("Invalid argument \"--{long}\" (at #{pos})).");
			return;
		},
		Err(args::Error::Handler{pos, val: OptionError::Duplicate(opt)}) =>
		{
			match (opt.get_short(), opt.get_long())
			{
				(None, None) => unreachable!("unnamed ArgOption (at #{pos}))"),
				(None, Some(long)) => println!("Duplicate argument \"--{long}\" (at #{pos}))."),
				(Some(short), None) => println!("Duplicate argument \"-{short}\" (at #{pos}))."),
				(Some(short), Some(long)) => println!("Duplicate argument \"--{long}\" (\"-{short}\", at #{pos}))."),
			}
			return;
		},
		Err(args::Error::EmptyName{pos}) =>
		{
			println!("Invalid empty argument (at #{pos}).");
			return;
		},
		_ => (),
	}
	
	let reg = block::build_registry();
	let mut ss = SchematicSerializer(&reg);
	let mut first = true;
	// process the file if any
	let file = match handler.get_value(opt_file).get_value()
	{
		None => false,
		Some(ref path) =>
		{
			match fs::read(path)
			{
				Ok(data) =>
				{
					match ss.deserialize(&mut DataRead::new(&data))
					{
						Ok(s) =>
						{
							if first {first = false;}
							else {println!();}
							println!("Schematic: @{path}");
							print_schematic(&s);
						},
						// continue processing literals & maybe interactive mode
						Err(e) => println!("Could not read schematic: {e:?}"),
					}
				},
				// continue processing literals & maybe interactive mode
				Err(e) => println!("Could not read file {path:?}: {e}"),
			}
			true
		},
	};
	// process schematics from command line
	for curr in handler.get_literals()
	{
		match ss.deserialize_base64(curr)
		{
			Ok(s) =>
			{
				if first {first = false;}
				else {println!();}
				println!("Schematic: {curr}");
				print_schematic(&s);
			},
			// continue processing literals & maybe interactive mode
			Err(e) => println!("Could not read schematic: {e:?}"),
		}
	}
	// if --interactive or no schematics: continue parsing from console
	if handler.get_value(opt_interact).is_present() || (!file && handler.get_literals().is_empty())
	{
		println!("\nEntering interactive mode, paste schematic to print details.\n");
		let mut buff = String::new();
		let stdin = io::stdin();
		loop
		{
			buff.clear();
			print!("> ");
			if let Err(e) = io::stdout().flush()
			{
				// what the print & println macros would do
				panic!("failed printing to stdout: {e}");
			}
			match stdin.read_line(&mut buff)
			{
				Ok(..) =>
				{
					let data = buff.trim();
					if data.is_empty() {break;}
					match ss.deserialize_base64(data)
					{
						Ok(s) => print_schematic(&s),
						// continue interactive mode, typos are especially likely here
						Err(e) => println!("Could not read schematic: {e:?}"),
					}
				},
				Err(e) =>
				{
					println!("Failed to read next line: {e}");
					break;
				},
			}
		}
	}
}

fn print_schematic(s: &Schematic)
{
	if let Some(name) = s.get_tags().get("name")
	{
		if !name.is_empty() {println!("Name: {name}");}
	}
	if let Some(desc) = s.get_tags().get("description")
	{
		if !desc.is_empty() {println!("Desc: {desc}");}
	}
	if let Some(labels) = s.get_tags().get("labels")
	{
		if !labels.is_empty() && labels != "[]" {println!("Tags: {:?}", labels);}
	}
	println!("\n{s}");
}
