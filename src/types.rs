// SPDX-License-Identifier: BSD-3-Clause

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

#[derive(Clone, Copy)]
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
}
