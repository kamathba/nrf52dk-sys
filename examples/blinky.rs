// Based on nRF5-sdk/examples/peripheral/blinky/main.c
#![no_main]
#![no_std]

#[macro_use(entry, exception)]
extern crate cortex_m_rt as rt;

// makes `panic!` print messages to the host stderr using semihosting
extern crate panic_semihosting;
use rt::ExceptionFrame;

extern crate nrf52dk_sys;
use nrf52dk_sys as nrf;

#[entry]
fn main() -> ! {
    unsafe {
        nrf::bsp_board_leds_init();

        loop {
            for led in 0..nrf::LEDS_NUMBER {
                nrf::bsp_board_led_invert(led);
                nrf::_nrf_delay_ms(500);
            }
        }
    }
}

#[exception]
fn HardFault(ef: &ExceptionFrame) -> ! {
    panic!("HardFault at {:#?}", ef);
}

#[exception]
fn DefaultHandler(irqn: i16) {
    panic!("Unhandled exception (IRQn = {})", irqn);
}
