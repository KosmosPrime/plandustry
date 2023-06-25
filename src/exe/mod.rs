mod args;
mod draw;
mod print;

macro_rules! print_err {
	($err:expr, $($msg:tt)*) => {
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

fn main() {
    let mut args = std::env::args();
    args.next().unwrap(); // path to executable
    match args.next() {
        None => eprintln!("Not enough arguments, valid commands are: draw, print"),
        Some(s) if s == "print" => print::main(args, 1),
        Some(s) if s == "draw" => draw::main(args, 1),
        Some(s) => eprintln!("Unknown argument {s}, valid commands are: draw, print"),
    }
}
