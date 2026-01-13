// SPDX-License-Identifier: BSD-3-Clause

use embassy_embedded_hal::SetConfig;
use embassy_futures::select::{Either, select};
use embassy_stm32::mode::Async;
use embassy_stm32::{bind_interrupts, peripherals};
use embassy_stm32::usart::{Config as UartConfig, InterruptHandler, OutputConfig, Uart};
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
	let mut config = UartConfig::default();
	config.tx_config = OutputConfig::PushPull;

	let mut serialPort = Uart::new
	(
		uart.peripheral,
		uart.rx,
		uart.tx,
		UartIrqs,
		uart.tx_dma,
		uart.rx_dma,
		config.clone()
	)
	.expect("Failed to set up main serial interface");

	let mut auxSerialReceiveBuffer = [0u8; 64];

	loop
	{
		let receiveFuture = receiveChannel.receive();
		let auxSerialReceiveFuture =
			serialPort.read(&mut auxSerialReceiveBuffer);
		match select(receiveFuture, auxSerialReceiveFuture).await
		{
			Either::First(request) =>
				handleReceiveRequest(request, &mut serialPort, &mut config).await,
			Either::Second(readResult) =>
			{
			}
		}
	}
}

async fn handleReceiveRequest(
	request: ReceiveRequest,
	serialPort: &mut Uart<'static, Async>,
	config: &mut UartConfig,
)
{
	match request
	{
		ReceiveRequest::ChangeEncoding(encoding) =>
		{
			config.baudrate = encoding.baudRate;
			config.stop_bits = encoding.stopBits();
			config.parity = encoding.parityType();
			config.data_bits = encoding.dataBits();

			serialPort.set_config(config)
				.expect("Unable to set desired encoding state");
		}
	}
}
