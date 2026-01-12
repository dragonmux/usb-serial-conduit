// SPDX-License-Identifier: BSD-3-Clause

use assign_resources::assign_resources;
use embassy_stm32::
{
    Config, Peri, Peripherals, peripherals,
};

assign_resources!
{
	usb: UsbResources
	{
		peripheral: USB_OTG_FS = UsbPeripheral,
		dm: PA11,
		dp: PA12,
	}
	uart: DmaUartResources
	{
		peripheral: USART2 = UartPeripheral,
		tx: PA2,
		rx: PA3,
		tx_dma: GPDMA1_CH0,
		rx_dma: GPDMA1_CH1,
	}
}

pub mod resources
{
	pub use super::
	{
		AssignedResources,
		UsbResources,
		DmaUartResources,
	};
}

pub fn init() -> Peripherals
{
	use embassy_stm32::rcc::
	{
		mux, AHBPrescaler, APBPrescaler, Hsi48Config, MSIRange, Pll, PllSource, PllPreDiv, PllMul, PllDiv,
		Sysclk, VoltageScale,
	};

	let mut config = Config::default();
	// Set up to use MSIS as our primary clock source, and turn MSIK off
	config.rcc.msis = Some(MSIRange::RANGE_48MHZ);
	config.rcc.msik = None;
	// Set up the HSI48 for USB w/ CRS to stabalise the clock
	config.rcc.hsi48 = Some(Hsi48Config { sync_from_usb: true });
	// Use PLL1 to provide a suitable clock to run the part at 160MHz
	config.rcc.pll1 = Some(
		Pll
		{
			source: PllSource::MSIS,
			// Predivide down to 12MHz to bring the clock into range for the PLL
			prediv: PllPreDiv::DIV3,
			// Multiply up to 320MHz
			mul: PllMul::MUL20,
			divp: None,
			divq: None,
			// Divide back down to 160MHz
			divr: Some(PllDiv::DIV2),
		}
	);
	// No prescaling is required on any of the busses this way, but we do need to use PLL1R as the clock source
	config.rcc.sys = Sysclk::PLL1_R;
	config.rcc.ahb_pre = AHBPrescaler::DIV1;
	config.rcc.apb1_pre = APBPrescaler::DIV1;
	config.rcc.apb2_pre = APBPrescaler::DIV1;
	config.rcc.apb3_pre = APBPrescaler::DIV1;
	// Have to run in the highest power (1.2Vcore) range to run this clock
	config.rcc.voltage_range = VoltageScale::RANGE1;

	// Set up the muxes to route HSI48 to the USB core
	config.rcc.mux.iclksel = mux::Iclksel::HSI48;

	embassy_stm32::init(config)
}
