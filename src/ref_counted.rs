// SPDX-License-Identifier: BSD-3-Clause

use core::
{
	cell::{Cell, UnsafeCell}, hint, marker::PhantomData, mem::MaybeUninit, ops::{Deref, DerefMut}, ptr::{self, NonNull}
};

type BorrowCounter = isize;
const UNUSED: BorrowCounter = 0;

/// This represents a pool capable of holding N T's
pub struct RcPool<T: Sized, const N: usize>
{
	pool: [MaybeUninit<RcInner<T>>; N],
	allocated: usize,
}

impl<T, const N: usize> RcPool<T, N>
{
	/// Create a new Rc pool
	pub const fn new() -> Self
	{
		Self
		{
			pool: [const { MaybeUninit::uninit() }; N],
			allocated: 0,
		}
	}
}

impl<T: Sized, const N: usize> RcPool<T, N>
{
	pub fn alloc(&mut self, value: T) -> Option<Rc<T>>
	{
		if self.allocated == N
		{
			None
		}
		else
		{
			let inner = self.pool[self.allocated]
				.write(
					RcInner
					{
						refCount: Cell::new(1),
						borrowCount: Cell::new(UNUSED),
						value: UnsafeCell::new(value),
					}
				);
			Some(Rc::fromInner(inner))
		}
	}
}

/// Container for the control block and data of a given reference counted type
struct RcInner<T: Sized>
{
	refCount: Cell<usize>,
	borrowCount: Cell<BorrowCounter>,
	value: UnsafeCell<T>,
}

impl<T> RcInner<T>
{
	fn count(&self) -> usize
	{
		self.refCount.get()
	}

	fn incCount(&self)
	{
		let count = self.count();
		// It should be entirely possible to be here if the reference count is not at least 1.
		// We emit this hint to ensure LLVM doesn't do a codegen stupid
		unsafe
		{
			hint::assert_unchecked(count != 0);
		}
		let count = count.wrapping_add(1);
		self.refCount.set(count);
		// If somehow that managed to overflow, panic with a message
		if count == 0
		{
			panic!("Reference count overflowed for Rc at {:?}", self as *const Self);
		}
	}

	fn decCount(&self)
	{
		self.refCount.set(self.count() - 1);
	}
}

/// A reference counted pointer to some data, allocated from a pool
pub struct Rc<T: Sized>
{
	ptr: NonNull<RcInner<T>>,
	marker: PhantomData<RcInner<T>>,
}

impl<T: Sized> Rc<T>
{
	fn fromInner(inner: &mut RcInner<T>) -> Self
	{
		Self
		{
			ptr: unsafe { NonNull::new_unchecked(inner) },
			marker: PhantomData,
		}
	}

	#[inline]
	unsafe fn fromInnerIn(ptr: NonNull<RcInner<T>>) -> Self
	{
		Self { ptr, marker: PhantomData }
	}

	#[inline(always)]
	fn inner(&self) -> &RcInner<T>
	{
		unsafe { self.ptr.as_ref() }
	}

	#[inline]
	pub fn borrow(&self) -> Ref<'_, T>
	{
		let value = unsafe { NonNull::new_unchecked(self.inner().value.get()) };
		Ref { value, marker: PhantomData }
	}

	#[inline]
	pub fn borrowMut(&self) -> RefMut<'_, T>
	{
		match self.tryBorrowMut()
		{
			Ok(mutRef) => mutRef,
			Err(_) => panic!("Rc already mutably borrowed"),
		}
	}

	pub fn tryBorrowMut(&self) -> Result<RefMut<'_, T>, BorrowMutError>
	{
		match BorrowRefMut::new(&self.inner().borrowCount)
		{
			Some(borrow) =>
			{
				let value = unsafe { NonNull::new_unchecked(self.inner().value.get()) };
				Ok(RefMut { value, _borrow: borrow, marker: PhantomData })
			},
			None => Err(BorrowMutError),
		}
	}
}

impl<T: Sized> Clone for Rc<T>
{
	#[inline]
	fn clone(&self) -> Self
	{
		unsafe
		{
			self.inner().incCount();
			Self::fromInnerIn(self.ptr)
		}
	}
}

impl<T: Sized> Drop for Rc<T>
{
	#[inline]
	fn drop(&mut self)
	{
		// Drop the reference count by one
		self.inner().decCount();
		if self.inner().count() == 0
		{
			// If we were the last reference, destroy the contained object
			unsafe
			{
				ptr::drop_in_place(&mut (*self.ptr.as_ptr()).value);
			}
		}
	}
}

pub struct Ref<'b, T: ?Sized + 'b>
{
	value: NonNull<T>,
	marker: PhantomData<&'b T>,
}

impl<T: ?Sized> Deref for Ref<'_, T>
{
	type Target = T;

	#[inline]
	fn deref(&self) -> &T
	{
		unsafe { self.value.as_ref() }
	}
}

pub struct BorrowMutError;

pub struct RefMut<'b, T: ?Sized + 'b>
{
	value: NonNull<T>,
	_borrow: BorrowRefMut<'b>,
	marker: PhantomData<&'b T>,
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
	fn deref_mut(&mut self) -> &mut T
	{
		unsafe { self.value.as_mut() }
	}
}

struct BorrowRefMut<'b>
{
	borrow: &'b Cell<BorrowCounter>,
}

impl<'b> BorrowRefMut<'b>
{
	#[inline]
	const fn new(borrow: &'b Cell<BorrowCounter>) -> Option<Self>
	{
		match borrow.get()
		{
			UNUSED =>
			{
				borrow.replace(UNUSED - 1);
				Some(BorrowRefMut { borrow })
			},
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
