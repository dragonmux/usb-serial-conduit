// SPDX-License-Identifier: BSD-3-Clause

use core::fmt::{Display, Formatter, Result};

use embassy_stm32::usart;

pub enum TransmitRequest
{
}

pub enum ReceiveRequest
{
	ChangeEncoding(SerialEncoding),
}

#[repr(u8)]
#[derive(Clone, Copy)]
pub enum StopBits
{
	One = 0,
	OneAndHalf = 1,
	Two = 2,
}

impl From<u8> for StopBits
{
	fn from(value: u8) -> Self
	{
		match value
		{
			0 => Self::One,
			1 => Self::OneAndHalf,
			2 => Self::Two,
			_ => panic!("Invalid stop bits setting for conversion")
		}
	}
}

impl Into<usart::StopBits> for StopBits
{
	fn into(self) -> usart::StopBits
	{
		match self
		{
			Self::One => usart::StopBits::STOP1,
			Self::OneAndHalf => usart::StopBits::STOP1P5,
			Self::Two => usart::StopBits::STOP2,
		}
	}
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

impl From<u8> for ParityType
{
	fn from(value: u8) -> Self
	{
		match value
		{
			0 => Self::None,
			1 => Self::Odd,
			2 => Self::Even,
			3 => Self::Mark,
			4 => Self::Space,
			_ => panic!("Invalid parity type setting for conversion")
		}
	}
}

impl Into<usart::Parity> for ParityType
{
	fn into(self) -> usart::Parity
	{
		match self
		{
			Self::None => usart::Parity::ParityNone,
			Self::Odd => usart::Parity::ParityOdd,
			Self::Even => usart::Parity::ParityEven,
			_ => panic!("Unable to represent {} to the hardware", self)
		}
	}
}

impl Display for ParityType
{
	fn fmt(&self, fmt: &mut Formatter<'_>) -> Result
	{
		match self
		{
			Self::None => write!(fmt, "none parity"),
			Self::Odd => write!(fmt, "odd parity"),
			Self::Even => write!(fmt, "even parity"),
			Self::Mark => write!(fmt, "mark parity"),
			Self::Space => write!(fmt, "space parity"),
		}
	}
}

#[derive(Clone, Copy)]
pub struct SerialEncoding
{
	pub baudRate: u32,
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

impl SerialEncoding
{
	pub fn fromData(data: &[u8]) -> Option<Self>
	{
		// There need to be at least 7 bytes to consume as a serial encoding
		if data.len() < 7
		{
			return None;
		}

		Some
		(
			Self
			{
				// Extract out the field components from the payload buffer
				baudRate: u32::from_le_bytes(data[0..4].try_into().unwrap()),
				stopBits: StopBits::from(data[4]),
				parityType: ParityType::from(data[5]),
				dataBits: data[6],
			}
		)
	}

	pub fn toData(&self, data: &mut [u8]) -> Option<usize>
	{
		// There need to be at least 7 bytes to format out a serial encoding state
		if data.len() < 7
		{
			return None;
		}

		// Copy out the fields into the payload buffer
		data[0..4].copy_from_slice(&self.baudRate.to_le_bytes());
		data[4] = self.stopBits as u8;
		data[5] = self.parityType as u8;
		data[6] = self.dataBits;
		Some(7)
	}

	pub fn stopBits(&self) -> usart::StopBits
	{
		self.stopBits.into()
	}

	pub fn parityType(&self) -> usart::Parity
	{
		self.parityType.into()
	}

	pub fn dataBits(&self) -> usart::DataBits
	{
		match self.dataBits
		{
			7 => usart::DataBits::DataBits7,
			8 => usart::DataBits::DataBits8,
			9 => usart::DataBits::DataBits9,
			bits => panic!("Unable to represent {bits} data bits to the hardware")
		}
	}
}
