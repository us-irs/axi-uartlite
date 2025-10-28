//! # AXI UART Lite v2.0 driver
//!
//! This is a native Rust driver for the AMD AXI UART Lite v2.0 IP core.
//!
//! # Features
//!
//! If asynchronous TX operations are used, the number of wakers  which defaults to 1 waker can
//! also be configured. The [tx_async] module provides more details on the meaning of this number.
//!
//! - `1-waker` which is also a `default` feature
//! - `2-wakers`
//! - `4-wakers`
//! - `8-wakers`
//! - `16-wakers`
//! - `32-wakers`
#![no_std]
#![cfg_attr(docsrs, feature(doc_cfg))]

use core::convert::Infallible;
use registers::Control;
pub mod registers;

pub mod tx;
pub use tx::*;

pub mod rx;
pub use rx::*;

pub mod tx_async;
pub use tx_async::*;

pub const FIFO_DEPTH: usize = 16;

#[derive(Debug, Default, Copy, Clone, Eq, PartialEq)]
pub struct RxErrorsCounted {
    parity: u8,
    frame: u8,
    overrun: u8,
}

impl RxErrorsCounted {
    pub const fn new() -> Self {
        Self {
            parity: 0,
            frame: 0,
            overrun: 0,
        }
    }

    pub const fn parity(&self) -> u8 {
        self.parity
    }

    pub const fn frame(&self) -> u8 {
        self.frame
    }

    pub const fn overrun(&self) -> u8 {
        self.overrun
    }

    pub fn has_errors(&self) -> bool {
        self.parity > 0 || self.frame > 0 || self.overrun > 0
    }
}

pub struct AxiUartlite {
    rx: Rx,
    tx: Tx,
    errors: RxErrorsCounted,
}

impl AxiUartlite {
    /// Create a new AXI UART Lite peripheral driver.
    ///
    /// # Safety
    ///
    /// - The `base_addr` must be a valid memory-mapped register address of an AXI UART Lite peripheral.
    /// - Dereferencing an invalid or misaligned address results in **undefined behavior**.
    /// - The caller must ensure that no other code concurrently modifies the same peripheral registers
    ///   in an unsynchronized manner to prevent data races.
    /// - This function does not enforce uniqueness of driver instances. Creating multiple instances
    ///   with the same `base_addr` can lead to unintended behavior if not externally synchronized.
    /// - The driver performs **volatile** reads and writes to the provided address.
    pub const unsafe fn new(base_addr: u32) -> Self {
        let regs = unsafe { registers::Registers::new_mmio_at(base_addr as usize) };
        Self {
            rx: Rx {
                regs: unsafe { regs.clone() },
                errors: None,
            },
            tx: Tx { regs, errors: None },
            errors: RxErrorsCounted::new(),
        }
    }

    #[inline(always)]
    pub const fn regs(&mut self) -> &mut registers::MmioRegisters<'static> {
        &mut self.tx.regs
    }

    /// Write into the UART Lite.
    ///
    /// Returns [nb::Error::WouldBlock] if the TX FIFO is full.
    #[inline]
    pub fn write_fifo(&mut self, data: u8) -> nb::Result<(), Infallible> {
        self.tx.write_fifo(data).unwrap();
        if let Some(errors) = self.tx.errors {
            self.handle_status_reg_errors(errors);
        }
        Ok(())
    }

    /// Write into the FIFO without checking the FIFO fill status.
    ///
    /// This can be useful to completely fill the FIFO if it is known to be empty.
    #[inline(always)]
    pub fn write_fifo_unchecked(&mut self, data: u8) {
        self.tx.write_fifo_unchecked(data);
    }

    #[inline]
    pub fn read_fifo(&mut self) -> nb::Result<u8, Infallible> {
        let val = self.rx.read_fifo().unwrap();
        if let Some(errors) = self.rx.errors {
            self.handle_status_reg_errors(errors);
        }
        Ok(val)
    }

    #[inline(always)]
    pub fn read_fifo_unchecked(&mut self) -> u8 {
        self.rx.read_fifo_unchecked()
    }

    // TODO: Make this non-mut as soon as pure reads are available
    #[inline(always)]
    pub fn tx_fifo_empty(&mut self) -> bool {
        self.tx.fifo_empty()
    }

    // TODO: Make this non-mut as soon as pure reads are available
    #[inline(always)]
    pub fn tx_fifo_full(&mut self) -> bool {
        self.tx.fifo_full()
    }

    // TODO: Make this non-mut as soon as pure reads are available
    #[inline(always)]
    pub fn rx_has_data(&mut self) -> bool {
        self.rx.has_data()
    }

    /// Read the error counters and also resets them.
    pub fn read_and_clear_errors(&mut self) -> RxErrorsCounted {
        let errors = self.errors;
        self.errors = RxErrorsCounted::new();
        errors
    }

    #[inline(always)]
    fn handle_status_reg_errors(&mut self, errors: RxErrors) {
        if errors.frame() {
            self.errors.frame = self.errors.frame.saturating_add(1);
        }
        if errors.parity() {
            self.errors.parity = self.errors.parity.saturating_add(1);
        }
        if errors.overrun() {
            self.errors.overrun = self.errors.overrun.saturating_add(1);
        }
    }

    #[inline]
    pub fn reset_rx_fifo(&mut self) {
        self.regs().write_ctrl_reg(
            Control::builder()
                .with_enable_interrupt(false)
                .with_reset_rx_fifo(true)
                .with_reset_tx_fifo(false)
                .build(),
        );
    }

    #[inline]
    pub fn reset_tx_fifo(&mut self) {
        self.regs().write_ctrl_reg(
            Control::builder()
                .with_enable_interrupt(false)
                .with_reset_rx_fifo(false)
                .with_reset_tx_fifo(true)
                .build(),
        );
    }

    #[inline]
    pub fn split(self) -> (Tx, Rx) {
        (self.tx, self.rx)
    }

    #[inline]
    pub fn enable_interrupt(&mut self) {
        self.regs().write_ctrl_reg(
            Control::builder()
                .with_enable_interrupt(true)
                .with_reset_rx_fifo(false)
                .with_reset_tx_fifo(false)
                .build(),
        );
    }

    #[inline]
    pub fn disable_interrupt(&mut self) {
        self.regs().write_ctrl_reg(
            Control::builder()
                .with_enable_interrupt(false)
                .with_reset_rx_fifo(false)
                .with_reset_tx_fifo(false)
                .build(),
        );
    }
}

impl embedded_hal_nb::serial::ErrorType for AxiUartlite {
    type Error = Infallible;
}

impl embedded_hal_nb::serial::Write for AxiUartlite {
    #[inline]
    fn write(&mut self, word: u8) -> nb::Result<(), Self::Error> {
        self.tx.write(word)
    }

    #[inline]
    fn flush(&mut self) -> nb::Result<(), Self::Error> {
        self.tx.flush()
    }
}

impl embedded_hal_nb::serial::Read for AxiUartlite {
    #[inline]
    fn read(&mut self) -> nb::Result<u8, Self::Error> {
        self.rx.read()
    }
}

impl embedded_io::ErrorType for AxiUartlite {
    type Error = Infallible;
}

impl embedded_io::Read for AxiUartlite {
    fn read(&mut self, buf: &mut [u8]) -> Result<usize, Self::Error> {
        self.rx.read(buf)
    }
}

impl embedded_io::Write for AxiUartlite {
    fn write(&mut self, buf: &[u8]) -> Result<usize, Self::Error> {
        self.tx.write(buf)
    }

    fn flush(&mut self) -> Result<(), Self::Error> {
        self.tx.flush()
    }
}
