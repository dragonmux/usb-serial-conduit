// SPDX-License-Identifier: BSD-3-Clause

pub enum TransmitRequest
{
}

pub enum ReceiveRequest
{
}

#[repr(u8)]
#[derive(Clone, Copy)]
pub enum StopBits
{
	One = 0,
	OneAndHalf = 1,
	Two = 2,
}

#[repr(u8)]
#[derive(Clone, Copy)]
pub enum ParityType
{
	None = 0,
	Odd = 1,
	Even = 2,
	Mark = 3,
	Space = 4,
}

pub struct SerialEncoding
{
	baudRate: u32,
	stopBits: StopBits,
	parityType: ParityType,
	dataBits: u8,
}

impl Default for SerialEncoding
{
	fn default() -> Self
	{
		Self
		{
			baudRate: 115200,
			stopBits: StopBits::One,
			parityType: ParityType::None,
			dataBits: 8,
		}
	}
}
