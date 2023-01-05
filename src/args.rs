use std::borrow::Cow;
use std::collections::HashMap;

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
	Handler{pos: usize, val: E},
	EmptyName{pos: usize},
}

pub fn parse<I: Iterator, H: ArgHandler>(args: &mut I, handler: &mut H) -> Result<bool, Error<H::Error>>
	where I::Item: AsRef<str>
{
	for (pos, arg) in args.enumerate()
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
					if name.is_empty() {return Err(Error::EmptyName{pos});}
					if let Err(val) = handler.on_long(name, value)
					{
						return Err(Error::Handler{pos, val});
					}
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
							if let Err(val) = handler.on_short(c, value)
							{
								return Err(Error::Handler{pos, val});
							}
						}
					}
					else {return Err(Error::EmptyName{pos});}
				}
			}
			else
			{
				if let Err(val) = handler.on_literal(arg)
				{
					return Err(Error::Handler{pos, val});
				}
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

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ArgOption
{
	short: Option<char>,
	long: Option<Cow<'static, str>>,
}

impl ArgOption
{
	pub const fn new(short: Option<char>, long: Option<Cow<'static, str>>) -> Self
	{
		if short.is_none() && long.is_none()
		{
			panic!("option must have at least a short or long name");
		}
		Self{short, long}
	}
	
	pub fn get_short(&self) -> Option<char>
	{
		self.short
	}
	
	pub fn get_long(&self) -> Option<&str>
	{
		match self.long
		{
			None => None,
			Some(ref s) => Some(s),
		}
	}
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum OptionValue
{
	Absent,
	Present,
	Value(String),
}

impl OptionValue
{
	pub const fn is_absent(&self) -> bool
	{
		match self
		{
			OptionValue::Absent => true,
			_ => false,
		}
	}
	
	pub const fn is_present(&self) -> bool
	{
		match self
		{
			OptionValue::Present | OptionValue::Value(..) => true,
			_ => false,
		}
	}
	
	pub const fn has_value(&self) -> bool
	{
		match self
		{
			OptionValue::Value(..) => true,
			_ => false,
		}
	}
	
	pub const fn get_value(&self) -> Option<&String>
	{
		match self
		{
			OptionValue::Value(v) => Some(v),
			_ => None,
		}
	}
}

#[derive(Clone, Debug)]
pub struct OptionHandler
{
	options: Vec<(ArgOption, OptionValue)>,
	short_map: HashMap<char, usize>,
	long_map: HashMap<String, usize>,
	literals: Vec<String>,
}

impl OptionHandler
{
	pub fn new() -> Self
	{
		Self{options: Vec::new(), short_map: HashMap::new(), long_map: HashMap::new(), literals: Vec::new()}
	}
	
	pub fn add(&mut self, opt: ArgOption) -> Result<(), (ArgOption, &ArgOption)>
	{
		match opt.short
		{
			Some(c) => match self.short_map.get(&c)
			{
				Some(&i) => return Err((opt, &self.options[i].0)),
				_ => (),
			},
			_ => (),
		}
		match opt.long
		{
			Some(ref s) => match self.long_map.get(&**s)
			{
				Some(&i) => return Err((opt, &self.options[i].0)),
				_ => (),
			},
			_ => (),
		}
		
		let idx = self.options.len();
		self.options.push((opt, OptionValue::Absent));
		let opt = &self.options[idx].0;
		if let Some(c) = opt.short
		{
			self.short_map.insert(c, idx);
		}
		if let Some(ref s) = opt.long
		{
			let k = &**s;
			self.long_map.insert(k.to_owned(), idx);
		}
		Ok(())
	}
	
	pub fn options(&self) -> &Vec<(ArgOption, OptionValue)>
	{
		&self.options
	}
	
	pub fn get_short(&self, name: char) -> Option<&(ArgOption, OptionValue)>
	{
		self.short_map.get(&name).map(|&i| &self.options[i])
	}
	
	pub fn get_long(&self, name: &str) -> Option<&(ArgOption, OptionValue)>
	{
		self.long_map.get(name).map(|&i| &self.options[i])
	}
	
	pub fn get_literals(&self) -> &Vec<String>
	{
		&self.literals
	}
	
	fn set_arg(&mut self, idx: usize, value: Option<&str>) -> Result<(), OptionError>
	{
		let (ref o, ref mut v) = self.options[idx];
		if *v == OptionValue::Absent
		{
			match value
			{
				None => *v = OptionValue::Present,
				Some(s) => *v = OptionValue::Value(s.to_owned()),
			}
			Ok(())
		}
		else {Err(OptionError::Duplicate(o.clone()))}
	}
	
	pub fn clear(&mut self)
	{
		self.options.iter_mut().for_each(|(_, v)| *v = OptionValue::Absent);
	}
}

impl ArgHandler for OptionHandler
{
	type Error = OptionError;
	
	fn on_literal(&mut self, name: &str) -> Result<(), Self::Error>
	{
		self.literals.push(name.to_owned());
		Ok(())
	}
	
	fn on_short(&mut self, name: char, value: Option<&str>) -> Result<(), Self::Error>
	{
		match self.short_map.get(&name)
		{
			None => Err(OptionError::NoSuchShort(name)),
			Some(&i) => self.set_arg(i, value),
		}
	}
	
	fn on_long(&mut self, name: &str, value: Option<&str>) -> Result<(), Self::Error>
	{
		match self.long_map.get(name)
		{
			None => Err(OptionError::NoSuchLong(name.to_owned())),
			Some(&i) => self.set_arg(i, value),
		}
	}
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum OptionError
{
	NoSuchShort(char),
	NoSuchLong(String),
	Duplicate(ArgOption),
}
