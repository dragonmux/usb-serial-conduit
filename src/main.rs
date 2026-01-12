// SPDX-License-Identifier: BSD-3-Clause

#![allow(non_snake_case)]
#![no_std]
#![no_main]

mod resources;
mod serial;
mod serial_number;
mod usb;

use embassy_executor::Spawner;
// Magically inject the parts of the defmt machinary that are needed for doing defmt over RTT 🙃
use defmt_rtt as _;
// Magically inject #[panic_handler] so we get panic handling.. don't ask, it's absolutely magic how this can do that.
use panic_probe as _;

use crate::resources::resources::*;
use crate::serial::serialTask;
use crate::serial_number::readSerialNumber;
use crate::usb::usbTask;

#[embassy_executor::main]
async fn main(spawner: Spawner)
{
    // Initialise the execution environment so we're on the right clock
    let peripherals = resources::init();
    let resources = split_resources!(peripherals);

    // Read the serial number for the USB task to use
	readSerialNumber();

    // spawn the task to handle USB for us
    spawner.spawn(usbTask(resources.usb).unwrap());
    spawner.spawn(serialTask(resources.uart).unwrap());
}
