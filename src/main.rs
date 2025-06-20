#![no_std]
#![no_main]

use panic_rtt_target as _;
use rtt_target::{rprintln, rtt_init_print};
use cortex_m_rt::entry;

use microbit::{
    board::Board,
    hal::{prelude::*, saadc::{SaadcConfig, Resolution}, Timer},
    hal::gpio::{Level, p0::P0_08}, // Import the specific pin and Level
};

#[entry]
fn main() -> ! {
    rtt_init_print!();
    let board = Board::take().unwrap();
    let mut timer = Timer::new(board.TIMER0);

    // ADC Setup (same as before)
    let saadc_config = SaadcConfig {
        resolution: Resolution::_12BIT,
        ..Default::default()
    };
    let mut adc = microbit::hal::saadc::Saadc::new(board.SAADC, saadc_config);

    // Sensor Pin Setup (same as before)
    let mut sensor_pin_1 = board.pins.p0_02.into_floating_input();
    let mut sensor_pin_2 = board.pins.p0_03.into_floating_input();
    let mut sensor_pin_3 = board.pins.p0_04.into_floating_input();

    // --- NEW: LED Control Pin Setup ---
    // Configure pin P8 as a digital output, starting in the "Low" (off) state.
    // Change change 2
    let mut led_pin = board.pins.p0_09.into_push_pull_output(Level::Low);

    rprintln!("Plant Watering System Armed!");
    rprintln!("----------------------------");

    loop {
        let val1 = adc.read(&mut sensor_pin_1).unwrap() as u16;
        let val2 = adc.read(&mut sensor_pin_2).unwrap() as u16;
        let val3 = adc.read(&mut sensor_pin_3).unwrap() as u16;

        rprintln!("S1: {:<4} | S2: {:<4} | S3: {:<4}", val1, val2, val3);

        // --- NEW: Control Logic ---
        // Check the moisture level of the first sensor.
        if val1 < 4000 {
            // If the value is LOW (wet), turn the LED ON.
            rprintln!("S1 is WET, turning LED ON");
            led_pin.set_high().unwrap(); // Set P8 high, turning on the transistor
        } else {
            // If the value is HIGH (dry), turn the LED OFF.
            rprintln!("S1 is DRY, turning LED OFF");
            led_pin.set_low().unwrap(); // Set P8 low, turning off the transistor
        }

        timer.delay_ms(1000_u32);
    }
}