use std::any::type_name;
use std::collections::HashMap;
use std::collections::hash_map::Entry;
use std::error::Error;
use std::fmt;

pub trait RegistryEntry
{
	fn get_name(&self) -> &str;
}

pub struct Registry<'l, E: RegistryEntry + fmt::Debug + 'static>
{
	by_name: HashMap<&'l str, &'l E>,
}

impl<'l, E: RegistryEntry + fmt::Debug + 'static> Registry<'l, E>
{
	pub fn new() -> Self
	{
		Self{by_name: HashMap::new()}
	}
	
	pub fn register(&mut self, val: &'l E) -> Result<&'l E, RegisterError<'l, E>>
	{
		match self.by_name.entry(&val.get_name())
		{
			Entry::Occupied(e) => Err(RegisterError(e.get())),
			Entry::Vacant(e) => Ok(e.insert(val)),
		}
	}
	
	pub fn get(&self, name: &str) -> Option<&'l E>
	{
		self.by_name.get(name).map(|&r| r)
	}
}

#[derive(Clone, Copy, Debug)]
pub struct RegisterError<'l, E: RegistryEntry + fmt::Debug + 'static>(pub &'l E);

impl<'l, E: RegistryEntry + fmt::Debug + 'static> fmt::Display for RegisterError<'l, E>
{
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result
	{
		write!(f, "{} {:?} already exists", type_name::<E>(), self.0.get_name())
	}
}

impl<'l, E: RegistryEntry + fmt::Debug + 'static> Error for RegisterError<'l, E> {}
