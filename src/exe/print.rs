use std::borrow::Cow;
use std::env::Args;
use std::io::{self, Write};
use std::fs;

use crate::block::build_registry;
use crate::data::{DataRead, Serializer};
use crate::data::schematic::{Schematic, SchematicSerializer};
use crate::exe::print_err;
use crate::exe::args::{self, ArgCount, ArgOption, OptionHandler};

pub fn main(mut args: Args, arg_off: usize)
{
	let mut handler = OptionHandler::new();
	let opt_file = handler.add(ArgOption::new(Some('f'), Some(Cow::Borrowed("file")), ArgCount::Required(usize::MAX))).unwrap();
	let opt_interact = handler.add(ArgOption::new(Some('i'), Some(Cow::Borrowed("interactive")), ArgCount::Forbidden)).unwrap();
	if let Err(e) = args::parse(&mut args, &mut handler, arg_off)
	{
		print_err!(e, "Command error");
		return;
	}
	
	let reg = build_registry();
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
								print_err!(e, "Could not read schematic from {path}");
							},
						}
					},
					// continue processing files, literals & maybe interactive mode
					Err(e) =>
					{
						if need_space {println!();}
						first = false;
						need_space = false;
						print_err!(e, "Could not read file {path:?}");
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
				print_err!(e, "Could not read schematic");
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
							print_err!(e, "Could not read schematic");
						},
					}
				},
				Err(e) =>
				{
					if need_space {println!();}
					print_err!(e, "Failed to read next schematic");
					break;
				},
			}
		}
	}
}

pub fn print_schematic(s: &Schematic)
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
	let (cost, sandbox) = s.compute_total_cost();
	if !cost.is_empty()
	{
		println!("Build cost: {cost}{}", if sandbox {" (Sandbox only)"} else {""});
	}
	else if sandbox
	{
		println!("Can only be built in the Sandbox");
	}
	println!("\n{s}");
}
