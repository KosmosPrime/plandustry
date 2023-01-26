use std::env::Args;

pub mod args;
pub mod edit;
pub mod print;

macro_rules!print_err
{
	($err:expr, $($msg:tt)*) =>
	{
		{
			use std::error::Error;
			
			let err = $err;
			eprint!($($msg)*);
			eprintln!(": {err}");
			let mut err_ref = &err as &dyn Error;
			loop
			{
				if let Some(next) = err_ref.source()
				{
					eprintln!("\tSource: {next}");
					err_ref = next;
				}
				else {break;}
			}
		}
	};
}
pub(crate) use print_err;

pub fn main(mut args: Args)
{
	match args.next()
	{
		None => panic!("not enough arguments"),
		Some(s) if s == "edit" => edit::main(args, 1),
		Some(s) if s == "print" => print::main(args, 1),
		Some(s) => panic!("unknown argument {s}"),
	}
}
