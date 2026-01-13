// SPDX-License-Identifier: BSD-3-Clause

use core::{cell::{Cell, UnsafeCell}, marker::PhantomData, ops::{Deref, DerefMut}, ptr::NonNull};

type BorrowCounter = isize;
const UNUSED: BorrowCounter = 0;

pub struct UnsafeRefCell<T: ?Sized>
{
	borrow: Cell<BorrowCounter>,
	value: UnsafeCell<T>,
}

impl<T> UnsafeRefCell<T>
{
	pub const fn new(value: T) -> Self
	{
		Self
		{
			value: UnsafeCell::new(value),
			borrow: Cell::new(UNUSED),
		}
	}
}

impl<T: ?Sized> UnsafeRefCell<T>
{
	#[inline]
	pub const fn borrow(&self) -> Ref<T>
	{
		let value = unsafe { NonNull::new_unchecked(self.value.get()) };
		Ref { value }
	}

	pub fn borrowMut(&self) -> RefMut<'_, T>
	{
		match self.tryBorrowMut()
		{
			Ok(borrow) => borrow,
			Err(_) => panic!("UnsafeRefCell already borrowed")
		}
	}

	pub fn tryBorrowMut(&self) -> Result<RefMut<'_, T>, BorrowMutError>
	{
		match BorrowRefMut::new(&self.borrow)
		{
			Some(borrow) =>
			{
				let value = unsafe { NonNull::new_unchecked(self.value.get()) };
				Ok(RefMut { value, borrow, marker: PhantomData })
			}
			None => Err(BorrowMutError),
		}
	}
}

pub struct BorrowMutError;

struct BorrowRefMut<'b>
{
	borrow: &'b Cell<BorrowCounter>,
}

impl<'b> BorrowRefMut<'b>
{
	#[inline]
	fn new(borrow: &'b Cell<BorrowCounter>) -> Option<Self>
	{
		match borrow.get()
		{
			UNUSED =>
			{
				borrow.replace(UNUSED - 1);
				Some(BorrowRefMut { borrow })
			}
			_ => None,
		}
	}
}

impl Drop for BorrowRefMut<'_>
{
	#[inline]
	fn drop(&mut self)
	{
		let borrow = self.borrow.get();
		debug_assert!(borrow < UNUSED);
		self.borrow.replace(borrow + 1);
	}
}

pub struct Ref<T: ?Sized>
{
	value: NonNull<T>,
}

impl<T: ?Sized> Deref for Ref<T>
{
	type Target = T;

	#[inline]
	fn deref(&self) -> &T
	{
		unsafe { self.value.as_ref() }
	}
}

pub struct RefMut<'b, T: ?Sized + 'b>
{
	value: NonNull<T>,
	borrow: BorrowRefMut<'b>,
	marker: PhantomData<&'b mut T>,
}

impl<T: ?Sized> Deref for RefMut<'_, T>
{
	type Target = T;

	#[inline]
	fn deref(&self) -> &T
	{
		unsafe { self.value.as_ref() }
	}
}

impl<T: ?Sized> DerefMut for RefMut<'_, T>
{
	#[inline]
	fn deref_mut(&mut self) -> &mut T
	{
		unsafe { self.value.as_mut() }
	}
}
