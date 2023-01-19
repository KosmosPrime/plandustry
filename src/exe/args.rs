use std::borrow::Cow;
use std::collections::HashMap;
use std::error;
use std::fmt;
use std::slice::from_ref;

pub trait ArgHandler
{
	type Error: error::Error + 'static;
	
	fn on_literal(&mut self, name: &str) -> Result<(), Self::Error>;
	
	fn on_short(&mut self, name: char, value: Option<&str>) -> Result<(), Self::Error>;
	
	fn on_long(&mut self, name: &str, value: Option<&str>) -> Result<(), Self::Error>;
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Error<E: error::Error + 'static>
{
	Handler{pos: usize, val: E},
	EmptyName{pos: usize},
}

impl<E: error::Error + 'static> fmt::Display for Error<E>
{
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result
	{
		match self
		{
			Error::Handler{pos, val} => write!(f, "{val} (at #{pos})"),
			Error::EmptyName{pos} => write!(f, "Malformed argument (at #{pos})"),
		}
	}
}

impl<E: error::Error + 'static> error::Error for Error<E>
{
	fn source(&self) -> Option<&(dyn error::Error + 'static)>
	{
		match self
		{
			Error::Handler{pos: _, val} => Some(val),
			_ => None,
		}
	}
}

pub fn parse<I: Iterator, H: ArgHandler>(args: &mut I, handler: &mut H, arg_off: usize) -> Result<bool, Error<H::Error>>
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
					if name.is_empty() {return Err(Error::EmptyName{pos: arg_off + pos});}
					if let Err(val) = handler.on_long(name, value)
					{
						return Err(Error::Handler{pos: arg_off + pos, val});
					}
				}
				else
				{
					let (value, end) = match arg.bytes().enumerate().find(|(_, b)| *b == b'=')
					{
						None => (None, arg.len()),
						Some((i, _)) => (Some(&arg[i + 1..]), i),
					};
					if end > 1
					{
						for c in arg[1..end].chars()
						{
							if let Err(val) = handler.on_short(c, value)
							{
								return Err(Error::Handler{pos: arg_off + pos, val});
							}
						}
					}
					else {return Err(Error::EmptyName{pos: arg_off + pos});}
				}
			}
			else
			{
				if let Err(val) = handler.on_literal(arg)
				{
					return Err(Error::Handler{pos: arg_off + pos, val});
				}
			}
		}
	}
	Ok(true)
}

