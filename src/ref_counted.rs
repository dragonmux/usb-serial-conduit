// SPDX-License-Identifier: BSD-3-Clause

use core::{cell::UnsafeCell, ops::{Deref, DerefMut}, ptr::NonNull};

pub struct RefCounted<T: ?Sized>
{
	value: UnsafeCell<T>
}

impl<T> RefCounted<T>
{
	pub const fn new(value: T) -> Self
	{
		Self
		{
			value: UnsafeCell::new(value)
		}
	}
}

impl<T: ?Sized> RefCounted<T>
{
	pub fn ref_to(&self) -> RefTo<T>
	{
		let value = unsafe { NonNull::new_unchecked(self.value.get()) };
		RefTo { value }
	}
}

impl<T: ?Sized> Deref for RefCounted<T>
{
	type Target = T;

	#[inline]
	fn deref(&self) -> &T
	{
		unsafe { &*self.value.get() }
	}
}

impl<T: ?Sized> DerefMut for RefCounted<T>
{
	#[inline]
	fn deref_mut(&mut self) -> &mut T
	{
		self.value.get_mut()
	}
}

pub struct RefTo<T: ?Sized>
{
	value: NonNull<T>,
}

impl<T: ?Sized> Deref for RefTo<T>
{
	type Target = T;

	#[inline]
	fn deref(&self) -> &T
	{
		unsafe { self.value.as_ref() }
	}
}
