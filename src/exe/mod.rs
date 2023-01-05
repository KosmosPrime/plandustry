use std::env::Args;

pub mod args;
pub mod print;

pub fn main(mut args: Args)
{
	match args.next()
	{
		None => panic!("not enough arguments"),
		Some(s) if s == "print" => print::main(args),
		Some(s) => panic!("unknown argument {s}"),
	}
}
