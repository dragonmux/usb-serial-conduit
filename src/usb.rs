// SPDX-License-Identifier: BSD-3-Clause

use embassy_stm32::{bind_interrupts, peripherals};
use embassy_stm32::usb::{Config as OtgConfig, Driver, InterruptHandler};
use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use embassy_sync::channel::{Receiver, Sender};
use embassy_usb::control::{self, Request};
use embassy_usb::driver::{Direction, EndpointAddress};
use embassy_usb::types::InterfaceNumber;
use embassy_usb::{Builder, Config as DeviceConfig, Handler, UsbVersion};
use static_cell::ConstStaticCell;

use crate::resources::UsbResources;
use crate::serial_number::serialNumber;
use crate::types::{ReceiveRequest, SerialEncoding, TransmitRequest};

const VID: u16 = 0x1209;
const PID: u16 = 0xbadb;

/// Communications Device Class Device
const USB_CLASS_CDC: u8 = 0x02;
/// Data interface
const USB_CLASS_DATA: u8 = 0x0a;
/// Miscellaneous Device
const USB_CLASS_MISC: u8 = 0xef;

/// CDC ACM subclass device
const CDC_SUBCLASS_ACM: u8 = 2;
/// Non-specific CDC protocol (control)
const CDC_PROTOCOL_NONE: u8 = 0;

/// Non-specific data subclass
const DATA_SUBCLASS_NONE: u8 = 0;
/// Non-specific data protocol
const DATA_PROTOCOL_NONE: u8 = 0;

/// Common Class
const MISC_SUBCLASS_COMMON: u8 = 2;
// Interface Association
const MISC_PROTOCOL_IAD: u8 = 1;

bind_interrupts!
(
	struct UsbIrqs
	{
    	OTG_FS => InterruptHandler<peripherals::USB_OTG_FS>;
	}
);

// Buffer that must be large enough to receive any possible packet we can dequeue
static RX_BUFFER: ConstStaticCell<[u8; 64]> = ConstStaticCell::new([0u8; 64]);
// Buffer that must be large enough to hold any possible control packet (in or out) that might be generated
static CONTROL_BUFFER: ConstStaticCell<[u8; 64]> = ConstStaticCell::new([0u8; 64]);
// Buffer that must be large enough to hold the completed configuration descriptor
static CONFIGURATION_DESCRIPTOR: ConstStaticCell<[u8; 64]> = ConstStaticCell::new([0u8; 64]);

#[embassy_executor::task]
pub async fn usbTask
(
	usb: UsbResources,
	transmitChannel: Receiver<'static, CriticalSectionRawMutex, TransmitRequest, 1>,
	receiveChannel: Sender<'static, CriticalSectionRawMutex, ReceiveRequest, 1>,
)
{
	let mut config = OtgConfig::default();
	// We have VBus hooked up on this hardware, so do this.
	config.vbus_detection = true;
	// Create an instance of the USB driver for our peripheral
	let driver = Driver::new_fs
	(
		usb.peripheral,
		UsbIrqs,
		usb.dp,
		usb.dm,
		RX_BUFFER.take(),
		config
	);

	// Build the device configuration state we intend to use
	let deviceConfig = deviceConfig().await;
	// Along with grabbing the buffer for hold the config descriptor
	let configDescriptor = CONFIGURATION_DESCRIPTOR.take();

	// Create the serial handler here so we get teardown ops in the right order
	let mut serialHandler = SerialHandler::new(transmitChannel, receiveChannel);

	// Make an instance of the embassy USB state builder
	let mut builder = Builder::new
	(
		driver,
		deviceConfig,
		configDescriptor,
		&mut [],
		&mut [],
		CONTROL_BUFFER.take(),
	);

	// Define a new "function" to be the root of the CDC-ACM support
	let mut serialFunction = builder.function
	(
		USB_CLASS_CDC,
		CDC_SUBCLASS_ACM,
		CDC_PROTOCOL_NONE
	);
	// Now define the control interface
	let mut serialControlInterface = serialFunction.interface();
	let mut serialControlInterface = serialControlInterface.alt_setting
	(
		USB_CLASS_CDC,
		CDC_SUBCLASS_ACM,
		CDC_PROTOCOL_NONE,
		None
	);
	serialHandler.controlInterface(serialControlInterface.interface_number());
	// Extract the endpoint for sending notifications for this control interface
	let serialNotification = serialControlInterface.endpoint_interrupt_in
	(
		Some(EndpointAddress::from_parts(2, Direction::In)),
		16,
		100
	);

	// Followed by the data interface
	let mut serialDataInterface = serialFunction.interface();
	let mut serialDataInterface = serialDataInterface.alt_setting
	(
		USB_CLASS_DATA,
		DATA_SUBCLASS_NONE,
		DATA_PROTOCOL_NONE,
		None
	);
	// Extract the endpoints for communicating on the data interface
	let serialDataTx = serialDataInterface.endpoint_bulk_in
	(
		Some(EndpointAddress::from_parts(1, Direction::In)),
		64
	);
	let serialDataRx = serialDataInterface.endpoint_bulk_out
	(
		Some(EndpointAddress::from_parts(1, Direction::Out)),
		64
	);

	// Drop our reference to the function so the builder can work
	drop(serialFunction);

	// Register the serial handler so we can deal with CDC ACM state requests
	builder.handler(&mut serialHandler);

	// Turn the completed builder into a USB device and run it
	let mut usbDevice = builder.build();
	usbDevice.run().await
}

