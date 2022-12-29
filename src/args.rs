pub trait ArgHandler
{
	type Error;
	
	fn on_literal(&mut self, name: &str) -> Result<(), Self::Error>;
	
	fn on_short(&mut self, name: char, value: Option<&str>) -> Result<(), Self::Error>;
	
	fn on_long(&mut self, name: &str, value: Option<&str>) -> Result<(), Self::Error>;
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Error<E>
{
	Handler(E),
	EmptyName,
}

impl<E> From<E> for Error<E>
{
	fn from(value: E) -> Self
	{
		Self::Handler(value)
	}
}

pub fn parse<I: Iterator, H: ArgHandler>(args: &mut I, handler: &mut H) -> Result<bool, Error<H::Error>>
	where I::Item: AsRef<str>
{
	for arg in args
	{
		let arg = arg.as_ref();
		if !arg.is_empty()
		{
			if arg.as_bytes()[0] == b'-'
			{
				if arg.len() >= 2 && arg.as_bytes()[1] == b'-'
				{
					if arg == "--" {return Ok(false);}
					let (name, value) = match arg.bytes().enumerate().find(|(_, b)| *b == b'=')
					{
						None => (&arg[2..], None),
						Some((i, _)) => (&arg[2..i], Some(&arg[i + 1..])),
					};
					if name.is_empty() {return Err(Error::EmptyName);}
					handler.on_long(name, value)?;
				}
				else
				{
					let (value, end) = match arg.bytes().enumerate().find(|(_, b)| *b == b'=')
					{
						None => (None, arg.len()),
						Some((i, _)) => (Some(&arg[i + 1..]), i),
					};
					if end > 2 || (end == 2 && value.is_none())
					{
						for c in arg[1..end].chars()
						{
							handler.on_short(c, value)?;
						}
					}
					else {return Err(Error::EmptyName);}
				}
			}
			else
			{
				handler.on_literal(arg)?;
			}
		}
	}
	Ok(true)
}

pub fn parse_args<H: ArgHandler>(handler: &mut H) -> Result<(), Error<H::Error>>
{
	parse(&mut std::env::args(), handler)?;
	Ok(())
}
