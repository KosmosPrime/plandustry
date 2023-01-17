pub mod access;
pub mod block;
pub mod content;
pub mod data;
pub mod exe;
pub mod fluid;
pub mod item;
pub mod logic;

fn main()
{
	let mut args = std::env::args();
	args.next().unwrap(); // path to executable
	exe::main(args);
}
