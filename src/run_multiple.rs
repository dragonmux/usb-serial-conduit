// SPDX-License-Identifier: BSD-3-Clause

use core::{pin::Pin, task::{Context, Poll}};

enum MaybeDone<Fut: Future>
{
	// A Future that is yet to complete
	Future(Fut),
	// A the output of a Future that has completed
	Done(Fut::Output),
}

impl<Fut: Future> MaybeDone<Fut>
{
	fn poll(self: Pin<&mut Self>, ctx: &mut Context<'_>) -> bool
	{
		let this = unsafe { self.get_unchecked_mut() };
		match this
		{
			Self::Future(fut) => match unsafe { Pin::new_unchecked(fut) }.poll(ctx)
			{
				Poll::Ready(result) =>
				{
					*this = Self::Done(result);
					true
				}
				Poll::Pending => false,
			},
			_ => true,
		}
	}
}

pub struct RunTwo<Future1: Future, Future2: Future>
{
	future1: MaybeDone<Future1>,
	future2: MaybeDone<Future2>,
}

impl<Future1: Future, Future2: Future> RunTwo<Future1, Future2>
{
	pub fn new(future1: Future1, future2: Future2) -> Self
	{
		Self
		{
			future1: MaybeDone::Future(future1),
			future2: MaybeDone::Future(future2),
		}
	}
}

impl<Future1: Future, Future2: Future> Future for RunTwo<Future1, Future2>
{
	type Output = ();

	fn poll(self: Pin<&mut Self>, ctx: &mut Context<'_>) -> Poll<()>
	{
		let this = unsafe { self.get_unchecked_mut() };
		let allDone =
			unsafe { Pin::new_unchecked(&mut this.future1) }.poll(ctx) &&
			unsafe { Pin::new_unchecked(&mut this.future2) }.poll(ctx);

		if allDone
		{
			Poll::Ready(())
		}
		else
		{
			Poll::Pending
		}
	}
}
