use std::borrow::Cow;
use std::env::Args;
use std::io::{self, Write};
use std::fs;

use crate::args::{ArgCount, ArgOption, OptionError, OptionHandler};
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
	let opt_file = handler.add(ArgOption::new(Some('f'), Some(Cow::Borrowed("file")), ArgCount::Required(usize::MAX))).unwrap();
	let opt_interact = handler.add(ArgOption::new(Some('i'), Some(Cow::Borrowed("interactive")), ArgCount::Forbidden)).unwrap();
	match args::parse(&mut args, &mut handler)
	{
		Err(args::Error::Handler{pos, val: OptionError::NoSuchShort(short)}) =>
		{
			println!("Invalid argument \"-{short}\" (at #{})).", pos + 1);
			return;
		},
		Err(args::Error::Handler{pos, val: OptionError::NoSuchLong(long)}) =>
		{
			println!("Invalid argument \"--{long}\" (at #{})).", pos + 1);
			return;
		},
		Err(args::Error::Handler{pos, val: OptionError::ValueForbidden(opt)}) =>
		{
			match (opt.get_short(), opt.get_long())
			{
				(None, None) => unreachable!("unnamed ArgOption (at #{}))", pos + 1),
				(None, Some(long)) => println!("Illegal valued argument \"--{long}\" (at #{})).", pos + 1),
				(Some(short), None) => println!("Illegal valued argument \"-{short}\" (at #{})).", pos + 1),
				(Some(short), Some(long)) => println!("Illegal valued argument \"--{long}\" (\"-{short}\", at #{})).", pos + 1),
			}
			return;
		},
		Err(args::Error::Handler{pos, val: OptionError::ValueRequired(opt)}) =>
		{
			match (opt.get_short(), opt.get_long())
			{
				(None, None) => unreachable!("unnamed ArgOption (at #{}))", pos + 1),
				(None, Some(long)) => println!("Missing value to argument \"--{long}\" (at #{})).", pos + 1),
				(Some(short), None) => println!("Missing value to argument \"-{short}\" (at #{})).", pos + 1),
				(Some(short), Some(long)) => println!("Missing value to argument \"--{long}\" (\"-{short}\", at #{})).", pos + 1),
			}
			return;
		},
		Err(args::Error::Handler{pos, val: OptionError::TooMany(opt)}) =>
		{
			let max = opt.get_count().get_max_count().unwrap();
			match (opt.get_short(), opt.get_long())
			{
				(None, None) => unreachable!("unnamed ArgOption (at #{}))", pos + 1),
				(None, Some(long)) => println!("Duplicate argument \"--{long}\" (more than {max} at #{})).", pos + 1),
				(Some(short), None) => println!("Duplicate argument \"-{short}\" (more than {max} at #{})).", pos + 1),
				(Some(short), Some(long)) => println!("Duplicate argument \"--{long}\" (\"-{short}\", more than {max} at #{})).", pos + 1),
			}
			return;
		},
		Err(args::Error::EmptyName{pos}) =>
		{
			println!("Invalid empty argument (at #{}).", pos + 1);
			return;
		},
		_ => (),
	}
	
	let reg = block::build_registry();
	let mut ss = SchematicSerializer(&reg);
	let mut first = true;
	let mut need_space = false;
	// process the files if any
	let file = match handler.get_value(opt_file).get_values()
	{
		None => false,
		Some(paths) =>
		{
			for path in paths
			{
				match fs::read(path)
				{
					Ok(data) =>
					{
						match ss.deserialize(&mut DataRead::new(&data))
						{
							Ok(s) =>
							{
								if !first || need_space {println!();}
								first = false;
								need_space = true;
								println!("Schematic: @{path}");
								print_schematic(&s);
							},
							// continue processing files, literals & maybe interactive mode
							Err(e) =>
							{
								if need_space {println!();}
								first = false;
								need_space = false;
								println!("Could not read schematic: {e:?}");
							},
						}
					},
					// continue processing files, literals & maybe interactive mode
					Err(e) =>
					{
						if need_space {println!();}
						first = false;
						need_space = false;
						println!("Could not read file {path:?}: {e}");
					},
				}
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
				if !first || need_space {println!();}
				first = false;
				need_space = true;
				println!("Schematic: {curr}");
				print_schematic(&s);
			},
			// continue processing literals & maybe interactive mode
			Err(e) =>
			{
				if need_space {println!();}
				first = false;
				need_space = false;
				println!("Could not read schematic: {e:?}");
			},
		}
	}
	// if --interactive or no schematics: continue parsing from console
	if handler.get_value(opt_interact).is_present() || (!file && handler.get_literals().is_empty())
	{
		if need_space {println!();}
		need_space = false;
		println!("Entering interactive mode, paste schematic to print details.");
		let mut buff = String::new();
		let stdin = io::stdin();
		loop
		{
			buff.clear();
			if need_space {println!();}
			need_space = false;
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
						Ok(s) =>
						{
							println!();
							need_space = true;
							print_schematic(&s)
						},
						// continue interactive mode, typos are especially likely here
						Err(e) =>
						{
							if need_space {println!();}
							need_space = false;
							println!("Could not read schematic: {e:?}")
						},
					}
				},
				Err(e) =>
				{
					if need_space {println!();}
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
