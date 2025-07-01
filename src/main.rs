#![no_std]
#![no_main]

use cortex_m_rt::entry;
use panic_rtt_target as _;
use rtt_target::{rprintln, rtt_init_print};

use microbit::{
    board::Board,
    hal::{
        gpio::{Level, p0::P0_08}, // Use P0_08 for the physical P8 pin
        prelude::*,
        saadc::{Resolution, SaadcConfig},
        Timer,
    },
};

//================================================
// --- Watering System Configuration ---
//
// Adjust these values to fine-tune your system.
//
// --- Sensor Thresholds ---
// These values depend on your specific sensors and soil.
// Higher values mean the soil is DRIER.
//
// DRY_THRESHOLD: The ADC value above which watering will START.
const DRY_THRESHOLD: i16 = 3200;
// WET_THRESHOLD: The ADC value below which watering will STOP.
const WET_THRESHOLD: i16 = 1600;

// --- Timing Intervals (in milliseconds) ---
//
// DAILY_CHECK_INTERVAL_MS: How often to check the soil moisture.
// For testing, use a shorter interval (e.g., 30 minutes = 1_800_000).
// For production, use 24 hours (86_400_000).
const DAILY_CHECK_INTERVAL_MS: u32 = 1_800_000; // Test mode: 30 minutes

// WATERING_CYCLE_MS: The duration of each individual watering burst.
const WATERING_CYCLE_MS: u32 = 15_000; // 15 seconds

// MAX_WATERING_CYCLES: Safety limit for how many times the pump will run
// before giving up. This prevents the pump from running forever if a
// sensor fails or the water runs out.
// (10 cycles * 15s = 2.5 minutes max pump run time)
const MAX_WATERING_CYCLES: u8 = 10;
//================================================


#[entry]
fn main() -> ! {
    rtt_init_print!();
    let board = Board::take().unwrap();
    let mut timer = Timer::new(board.TIMER0);

    // --- ADC Setup for Moisture Sensors ---
    let saadc_config = SaadcConfig {
        resolution: Resolution::_12BIT,
        ..Default::default()
    };
    let mut adc = microbit::hal::saadc::Saadc::new(board.SAADC, saadc_config);

    // --- Pin Setup ---
    // Moisture Sensors
    let mut sensor1 = board.pins.p0_02.into_floating_input(); // Sensor on P0
    let mut sensor2 = board.pins.p0_03.into_floating_input(); // Sensor on P1
    let mut sensor3 = board.pins.p0_04.into_floating_input(); // Sensor on P2

    // Water Pump Relay Control Pin (P8)
    // Most relay modules are "active-low," meaning a LOW signal turns them ON.
    // We will start the pin HIGH to ensure the pump is OFF initially.
    let mut pump_pin = board.pins.p0_09.into_push_pull_output(Level::High);

    rprintln!("--- Plant Watering System Initialized ---");
    rprintln!("Dry Threshold: {}", DRY_THRESHOLD);
    rprintln!("Wet Threshold: {}", WET_THRESHOLD);
    rprintln!("Check Interval: {} ms", DAILY_CHECK_INTERVAL_MS);
    rprintln!("-----------------------------------------");


    loop {
        rprintln!("\n--- Performing Moisture Check ---");
        let s1 = adc.read(&mut sensor1).unwrap();
        let s2 = adc.read(&mut sensor2).unwrap();
        let s3 = adc.read(&mut sensor3).unwrap();
        rprintln!("Sensor Values | Plant 1: {} | Plant 2: {} | Plant 3: {}", s1, s2, s3);

        // Find the highest sensor value, which corresponds to the driest plant.
        let driest_plant_value = s1.max(s2).max(s3);
        rprintln!("Soil Condition | Driest plant is at: {}", driest_plant_value);


        // --- Watering Logic ---
        if driest_plant_value > DRY_THRESHOLD {
            rprintln!("A plant is too dry. Starting watering process.");
            // Set the pin LOW to turn the pump ON (for active-low relays).
            pump_pin.set_low().unwrap();

            // This loop will continue watering in cycles until the wettest plant
            // is moist enough OR the safety timeout is reached.
            for i in 0..MAX_WATERING_CYCLES {
                rprintln!("Watering cycle {}/{}...", i + 1, MAX_WATERING_CYCLES);
                timer.delay_ms(WATERING_CYCLE_MS);

                // Re-read sensor values to check progress.
                let current_s1 = adc.read(&mut sensor1).unwrap();
                let current_s2 = adc.read(&mut sensor2).unwrap();
                let current_s3 = adc.read(&mut sensor3).unwrap();

                // Check the WETTEST plant. We stop when at least one plant is wet enough.
                let wettest_plant_value = current_s1.min(current_s2).min(current_s3);
                rprintln!("Moisture check... Wettest plant is now: {}", wettest_plant_value);

                if wettest_plant_value <= WET_THRESHOLD {
                    rprintln!("A plant has reached the target moisture level.");
                    break; // Exit the watering loop.
                }

                // If this was the last cycle and we're still not wet enough, log a warning.
                if i == MAX_WATERING_CYCLES - 1 {
                    rprintln!("WARNING: Watering timeout reached! Check water level or sensors.");
                }
            }

            // No matter how the loop ended (success or timeout), always turn the pump OFF.
            rprintln!("Watering process complete. Stopping pump.");
            pump_pin.set_high().unwrap(); // Set HIGH to turn pump OFF.

        } else {
            rprintln!("All plants are sufficiently watered. No action needed.");
        }


        // --- Wait for the next major check cycle ---
        rprintln!("Check complete. Waiting for {} ms...", DAILY_CHECK_INTERVAL_MS);
        // This is the CORRECTED way to delay. The function takes a u32.
        timer.delay_ms(DAILY_CHECK_INTERVAL_MS);
    }
}