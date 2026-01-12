// SPDX-License-Identifier: BSD-3-Clause

use embassy_stm32::{bind_interrupts, peripherals};
use embassy_stm32::usb::{Config as OtgConfig, Driver, InterruptHandler};
use embassy_usb::control::{self, Request};
use embassy_usb::{Builder, Config as DeviceConfig, Handler, UsbVersion};
use static_cell::ConstStaticCell;

use crate::resources::UsbResources;
use crate::serial_number::serialNumber;

const VID: u16 = 0x1209;
const PID: u16 = 0xbadb;

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
pub async fn usbTask(usb: UsbResources)
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
	let mut serialHandler = SerialHandler::new();

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
	// Device is described in the interface descriptor, not here
	config.device_class = 0;
	config.device_sub_class = 0;
	config.device_protocol = 0;
	// Use a 64 byte max packet size for EP0 (max for FS)
	config.max_packet_size_0 = 64;
	// BCD encoded device version
	config.device_release = 0x0001;
	// Set up our device description strings
	config.manufacturer = Some("dragonmux");
	config.product = Some("BMD USB serial conduit");
	config.serial_number = Some(serialNumber().await);
	// We do not want or need to use interface association descriptors
	config.composite_with_iads = false;
	// Allow us to draw up to 100mA
	config.max_power = 100;
	config
}

struct SerialHandler
{
}

impl SerialHandler
{
	pub fn new() -> Self
	{
		// Bring up a new serial events handler in idle state
		Self
		{
		}
	}
}

impl Handler for SerialHandler
{
	fn control_in<'a>(&'a mut self, packet: Request, data: &'a mut [u8]) -> Option<control::InResponse<'a>>
	{
		None
	}

	fn control_out(&mut self, packet: Request, data: &[u8]) -> Option<control::OutResponse>
	{
		None
	}
}
