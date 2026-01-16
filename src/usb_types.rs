// SPDX-License-Identifier: BSD-3-Clause

use bitmask_enum::bitmask;
use embassy_usb::types::InterfaceNumber;

const TYPE_CDC_INTERFACE: u8 = 0x24;
#[allow(unused)]
const TYPE_CDC_ENDPOINT: u8 = 0x25;

const SUBTYPE_CDC_HEADER: u8 = 0x00;
const SUBTYPE_CDC_CALL_MANAGEMENT: u8 = 0x01;
const SUBTYPE_CDC_ACM: u8 = 0x02;
const SUBTYPE_CDC_UNION: u8 = 0x06;

pub struct UsbCdcHeaderDescriptor
{
	cdcVersion: UsbCdcVersion,
}

#[repr(u16)]
#[derive(Clone, Copy)]
pub enum UsbCdcVersion
{
	OneDotOne = 0x0110,
}

pub struct UsbCdcCallManagementDescriptor
{
	capabilities: UsbCdcCallManagementCapabilities,
	dataInterface: u8,
}

#[bitmask(u8)]
pub enum UsbCdcCallManagementCapabilities
{
	SelfManaged = 0,
	ManagementOverDataInterface = 1,
}

pub struct UsbCdcAcmDescriptor
{
	capabilities: UsbCdcAcmCapabilities
}

#[bitmask(u8)]
pub enum UsbCdcAcmCapabilities
{
	SupportsCommFeatures = 0,
	SupportsLineCoding = 1,
	SupportsSendBreak = 2,
	SupportsNetworkConnection = 3,
}

pub struct UsbCdcUnionDescriptor
{
	controlInterface: u8,
	subInterface0: u8,
}

impl UsbCdcHeaderDescriptor
{
	pub const fn new(cdcVersion: UsbCdcVersion) -> Self
	{
		Self
		{
			cdcVersion
		}
	}

	pub const fn descriptorType(&self) -> u8
	{
		TYPE_CDC_INTERFACE
	}

	pub fn toBytes(&self) -> [u8; 3]
	{
		let mut result = [SUBTYPE_CDC_HEADER, 0, 0];
		let version = self.cdcVersion as u16;
		result[1..2].copy_from_slice(&version.to_le_bytes());
		result
	}
}

impl UsbCdcCallManagementDescriptor
{
	pub const fn new(capabilities: UsbCdcCallManagementCapabilities, dataInterface: u8) -> Self
	{
		Self { capabilities, dataInterface }
	}

	pub const fn descriptorType(&self) -> u8
	{
		TYPE_CDC_INTERFACE
	}

	pub fn toBytes(&self) -> [u8; 3]
	{
		[SUBTYPE_CDC_CALL_MANAGEMENT, self.capabilities.bits, self.dataInterface]
	}
}

impl UsbCdcAcmDescriptor
{
	pub const fn new(capabilities: UsbCdcAcmCapabilities) -> Self
	{
		Self { capabilities }
	}

	pub const fn descriptorType(&self) -> u8
	{
		TYPE_CDC_INTERFACE
	}

	pub fn toBytes(&self) -> [u8; 2]
	{
		[SUBTYPE_CDC_ACM, self.capabilities.bits]
	}
}

impl UsbCdcUnionDescriptor
{
	pub const fn new(controlInterface: InterfaceNumber, subInterface0: u8) -> Self
	{
		Self
		{
			controlInterface: controlInterface.0,
			subInterface0,
		}
	}

	pub const fn descriptorType(&self) -> u8
	{
		TYPE_CDC_INTERFACE
	}

	pub fn toBytes(&self) -> [u8; 3]
	{
		[SUBTYPE_CDC_UNION, self.controlInterface, self.subInterface0]
	}
}
