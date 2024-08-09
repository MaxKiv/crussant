#![no_std]
#![no_main]

use embedded_graphics::mono_font::ascii::FONT_6X10;
use embedded_graphics::pixelcolor::Rgb565;
use embedded_graphics::primitives::PrimitiveStyleBuilder;
use epd_waveshare::color::Black;
use epd_waveshare::epd1in54b::{Display1in54b, Epd1in54b};
use esp_hal::entry;
use esp_hal::gpio::any_pin::AnyPin;
use esp_hal::gpio::{Input, Io, Level};
use esp_hal::peripherals::Peripherals;
use esp_hal::prelude::*;
use esp_hal::spi::master::Spi;
use esp_hal::spi::SpiMode;
use esp_hal::system::SystemControl;
use esp_hal::{clock::ClockControl, delay::Delay};

use embedded_graphics::{prelude::*, primitives::Line};
use epd_waveshare::{epd1in54::*, prelude::*};

use esp_backtrace as _;

use esp_println::println;

#[entry]
fn main() -> ! {
    let peripherals = Peripherals::take();

    let system = SystemControl::new(peripherals.SYSTEM);
    let clocks = ClockControl::boot_defaults(system.clock_control).freeze();

    println!("Hello world!");

    // Initialize SPI pins
    let io = Io::new(peripherals.GPIO, peripherals.IO_MUX);
    let sclk = io.pins.gpio6; // SPI clock pin
    let mosi = io.pins.gpio7; // Master Out Slave In pin
    let miso = io.pins.gpio2; // Master In Slave Out pin

    let miso = AnyPin::new(miso);
    let mosi = AnyPin::new(mosi);

    println!("Creating spi device");
    let mut spi = Spi::new(peripherals.SPI2, 4.MHz(), SpiMode::Mode0, &clocks)
        .with_sck(sclk)
        .with_mosi(mosi)
        .with_miso(miso);

    // Initialize the rest of the required display pins
    let cs = esp_hal::gpio::Output::new(io.pins.gpio10, Level::Low);
    let busy_in = Input::new(io.pins.gpio9, esp_hal::gpio::Pull::Up); // Display busy refreshing pin: Display pull this high when its busy
    let dc = esp_hal::gpio::Output::new(io.pins.gpio0, Level::Low); // Display Data/Command pin: Determines whether the transmitted data is a command or display data.
    let rst = esp_hal::gpio::Output::new(io.pins.gpio1, Level::High); // Display reset pin: Active low pin that resets the E-ink module to known state

    let mut delay = Delay::new(&clocks);

    // Setup EPD
    println!("Setup EPD");
    let mut epd = Epd1in54b::new(&mut spi, cs, busy_in, dc, rst, &mut delay)
        .expect("issue constructing Epd1in54 driver");

    println!("Wake EPD");
    epd.wake_up(&mut spi, &mut delay).expect("wake up failed");

    delay.delay_millis(5000);

    // Clear the full screen
    println!("Clearing screen");
    epd.set_background_color(Color::White);
    epd.clear_frame(&mut spi, &mut delay)
        .expect("err clearing frame");
    epd.display_frame(&mut spi, &mut delay)
        .expect("err displaying frame");

    delay.delay_millis(5000);

    // Use display graphics from embedded-graphics
    let mut display = Display1in54b::default();
    // let style = embedded_graphics::mono_font::MonoTextStyle::new(&FONT_6X10, Rgb565::WHITE);
    // Create a text at position (20, 30) and draw it using the previously defined style
    // Text::new("Hello Rust!", Point::new(20, 30), style)
    //     .draw(&mut display)
    //     .expect("Failed to draw text");

    // // Use embedded graphics for drawing a line
    // let style = PrimitiveStyleBuilder::new()
    //     .stroke_color(Rgb565::RED)
    //     .stroke_width(1)
    //     .build();
    // let _ = Line::new(Point::new(0, 120), Point::new(0, 295))
    //     .into_styled(style)
    //     .draw(&mut display);

    // Draw some squares
    let small_buffer = [TriColor::Black.get_byte_value(); 5000]; //160x160
    epd.update_frame(&mut spi, &small_buffer, &mut delay)
        .expect("err frame update 1");

    let small_buffer = [TriColor::White.get_byte_value(); 5000]; //80x80
    epd.update_frame(&mut spi, &small_buffer, &mut delay)
        .expect("err frame update 2");

    let small_buffer = [TriColor::Chromatic.get_byte_value(); 5000]; //8x8
    epd.update_frame(&mut spi, &small_buffer, &mut delay)
        .expect("err frame update 3");

    // Display updated frame
    println!("Displaying frame update");

    // epd.update_frame(&mut spi, display.buffer(), &mut delay)
    //     .expect("epd update frame err");
    // epd.display_frame(&mut spi, &mut delay)
    //     .expect("err displaying updated frame");
    //
    delay.delay_millis(5000);

    // Set the EPD to sleep
    println!("Sleeping EPD");
    epd.sleep(&mut spi, &mut delay)
        .expect("issue setting display to sleep");

    delay.delay_millis(5000);
    loop {}
}
