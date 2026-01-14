// SPDX-License-Identifier: BSD-3-Clause

#![allow(non_snake_case)]
#![no_std]
#![no_main]

mod ref_counted;
mod resources;
mod run_multiple;
mod serial;
mod serial_number;
mod types;
mod usb;

use embassy_executor::Spawner;
use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use embassy_sync::channel::Channel;
use embedded_alloc::LlffHeap as Heap;
// Magically inject the parts of the defmt machinary that are needed for doing defmt over RTT 🙃
use defmt_rtt as _;
// Magically inject #[panic_handler] so we get panic handling.. don't ask, it's absolutely magic how this can do that.
use panic_probe as _;
use static_cell::ConstStaticCell;

use crate::resources::resources::*;
use crate::serial::serialTask;
use crate::serial_number::readSerialNumber;
use crate::types::{ReceiveRequest, TransmitRequest};
use crate::usb::usbTask;

const HEAP_SIZE: usize = 1024 * 4; // 4KiB heap
#[global_allocator]
static HEAP: Heap = Heap::empty();
static HEAP_MEM: ConstStaticCell<[u8; HEAP_SIZE]> = ConstStaticCell::new([0; HEAP_SIZE]);

// Create a pair of channels for moving information between the USB and serial tasks
static TRANSMIT_CHANNEL: Channel<CriticalSectionRawMutex, TransmitRequest, 1> = Channel::new();
static RECEIVE_CHANNEL: Channel<CriticalSectionRawMutex, ReceiveRequest, 1> = Channel::new();

#[embassy_executor::main]
async fn main(spawner: Spawner)
{
	// Initialise our heap so we can use allocating types
	unsafe
	{
		let heapMemory = HEAP_MEM.take() as *mut u8;
		HEAP.init(heapMemory as usize, HEAP_SIZE);
	}

	// Initialise the execution environment so we're on the right clock
	let peripherals = resources::init();
	let resources = split_resources!(peripherals);

	// Read the serial number for the USB task to use
	readSerialNumber();

	// Spawn the task to handle USB for us
	spawner.spawn(usbTask(
		resources.usb, TRANSMIT_CHANNEL.receiver(), RECEIVE_CHANNEL.sender()
	).unwrap());
	// And then the one to handle serial
	spawner.spawn(serialTask(
		resources.uart, TRANSMIT_CHANNEL.sender(), RECEIVE_CHANNEL.receiver()
	).unwrap());
}
