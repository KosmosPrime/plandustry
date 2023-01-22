use std::error::Error;
use std::fmt;
use std::iter::{Enumerate, FusedIterator};
use std::slice;

use crate::item;

#[derive(Clone, Debug, Eq)]
pub struct Storage
{
	base: Vec<u32>,
	total: u64,
}

impl Storage
{
	pub const fn new() -> Self
	{
		Self{base: Vec::new(), total: 0}
	}
	
	pub fn is_empty(&self) -> bool
	{
		self.total == 0
	}
	
	pub fn get_total(&self) -> u64
	{
		self.total
	}
	
	pub fn get(&self, ty: item::Type) -> u32
	{
		match self.base.get(u16::from(ty) as usize)
		{
			None => 0,
			Some(cnt) => *cnt,
		}
	}
	
	pub fn set(&mut self, ty: item::Type, count: u32) -> u32
	{
		let idx = u16::from(ty) as usize;
		match self.base.get_mut(idx)
		{
			None =>
			{
				self.base.resize(idx + 1, 0);
				self.base[idx] = count;
				self.total += count as u64;
				0
			},
			Some(curr) =>
			{
				let prev = *curr;
				self.total = self.total - prev as u64 + count as u64;
				*curr = count;
				prev
			},
		}
	}
	
	pub fn add(&mut self, ty: item::Type, add: u32, max: u32) -> (u32, u32)
	{
		let idx = u16::from(ty) as usize;
		match self.base.get_mut(idx)
		{
			None =>
			{
				let actual = add.min(max);
				self.base.resize(idx + 1, 0);
				self.base[idx] = actual;
				self.total += add as u64;
				(actual, actual)
			}
			Some(curr) =>
			{
				if *curr < max
				{
					let actual = add.min(max - *curr);
					*curr += actual;
					self.total += actual as u64;
					(actual, *curr)
				}
				else {(0, *curr)}
			},
		}
	}
	
	pub fn try_add(&mut self, ty: item::Type, add: u32, max: u32) -> Result<(u32, u32), TryAddError>
	{
		let idx = u16::from(ty) as usize;
		match self.base.get_mut(idx)
		{
			None =>
			{
				if add <= max
				{
					self.base.resize(idx + 1, 0);
					self.base[idx] = add;
					self.total += add as u64;
					Ok((add, add))
				}
				else {Err(TryAddError{ty, have: 0, add, max})}
			},
			Some(curr) =>
			{
				if *curr <= max && max - *curr <= add
				{
					*curr += add;
					self.total += add as u64;
					Ok((add, *curr))
				}
				else {Err(TryAddError{ty, have: *curr, add, max})}
			},
		}
	}
	
	pub fn sub(&mut self, ty: item::Type, sub: u32, min: u32) -> (u32, u32)
	{
		match self.base.get_mut(u16::from(ty) as usize)
		{
			None => (0, 0),
			Some(curr) =>
			{
				if *curr > min
				{
					let actual = sub.min(*curr - min);
					*curr -= actual;
					self.total -= actual as u64;
					(actual, *curr)
				}
				else {(0, *curr)}
			},
		}
	}
	
	pub fn try_sub(&mut self, ty: item::Type, sub: u32, min: u32) -> Result<(u32, u32), TrySubError>
	{
		let idx = u16::from(ty) as usize;
		match self.base.get_mut(idx)
		{
			None => Err(TrySubError{ty, have: 0, sub, min}),
			Some(curr) =>
			{
				if *curr >= min && *curr - min >= sub
				{
					*curr -= sub;
					self.total -= sub as u64;
					Ok((sub, *curr))
				}
				else {Err(TrySubError{ty, have: *curr, sub, min})}
			},
		}
	}
	
	pub fn add_all(&mut self, other: &Storage, max_each: u32) -> (u64, u64)
	{
		let mut added = 0u64;
		if max_each > 0 && other.get_total() > 0
		{
			let mut iter = other.base.iter().enumerate();
			// resize our vector only once and if necessary
			let (last, add_last) = iter.rfind(|(_, n)| **n != 0).unwrap();
			if self.base.len() <= last
			{
				self.base.resize(last + 1, 0);
			}
			// process items by increasing ID
			for (idx, add) in iter
			{
				let curr = self.base[idx];
				if curr < max_each
				{
					let actual = (*add).min(max_each - curr);
					self.base[idx] += actual;
					added += actual as u64;
				}
			}
			// process the final element (which we've retrieved first)
			let curr = self.base[last];
			if curr < max_each
			{
				let actual = (*add_last).min(max_each - curr);
				self.base[last] += actual;
				added += actual as u64;
			}
			// update total
			self.total += added;
		}
		(added, self.total)
	}
	
