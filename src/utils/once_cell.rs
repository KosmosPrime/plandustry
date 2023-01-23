use std::cell::UnsafeCell;
use std::mem::MaybeUninit;
use std::ptr;
use std::sync::atomic::{AtomicU8, Ordering};

const STATE_INIT: u8 = 0;
const STATE_LOCKED: u8 = STATE_INIT + 1;
const STATE_READY: u8 = STATE_LOCKED + 1;

pub struct OnceCell<T>
{
	value: UnsafeCell<MaybeUninit<T>>,
	state: AtomicU8,
}

impl<T> OnceCell<T>
{
	pub const fn new() -> Self
	{
		Self{value: UnsafeCell::new(MaybeUninit::uninit()), state: AtomicU8::new(STATE_INIT)}
	}
	
	pub const fn new_init(value: T) -> Self
	{
		Self{value: UnsafeCell::new(MaybeUninit::new(value)), state: AtomicU8::new(STATE_READY)}
	}
	
	pub fn get(&self) -> Option<&T>
	{
		if self.state.load(Ordering::Acquire) == STATE_READY
		{
			// SAFETY: won't be overwritten for the lifetime of this reference
			Some(unsafe{(&*self.value.get()).assume_init_ref()})
		}
		else {None}
	}
	
	pub fn get_or_wait(&self) -> Option<&T>
	{
		loop
		{
			match self.state.load(Ordering::Acquire)
			{
				STATE_INIT => return None,
				STATE_LOCKED => (), // continue
				STATE_READY => return Some(unsafe{(&*self.value.get()).assume_init_ref()}),
				x => unreachable!("invalid state {x}"),
			}
		}
	}
	
	pub fn get_or_init<F: Fn() -> T>(&self, init: F) -> &T
	{
		loop
		{
			match self.state.compare_exchange(STATE_INIT, STATE_LOCKED, Ordering::AcqRel, Ordering::Acquire)
			{
				Ok(..) =>
				{
					let value = init();
					let written = &*unsafe{&mut *self.value.get()}.write(value);
					self.state.store(STATE_READY, Ordering::Release);
					return written;
				},
				Err(STATE_READY) => return unsafe{(&*self.value.get()).assume_init_ref()},
				Err(..) => (), // locked or spurious failure
			}
		}
	}
	
	pub fn set(&self, value: T) -> Result<&T, T>
	{
		// don't set state to STATE_READY on success because we have to release afterward anyway
		match self.state.compare_exchange(STATE_INIT, STATE_LOCKED, Ordering::AcqRel, Ordering::Acquire)
		{
			Ok(..) =>
			{
				// SAFETY: unique because only one thread can lock the atomic state
				let written = &*unsafe{&mut *self.value.get()}.write(value);
				self.state.store(STATE_READY, Ordering::Release);
				Ok(written)
			},
			// SAFETY: guaranteed to be initialized & protected by acquire ordering
			Err(STATE_READY) => return Ok(unsafe{(&*self.value.get()).assume_init_ref()}),
			Err(..) => Err(value), // locked or spurious failure
		}
	}
	
	pub fn set_mut(&mut self, mut value: T) -> Result<Option<T>, T>
	{
		// don't set state to STATE_READY on success because we have to release afterward anyway
		match self.state.compare_exchange(STATE_INIT, STATE_LOCKED, Ordering::AcqRel, Ordering::Acquire)
		{
			Ok(..) =>
			{
				self.value.get_mut().write(value);
				self.state.store(STATE_READY, Ordering::Release);
				Ok(None)
			},
			Err(STATE_READY) =>
			{
				// SAFETY: guaranteed to be initialized & protected by acquire ordering
				std::mem::swap(unsafe{self.value.get_mut().assume_init_mut()}, &mut value);
				// ensure changes are visible to others acquiring the atomic state
				self.state.store(STATE_READY, Ordering::Release);
				// we've swapped the previous value into this variable
				Ok(Some(value))
			},
			Err(..) => Err(value), // locked or spurious failure
		}
	}
	
	pub fn into_inner(mut self) -> Option<T>
	{
		// must be atomic so potential writes during the drop see a valid state
		let inner = match self.state.load(Ordering::Acquire)
		{
			STATE_INIT => None,
			STATE_LOCKED => unreachable!("consumed cell during initialization"),
			// SAFETY: initialized & we'll forget about it afterwards
			STATE_READY => Some(unsafe{self.value.get_mut().assume_init_read()}),
			x => unreachable!("invalid state {x}"),
		};
		// SAFETY: just in case AtomicU8 has a drop handler
		unsafe{ptr::drop_in_place(&mut self.state as *mut _);}
		std::mem::forget(self);
		inner
	}
}

impl<T> Default for OnceCell<T>
{
	fn default() -> Self
	{
		OnceCell::new()
	}
}

impl<T> From<T> for OnceCell<T>
{
	fn from(value: T) -> Self
	{
		OnceCell::new_init(value)
	}
}

impl<T> From<OnceCell<T>> for Option<T>
{
	fn from(value: OnceCell<T>) -> Self
	{
		value.into_inner()
	}
}

impl<T> Drop for OnceCell<T>
{
	fn drop(&mut self)
	{
		match *self.state.get_mut()
		{
			STATE_INIT => (),
			STATE_LOCKED => unreachable!("dropped cell during initialization"),
			// MaybeUninit requires us to manually drop the value
			STATE_READY => unsafe{self.value.get_mut().assume_init_drop()},
			x => unreachable!("invalid state {x}"),
		}
	}
}

unsafe impl<T: Send> Send for OnceCell<T> {}

unsafe impl<T: Sync> Sync for OnceCell<T> {}