pub fn parse_args<H: ArgHandler>(handler: &mut H) -> Result<(), Error<H::Error>>
{
	parse(&mut std::env::args(), handler, 0)?;
	Ok(())
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ArgCount
{
	Forbidden,
	Optional(usize),
	Required(usize),
}

impl ArgCount
{
	pub const fn has_value(&self) -> bool
	{
		match self
		{
			ArgCount::Optional(..) | ArgCount::Required(..) => true,
			_ => false
		}
	}
	
	pub const fn is_required(&self) -> bool
	{
		match self
		{
			ArgCount::Required(..) => true,
			_ => false
		}
	}
	
	pub const fn get_max_count(&self) -> Option<usize>
	{
		match self
		{
			ArgCount::Optional(max) | ArgCount::Required(max) => Some(*max),
			_ => None,
		}
	}
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ArgOption
{
	short: Option<char>,
	long: Option<Cow<'static, str>>,
	count: ArgCount,
}

impl ArgOption
{
	pub const fn new(short: Option<char>, long: Option<Cow<'static, str>>, count: ArgCount) -> Self
	{
		if short.is_none() && long.is_none()
		{
			panic!("option must have at least a short or long name");
		}
		if let Some(max) = count.get_max_count()
		{
			if max == 0
			{
				panic!("argument must be allowed to appear at least once");
			}
		}
		Self{short, long, count}
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
			Some(Cow::Borrowed(r)) => Some(r),
			Some(Cow::Owned(ref s)) => Some(s.as_str()),
		}
	}
	
	pub const fn get_count(&self) -> &ArgCount
	{
		&self.count
	}
}

impl fmt::Display for ArgOption
{
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result
	{
		match (self.get_short(), self.get_long())
		{
			(None, None) => unreachable!("unnamed ArgOption"),
			(None, Some(long)) => write!(f, "\"--{long}\""),
			(Some(short), None) => write!(f, "\"-{short}\""),
			(Some(short), Some(long)) => write!(f, "\"--{long}\" / \"-{short}\""),
		}
	}
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum OptionValue
{
	Absent,
	Present,
	Value(String),
	Values(Vec<String>),
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
			OptionValue::Present | OptionValue::Value(..) | OptionValue::Values(..) => true,
			_ => false,
		}
	}
	
	pub const fn has_value(&self) -> bool
	{
		match self
		{
			OptionValue::Value(..) => true,
			OptionValue::Values(..) => true,
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
	
	pub fn get_values(&self) -> Option<&[String]>
	{
		match self
		{
			OptionValue::Value(v) => Some(from_ref(v)),
			OptionValue::Values(v) => Some(v.as_ref()),
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
	
	pub fn add(&mut self, opt: ArgOption) -> Result<OptionRef, AddArgError>
	{
		match opt.short
		{
			Some(c) => match self.short_map.get(&c)
			{
				Some(&i) => return Err(AddArgError{to_add: opt, existing: &self.options[i].0}),
				_ => (),
			},
			_ => (),
		}
		match opt.long
		{
			Some(ref s) => match self.long_map.get(&**s)
			{
				Some(&i) => return Err(AddArgError{to_add: opt, existing: &self.options[i].0}),
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
		Ok(OptionRef(idx))
	}
	
	pub fn options(&self) -> &Vec<(ArgOption, OptionValue)>
	{
		&self.options
	}
	
	pub fn get(&self, opt_ref: OptionRef) -> (&ArgOption, &OptionValue)
	{
		let opt = &self.options[opt_ref.0];
		(&opt.0, &opt.1)
	}
	
	pub fn get_option(&self, opt_ref: OptionRef) -> &ArgOption
	{
		&self.options[opt_ref.0].0
	}
	
	pub fn get_value(&self, opt_ref: OptionRef) -> &OptionValue
	{
		&self.options[opt_ref.0].1
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
		let (ref o, ref mut curr) = self.options[idx];
		match o.count
		{
			ArgCount::Forbidden =>
			{
				if let None = value
				{
					if curr.is_absent() {*curr = OptionValue::Present;}
					Ok(())
				}
				else {Err(OptionError::ValueForbidden(o.clone()))}
			},
			ArgCount::Optional(max) =>
			{
				match curr
				{
					OptionValue::Absent | OptionValue::Present =>
					{
						if let Some(v) = value
						{
							if max == 1 {*curr = OptionValue::Value(v.to_owned());}
							else {*curr = OptionValue::Values(vec![v.to_owned()]);}
						}
						else {*curr = OptionValue::Present;}
						Ok(())
					},
					OptionValue::Value(..) => Err(OptionError::TooMany(o.clone())),
					OptionValue::Values(vec) =>
					{
						if vec.len() <= max
						{
							if let Some(v) = value
							{
								vec.push(v.to_owned());
							}
							Ok(())
						}
						else {Err(OptionError::TooMany(o.clone()))}
					},
				}
			},
			ArgCount::Required(max) =>
			{
				if let Some(v) = value
				{
					match curr
					{
						OptionValue::Absent =>
						{
							if max == 1 {*curr = OptionValue::Value(v.to_owned());}
							else {*curr = OptionValue::Values(vec![v.to_owned()]);}
							Ok(())
						},
						OptionValue::Present => unreachable!("argument missing required value"),
						OptionValue::Value(..) => Err(OptionError::TooMany(o.clone())),
						OptionValue::Values(vec) =>
						{
							if vec.len() <= max
							{
								vec.push(v.to_owned());
								Ok(())
							}
							else {Err(OptionError::TooMany(o.clone()))}
						},
					}
				}
				else {Err(OptionError::ValueRequired(o.clone()))}
			},
		}
	}
	
	pub fn clear(&mut self)
	{
		self.options.iter_mut().for_each(|(_, v)| *v = OptionValue::Absent);
	}
}

#[derive(Clone, Copy, Debug)]
pub struct OptionRef(usize);

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AddArgError<'l>
{
	pub to_add: ArgOption,
	pub existing: &'l ArgOption,
}

impl<'l> fmt::Display for AddArgError<'l>
{
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result
	{
		write!(f, "Duplicate argument {} (already have {})", self.to_add, self.existing)
	}
}

impl<'l> error::Error for AddArgError<'l> {}

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
	ValueForbidden(ArgOption),
	ValueRequired(ArgOption),
	TooMany(ArgOption),
}

impl fmt::Display for OptionError
{
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result
	{
		match self
		{
			OptionError::NoSuchShort(short) => write!(f, "Invalid argument \"-{short}\""),
			OptionError::NoSuchLong(long) => write!(f, "Invalid argument \"--{long}\""),
			OptionError::ValueForbidden(opt) => write!(f, "Argument {opt} has no value"),
			OptionError::ValueRequired(opt) => write!(f, "Argument {opt} requires a value"),
			OptionError::TooMany(opt) =>
			{
				if let Some(max) = opt.count.get_max_count() {write!(f, "Too many {opt} (max {max})")}
				else {write!(f, "Duplicate argument {opt}")}
			},
		}
	}
}

impl error::Error for OptionError {}