// Compile-time set up the device descriptor for this
async fn deviceConfig() -> DeviceConfig<'static>
{
	let mut config = DeviceConfig::new(VID, PID);
	// We're a USB 2.1 (USB 3 compliance over USB LS/FS/HS) device, meaning we can have BOS
	config.bcd_usb = UsbVersion::TwoOne;
	// Device is a misc IAD-based device
	config.device_class = USB_CLASS_MISC;
	config.device_sub_class = MISC_SUBCLASS_COMMON;
	config.device_protocol = MISC_PROTOCOL_IAD;
	// Use a 64 byte max packet size for EP0 (max for FS)
	config.max_packet_size_0 = 64;
	// BCD encoded device version
	config.device_release = 0x0001;
	// Set up our device description strings
	config.manufacturer = Some("dragonmux");
	config.product = Some("BMD USB serial conduit");
	config.serial_number = Some(serialNumber().await);
	// Allow us to draw up to 100mA
	config.max_power = 100;
	config
}

#[repr(u8)]
#[derive(Clone, Copy)]
enum CdcRequest
{
	SetLineCoding = 0x20,
	GetLineCoding = 0x21,
	SetControlLineState = 0x22,
}

impl From<u8> for CdcRequest
{
	fn from(value: u8) -> Self
	{
		match value
		{
			0x20 => Self::SetLineCoding,
			0x21 => Self::GetLineCoding,
			0x22 => Self::SetControlLineState,
			_ => panic!("Invalid CDC ACM request type for conversion"),
		}
	}
}

#[repr(u8)]
#[derive(Clone, Copy)]
enum CdcNotification
{
	SerialState = 0x20,
}

struct SerialHandler
{
	controlInterface: u16,
	transmitChannel: Receiver<'static, CriticalSectionRawMutex, TransmitRequest, 1>,
	receiveChannel: Sender<'static, CriticalSectionRawMutex, ReceiveRequest, 1>,
	encoding: SerialEncoding,
}

impl SerialHandler
{
	pub fn new(
		transmitChannel: Receiver<'static, CriticalSectionRawMutex, TransmitRequest, 1>,
		receiveChannel: Sender<'static, CriticalSectionRawMutex, ReceiveRequest, 1>,
	) -> Self
	{
		// Bring up a new serial events handler in idle state
		Self
		{
			controlInterface: 255,
			transmitChannel,
			receiveChannel,
			encoding: SerialEncoding::default(),
		}
	}

	pub fn controlInterface(&mut self, controlInterface: InterfaceNumber)
	{
		self.controlInterface = controlInterface.0 as u16;
	}

	fn controlLineState(&mut self, _state: u16)
	{
	}

	fn encodingToData(&self, data: &mut [u8]) -> Option<usize>
	{
		None
	}

	fn encodingFromData(&mut self, buffer: &[u8]) -> Option<()>
	{
		None
	}
}

impl Handler for SerialHandler
{
	fn control_in<'a>(&'a mut self, packet: Request, data: &'a mut [u8]) -> Option<control::InResponse<'a>>
	{
		if packet.recipient != control::Recipient::Interface ||
			packet.request_type != control::RequestType::Class ||
			packet.index != self.controlInterface
		{
			return None
		}

		match CdcRequest::from(packet.request)
		{
			CdcRequest::GetLineCoding =>
			{
				self.encodingToData(data)
					.map(|length| control::InResponse::Accepted(&data[0..length]))
			}
			_ => None
		}
	}

	fn control_out(&mut self, packet: Request, data: &[u8]) -> Option<control::OutResponse>
	{
		if packet.recipient != control::Recipient::Interface ||
			packet.request_type != control::RequestType::Class ||
			packet.index != self.controlInterface
		{
			return None
		}

		match CdcRequest::from(packet.request)
		{
			CdcRequest::SetControlLineState =>
			{
				self.controlLineState(packet.value);
				Some(control::OutResponse::Accepted)
			}
			CdcRequest::SetLineCoding =>
			{
				self.encodingFromData(data)
					.map(|()| control::OutResponse::Accepted)
			}
			_ => None
		}
	}
}
