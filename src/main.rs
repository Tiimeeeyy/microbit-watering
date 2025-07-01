#![no_std]
#![no_main]

use cortex_m_rt::entry;
use panic_rtt_target as _;
use rtt_target::{rprintln, rtt_init_print};

use microbit::{
    board::Board,
    hal::{
        gpio::{Level, p0::P0_08},
        prelude::*,
        saadc::{Resolution, SaadcConfig},
        Timer,
    },
};

// --- Configuration Constants ---
const DRY_THRESHOLD: i16 = 2800;
const WET_THRESHOLD: i16 = 1600;
const DAILY_CHECK_INTERVAL_MS: u32 = 1_800_000;
const WATERING_CYCLE_MS: u32 = 15_000;
const MAX_WATERING_CYCLES: u8 = 10;


#[entry]
fn main() -> ! {
    rtt_init_print!();
    let board = Board::take().unwrap();
    let mut timer = Timer::new(board.TIMER0);

    let saadc_config = SaadcConfig {
        resolution: Resolution::_12BIT,
        ..Default::default()
    };
    let mut adc = microbit::hal::saadc::Saadc::new(board.SAADC, saadc_config);

    let mut sensor1 = board.pins.p0_02.into_floating_input();
    let mut sensor2 = board.pins.p0_03.into_floating_input();
    let mut sensor3 = board.pins.p0_04.into_floating_input();

    // --- LOGIC FLIPPED HERE (1/3) ---
    // Start the pin LOW, which is the OFF state for an active-high relay.
    let mut pump_pin = board.pins.p0_09.into_push_pull_output(Level::Low);

    rprintln!("--- Plant Watering System Initialized (Active-High Logic) ---");

    loop {
        rprintln!("\n--- Performing Moisture Check ---");
        let s1 = adc.read(&mut sensor1).unwrap();
        let s2 = adc.read(&mut sensor2).unwrap();
        let s3 = adc.read(&mut sensor3).unwrap();
        rprintln!("Sensor Values | Plant 1: {} | Plant 2: {} | Plant 3: {}", s1, s2, s3);

        let driest_plant_value = s1.max(s2).max(s3);
        rprintln!("Soil Condition | Driest plant is at: {}", driest_plant_value);

        if driest_plant_value > DRY_THRESHOLD {
            rprintln!("A plant is too dry. Starting watering process.");
            // --- LOGIC FLIPPED HERE (2/3) ---
            // Set the pin HIGH to turn the pump ON.
            pump_pin.set_high().unwrap();

            for i in 0..MAX_WATERING_CYCLES {
                rprintln!("Watering cycle {}/{}...", i + 1, MAX_WATERING_CYCLES);
                timer.delay_ms(WATERING_CYCLE_MS);

                let current_s1 = adc.read(&mut sensor1).unwrap();
                let current_s2 = adc.read(&mut sensor2).unwrap();
                let current_s3 = adc.read(&mut sensor3).unwrap();
                let wettest_plant_value = current_s1.min(current_s2).min(current_s3);
                rprintln!("Moisture check... Wettest plant is now: {}", wettest_plant_value);

                if wettest_plant_value <= WET_THRESHOLD {
                    rprintln!("A plant has reached the target moisture level.");
                    break;
                }
                if i == MAX_WATERING_CYCLES - 1 {
                    rprintln!("WARNING: Watering timeout reached!");
                }
            }

            rprintln!("Watering process complete. Stopping pump.");
            // --- LOGIC FLIPPED HERE (3/3) ---
            // Set the pin LOW to turn the pump OFF.
            pump_pin.set_low().unwrap();

        } else {
            rprintln!("All plants are sufficiently watered. No action needed.");
        }

        rprintln!("Check complete. Waiting for {} ms...", DAILY_CHECK_INTERVAL_MS);
        timer.delay_ms(DAILY_CHECK_INTERVAL_MS);
    }
}