	pub fn pull_all(&mut self, other: &mut Storage, max_each: u32) -> (u64, u64, u64)
	{
		let mut added = 0u64;
		if max_each > 0 && other.get_total() > 0
		{
			let mut iter = other.base.iter_mut().enumerate();
			// resize our vector only once and if necessary
			let (last, add_last) = iter.rfind(|(_, n)| **n != 0).unwrap();
			if self.base.len() <= last
			{
				self.base.resize(last + 1, 0);
			}
			// process items by increasing ID
			for (idx, add) in iter
			{
				let curr = self.base[idx];
				if curr < max_each
				{
					let actual = (*add).min(max_each - curr);
					self.base[idx] += actual;
					*add -= actual;
					added += actual as u64;
				}
			}
			// process the final element (which we've retrieved first)
			let curr = self.base[last];
			if curr < max_each
			{
				let actual = (*add_last).min(max_each - curr);
				self.base[last] += actual;
				*add_last -= actual;
				added += actual as u64;
			}
			// update totals
			self.total += added;
			other.total -= added;
		}
		(added, self.total, other.total)
	}
	
	pub fn sub_all(&mut self, other: &Storage, min_each: u32) -> (u64, u64)
	{
		let mut subbed = 0u64;
		if self.get_total() > 0 && other.get_total() > 0
		{
			// no need for resizing, we only remove
			// process items by increasing ID
			for (idx, sub) in other.base.iter().enumerate()
			{
				if let Some(curr) = self.base.get(idx)
				{
					if *curr > min_each
					{
						let actual = (*sub).min(*curr - min_each);
						self.base[idx] -= actual;
						subbed += actual as u64;
					}
				}
				else {break;}
			}
			// update total
			self.total -= subbed;
		}
		(subbed, self.total)
	}
	
	pub fn diff_all(&mut self, other: &mut Storage, min_each: u32) -> (u64, u64, u64)
	{
		let mut subbed = 0u64;
		if self.get_total() > 0 && other.get_total() > 0
		{
			// no need for resizing, we only remove
			// consider only indexes present in both
			let end = self.base.len().min(other.base.len());
			let lhs = &mut self.base[..end];
			let rhs = &mut other.base[..end];
			// process items by increasing ID
			for (l, r) in lhs.into_iter().zip(rhs)
			{
				if *l > min_each && *r > min_each
				{
					let actual = (*l - min_each).min(*r - min_each);
					*l -= actual;
					*r -= actual;
					subbed -= actual as u64;
				}
			}
			// update totals
			self.total -= subbed;
			other.total -= subbed;
		}
		(subbed, self.total, other.total)
	}
	
	pub fn iter<'s>(&'s self) -> Iter<'s>
	{
		Iter{base: self.base.iter().enumerate(), all: true}
	}
	
	pub fn iter_nonzero<'s>(&'s self) -> Iter<'s>
	{
		Iter{base: self.base.iter().enumerate(), all: false}
	}
	
	pub fn clear(&mut self)
	{
		self.base.clear()
	}
}

// manual because padding with zeros doesn't affect equality
impl PartialEq for Storage
{
	fn eq(&self, other: &Self) -> bool
	{
		let mut li = self.base.iter().fuse();
		let mut ri = other.base.iter().fuse();
		loop
		{
			match (li.next(), ri.next())
			{
				(None, None) => return true,
				(l, r) =>
				{
					if l.unwrap_or(&0) != r.unwrap_or(&0) {return false;}
				},
			}
		}
	}
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct TryAddError
{
	pub ty: item::Type,
	pub have: u32,
	pub add: u32,
	pub max: u32
}

impl fmt::Display for TryAddError
{
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result
	{
		write!(f, "adding {} {:?} to current {} would exceed {}", self.add, self.ty, self.have, self.max)
	}
}

impl Error for TryAddError {}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct TrySubError
{
	pub ty: item::Type,
	pub have: u32,
	pub sub: u32,
	pub min: u32
}

impl fmt::Display for TrySubError
{
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result
	{
		write!(f, "removing {} {:?} from current {} would drop below {}", self.sub, self.ty, self.have, self.min)
	}
}

impl Error for TrySubError {}

#[derive(Clone, Debug)]
pub struct Iter<'l>
{
	base: Enumerate<slice::Iter<'l, u32>>,
	all: bool,
}

impl<'l> Iterator for Iter<'l>
{
	type Item = (item::Type, u32);
	
	fn next(&mut self) -> Option<Self::Item>
	{
		while let Some((idx, cnt)) = self.base.next()
		{
			if *cnt > 0 || self.all
			{
				if let Ok(ty) = item::Type::try_from(idx as u16)
				{
					return Some((ty, *cnt));
				}
			}
		}
		None
	}
	
	fn size_hint(&self) -> (usize, Option<usize>)
	{
		(0, self.base.size_hint().1)
	}
}

impl<'l> FusedIterator for Iter<'l> where Enumerate<slice::Iter<'l, u32>>: FusedIterator {}
