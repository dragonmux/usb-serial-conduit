// SPDX-License-Identifier: BSD-3-Clause

use embassy_stm32::uid::uid;
use embassy_sync::once_lock::OnceLock;

// Provide space for the serial number to be written into at runtime
static SERIAL_NUMBER: OnceLock<SerialNumber<8>> = OnceLock::new();

struct SerialNumber<const N: usize>
{
	value: [u8; N]
}

impl<const N: usize> SerialNumber<N>
{
	pub fn from_bytes(serialNumber: [u8; N]) -> Self
	{
		Self
		{
			value: serialNumber
		}
	}

	pub const fn as_str(&self) -> &str
	{
		unsafe { str::from_utf8_unchecked(&self.value) }
	}
}

pub fn readSerialNumber()
{
	let uniqueIDBytes = uid();
	let uniqueID1 = u32::from_ne_bytes(uniqueIDBytes[0..4].try_into().unwrap());
	let uniqueID2 = u32::from_ne_bytes(uniqueIDBytes[4..8].try_into().unwrap());
	let uniqueID3 = u32::from_ne_bytes(uniqueIDBytes[8..12].try_into().unwrap());
	let uniqueID = uniqueID1 + uniqueID2 + uniqueID3;
	let mut serialNumber = [0u8; 8];
	for (idx, byte) in serialNumber.iter_mut().enumerate()
	{
		let mut value = (((uniqueID >> (idx * 4)) & 0x0f) as u8) + ('0' as u8);
		if value > ('9' as u8)
		{
			value += 7;
		}
		*byte = value;
	}

	let _ = SERIAL_NUMBER.init(SerialNumber::from_bytes(serialNumber));
}

pub async fn serialNumber() -> &'static str
{
	SERIAL_NUMBER.get().await.as_str()
}
