// SPDX-License-Identifier: BSD-3-Clause

use embassy_stm32::{bind_interrupts, peripherals};
use embassy_stm32::usart::{Config as UartConfig, InterruptHandler, Uart};
use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use embassy_sync::channel::{Receiver, Sender};

use crate::resources::DmaUartResources;
use crate::types::{TransmitRequest, ReceiveRequest};

bind_interrupts!
(
	struct UartIrqs
	{
    	USART2 => InterruptHandler<peripherals::USART2>;
	}
);

#[embassy_executor::task]
pub async fn serialTask
(
	uart: DmaUartResources,
	transmitChannel: Sender<'static, CriticalSectionRawMutex, TransmitRequest, 1>,
	receiveChannel: Receiver<'static, CriticalSectionRawMutex, ReceiveRequest, 1>,
)
{
	let config = UartConfig::default();

	let serialPort = Uart::new
	(
		uart.peripheral,
		uart.rx,
		uart.tx,
		UartIrqs,
		uart.tx_dma,
		uart.rx_dma,
		config
	);
}
