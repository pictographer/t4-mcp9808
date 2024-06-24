//! Temperature Monitor
//!
//! Reports the current temperature over USB-Serial. See README.md for details.
#![no_std]
#![no_main]

#[rtic::app(device = teensy4_bsp, peripherals = true, dispatchers = [KPP])]
mod app {
    use board::t40 as t4_board;
    use bsp::board;
    use bsp::{
        hal::{gpio, iomuxc},
        pins,
    };
    use imxrt_log as logging;
    use mcp9808::{reg_res::ResolutionVal, reg_temp_generic::ReadableTempRegister, MCP9808};
    use rtic_monotonics::systick::{Systick, *};
    use rtic_monotonics::Monotonic;
    use teensy4_bsp::{self as bsp};
    use teensy4_panic as _;

    // See https://www.microchip.com/en-us/product/mcp9808
    const MCP9808_T_C_MIN: f32 = -40.0; // °C
    const MCP9808_T_C_MAX: f32 = 125.0; // °C

    const T_C_DEFAULT: f32 = 30.0; // °C

    /// There are no resources shared across tasks.
    #[shared]
    struct Shared {}

    /// These resources are local to individual tasks.
    #[local]
    struct Local {
        /// The LED on Teensy pin 13.
        led: board::Led,
        /// Button to decrease the temperature alarm threshold. Momentary switch between pin 10 and 3.3v.
        down_button: gpio::Input<pins::t40::P10>,
        /// Button to increase the temperature alarm threshold. Momentary switch between pin 11 and 3.3v
        up_button: gpio::Input<pins::t40::P11>,
        /// A convenient source of 3.3v
        high_output: gpio::Output<pins::t40::P12>,
        /// A poller to enable USB logging.
        poller: logging::Poller,
        /// I2C controller for MCP9808 temperature sensor. SDC is pin 16. SDA is pin 17.
        lpi2c3: board::Lpi2c3,
    }

    #[init]
    fn init(cx: init::Context) -> (Shared, Local) {
        // Get resources from the board.
        let board::Resources {
            mut gpio2,
            mut pins,
            lpi2c3,
            usb,
            ..
        } = t4_board(cx.device);

        // Configure the pins.

        const INPUT_PULLDOWN: iomuxc::Config =
            iomuxc::Config::zero().set_pull_keeper(Some(iomuxc::PullKeeper::Pulldown100k));

        iomuxc::configure(&mut pins.p10, INPUT_PULLDOWN);
        let down_button = gpio2.input(pins.p10);

        iomuxc::configure(&mut pins.p11, INPUT_PULLDOWN);
        let up_button = gpio2.input(pins.p11);

        let high_output = gpio2.output(pins.p12);

        let led = board::led(&mut gpio2, pins.p13);

        // Prepare to poll for USB-Serial activity
        let poller = logging::log::usbd(usb, logging::Interrupts::Enabled).unwrap();
        process_inputs::spawn().unwrap();

        // Create the I2C driver
        let lpi2c3: board::Lpi2c3 =
            board::lpi2c(lpi2c3, pins.p16, pins.p17, board::Lpi2cClockSpeed::KHz400);

        // Start the system timer. If we forget this, the LED will blink SOS (...---...)
        Systick::start(
            cx.core.SYST,
            board::ARM_FREQUENCY,
            rtic_monotonics::create_systick_token!(),
        );

        (
            Shared {},
            Local {
                led,
                down_button,
                up_button,
                high_output,
                poller,
                lpi2c3,
            },
        )
    }

    // Process USB events.
    #[task(binds = USB_OTG1, local = [poller])]
    fn log_over_usb(cx: log_over_usb::Context) {
        cx.local.poller.poll();
    }

    // Mechanical switches take time to settle. Human hands sometimes twitch.
    //
    // Accepts a Boolean function that reports when the input has reached its
    // expected final state. Delays until the function returns true and a bit
    // more time has passed.
    async fn debounce_button<F>(is_pressed: F)
    where
        F: Fn() -> bool,
    {
        // Wait for the button to change state and for the contact bouncing to
        // stop.
        while is_pressed() {
            Systick::delay(20.millis()).await;
        }
        // Wait a brief lockout after a button press to avoid responding to an
        // accidental twitch of the user's finger.
        Systick::delay(30.millis()).await;
    }

    // Is the given temperature in Celsius in the sensor's range?
    fn in_temperature_sensor_range(t_c: f32) -> bool {
        MCP9808_T_C_MIN <= t_c && t_c <= MCP9808_T_C_MAX
    }

    // Poll the temperature sensor and the input buttons.
    // Report the temperature twice per second unless a button is held.
    // Light the LED if the temperature exceeds the threshold.
    // Increase or decrease the threshold when the respective buttons are pressed.
    // Issue a warning if the threshold is out of range.
    // Issue an error if the temperature reading is out of range.
    #[task(local = [led, down_button, up_button, high_output, lpi2c3])]
    async fn process_inputs(cx: process_inputs::Context) {
        let mut threshold_c = T_C_DEFAULT;
        let mut mcp9808 = MCP9808::new(cx.local.lpi2c3);
        cx.local.high_output.set();
        loop {
            if Systick::now().duration_since_epoch().to_millis() % 500 == 0 {
                // Twice per second
                match mcp9808.read_temperature() {
                    Ok(temperature) => {
                        // Light up the LED if it's hotter than the threshold.
                        let t_c = temperature.get_celsius(ResolutionVal::Deg_0_0625C);
                        if t_c > threshold_c {
                            cx.local.led.set();
                        } else {
                            cx.local.led.clear();
                        }
                        log::info!("Temperature: {t_c} °C, Threshold: {threshold_c} °C");
                        if !in_temperature_sensor_range(t_c) {
                            log::error!("Sensor reading is out of range.");
                        }
                    }
                    Err(e) => log::error!("Error: {:?}", e),
                }
                // Wait briefly to avoid doing this multiple times within a given millisecond.
                Systick::delay(1.millis()).await;
            }
            let mut changed_threshold = false;
            if cx.local.down_button.is_set() {
                debounce_button(|| cx.local.down_button.is_set()).await;
                threshold_c -= 1.0;
                changed_threshold = true;
                log::info!("Decreased alarm theshold to {threshold_c} °C.");
            }
            if cx.local.up_button.is_set() {
                debounce_button(|| cx.local.up_button.is_set()).await;
                threshold_c += 1.0;
                changed_threshold = true;
                log::info!("Increased alarm theshold to {threshold_c} °C.");
            }
            if changed_threshold && !in_temperature_sensor_range(threshold_c) {
                log::warn!(
                    "Threshold is out of range {MCP9808_T_C_MIN} °C to {MCP9808_T_C_MAX} °C."
                );
            }
        }
    }
}
