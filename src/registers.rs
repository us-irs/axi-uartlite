#[bitbybit::bitfield(u32)]
pub struct RxFifo {
    #[bits(0..=7, r)]
    pub data: u8,
}

#[bitbybit::bitfield(u32)]
pub struct TxFifo {
    #[bits(0..=7, w)]
    pub data: u8,
}

#[bitbybit::bitfield(u32)]
pub struct Status {
    #[bit(7, r)]
    pub parity_error: bool,
    #[bit(6, r)]
    pub frame_error: bool,
    #[bit(5, r)]
    pub overrun_error: bool,
    #[bit(4, r)]
    pub intr_enabled: bool,
    #[bit(3, r)]
    pub tx_fifo_full: bool,
    #[bit(2, r)]
    pub tx_fifo_empty: bool,
    #[bit(1, r)]
    pub rx_fifo_full: bool,
    /// RX FIFO contains valid data.
    #[bit(0, r)]
    pub rx_fifo_valid_data: bool,
}

#[bitbybit::bitfield(u32, default = 0x0)]
pub struct Control {
    #[bit(4, w)]
    enable_interrupt: bool,
    #[bit(1, w)]
    reset_rx_fifo: bool,
    #[bit(0, w)]
    reset_tx_fifo: bool,
}

/// AXI UARTLITE register block definition.
#[derive(derive_mmio::Mmio)]
#[repr(C)]
pub struct Registers {
    #[mmio(PureRead)]
    rx_fifo: RxFifo,
    tx_fifo: TxFifo,
    #[mmio(PureRead)]
    stat_reg: Status,
    ctrl_reg: Control,
}
