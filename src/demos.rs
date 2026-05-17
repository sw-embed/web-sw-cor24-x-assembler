//! Bundled COR24 assembly example catalog for the Load-demo dropdown.
//!
//! Sources are embedded via `include_str!` from sibling
//! `sw-cor24-x-assembler/src/examples/assembler/` and (for the
//! bus-using demos) from this repo's `src/examples/`. Display names
//! are alphabetized; bus demos use a leading "I2C " / "SPI " prefix
//! so they group naturally in the dropdown.
//!
//! Each demo declares which simulated peripherals it uses via
//! `DemoConfig`; `main.rs` reads that on Assemble & Run to attach
//! only the relevant devices, so the device panels for unrelated
//! buses stay hidden during that demo.
//!
//! Adding a new example: drop the .s file, append the `Demo { ... }`
//! tuple in the alphabetical slot, and (if it touches I2C/SPI) set
//! the matching attach flag(s).

pub const DEFAULT_SOURCE: &str = include_str!(
    "../../sw-cor24-x-assembler/src/examples/assembler/button_echo.s"
);

/// Which simulated peripherals a demo expects on the bus when run.
///
/// `DemoConfig::NONE` means "purely-software demo, no buses needed"
/// — the I2C and SPI panels stay hidden. Each `true` flag attaches
/// one specific device at its canonical address; the panel for that
/// device then becomes visible while the demo runs.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct DemoConfig {
    pub attach_tmp101: bool,
    /// The Add1 test slave at I2C 0x50 — kept under the "test device"
    /// name in the UI because the slave's role is "exercise every
    /// bus-state path and add registers as we go" rather than
    /// strictly +1.
    pub attach_test_i2c: bool,
    pub attach_tmp125: bool,
    /// The SPI EchoDevice test slave — one-byte buffer that the
    /// next exchange drives on MISO. Same generalisation as the
    /// I2C test device: this is the SPI bus's "exercise every path"
    /// chip going forward.
    pub attach_test_spi: bool,
    /// DS1307 RTC at I2C 0x68. Battery-backed persistence happens
    /// in `main.rs` (localStorage); from the device's POV this is
    /// just an attachment toggle.
    pub attach_rtc: bool,
    /// SSD1306 monochrome OLED at I2C 0x3C. The 'OLED Clock' demo
    /// pairs this with the RTC, so demos may set both flags.
    pub attach_ssd1306: bool,
}

impl DemoConfig {
    pub const NONE: Self = Self {
        attach_tmp101: false,
        attach_test_i2c: false,
        attach_tmp125: false,
        attach_test_spi: false,
        attach_rtc: false,
        attach_ssd1306: false,
    };
    pub const TMP101_ONLY: Self = Self {
        attach_tmp101: true,
        ..Self::NONE
    };
    pub const TEST_I2C_ONLY: Self = Self {
        attach_test_i2c: true,
        ..Self::NONE
    };
    pub const TMP125_ONLY: Self = Self {
        attach_tmp125: true,
        ..Self::NONE
    };
    pub const TEST_SPI_ONLY: Self = Self {
        attach_test_spi: true,
        ..Self::NONE
    };
    pub const RTC_ONLY: Self = Self {
        attach_rtc: true,
        ..Self::NONE
    };
    pub const SSD1306_ONLY: Self = Self {
        attach_ssd1306: true,
        ..Self::NONE
    };
    pub const RTC_AND_SSD1306: Self = Self {
        attach_rtc: true,
        attach_ssd1306: true,
        ..Self::NONE
    };
}

pub struct Demo {
    pub name: &'static str,
    pub source: &'static str,
    pub config: DemoConfig,
}

/// All bundled examples, alphabetical by display name.
/// I2C/SPI demos use a "I2C " / "SPI " prefix to group naturally.
pub const EXAMPLES: &[Demo] = &[
    Demo { name: "Add", source: include_str!("../../sw-cor24-x-assembler/src/examples/assembler/add.s"), config: DemoConfig::NONE },
    Demo { name: "Assert", source: include_str!("../../sw-cor24-x-assembler/src/examples/assembler/assert.s"), config: DemoConfig::NONE },
    Demo { name: "Blink LED", source: include_str!("../../sw-cor24-x-assembler/src/examples/assembler/blink_led.s"), config: DemoConfig::NONE },
    Demo { name: "Button Echo", source: include_str!("../../sw-cor24-x-assembler/src/examples/assembler/button_echo.s"), config: DemoConfig::NONE },
    Demo { name: "Button Echo (MakerLisp)", source: include_str!("../../sw-cor24-x-assembler/src/examples/assembler/button_echo_makerlisp.s"), config: DemoConfig::NONE },
    Demo { name: "Comments", source: include_str!("../../sw-cor24-x-assembler/src/examples/assembler/comments.s"), config: DemoConfig::NONE },
    Demo { name: "Countdown", source: include_str!("../../sw-cor24-x-assembler/src/examples/assembler/countdown.s"), config: DemoConfig::NONE },
    Demo { name: "Echo", source: include_str!("../../sw-cor24-x-assembler/src/examples/assembler/echo.s"), config: DemoConfig::NONE },
    Demo { name: "Fibonacci", source: include_str!("../../sw-cor24-x-assembler/src/examples/assembler/fibonacci.s"), config: DemoConfig::NONE },
    Demo { name: "I2C OLED Hello", source: include_str!("../../sw-cor24-x-assembler/src/examples/assembler/i2c_ssd1306_hello.s"), config: DemoConfig::SSD1306_ONLY },
    Demo { name: "I2C OLED RTC Clock", source: include_str!("../../sw-cor24-x-assembler/src/examples/assembler/i2c_ssd1306_rtc_clock.s"), config: DemoConfig::RTC_AND_SSD1306 },
    Demo { name: "I2C RTC Read", source: include_str!("../../sw-cor24-x-assembler/src/examples/assembler/i2c_ds1307_read.s"), config: DemoConfig::RTC_ONLY },
    Demo { name: "I2C RTC Set", source: include_str!("../../sw-cor24-x-assembler/src/examples/assembler/i2c_ds1307_set.s"), config: DemoConfig::RTC_ONLY },
    Demo { name: "I2C TMP101 Read", source: include_str!("examples/tmp101_read.s"), config: DemoConfig::TMP101_ONLY },
    Demo { name: "I2C Test Device Ping", source: include_str!("../../sw-cor24-x-assembler/src/examples/assembler/i2c_add1_ping.s"), config: DemoConfig::TEST_I2C_ONLY },
    Demo { name: "Literals", source: include_str!("../../sw-cor24-x-assembler/src/examples/assembler/literals.s"), config: DemoConfig::NONE },
    Demo { name: "Loop Trace", source: include_str!("../../sw-cor24-x-assembler/src/examples/assembler/loop_trace.s"), config: DemoConfig::NONE },
    Demo { name: "Memory Access", source: include_str!("../../sw-cor24-x-assembler/src/examples/assembler/memory_access.s"), config: DemoConfig::NONE },
    Demo { name: "Multiply", source: include_str!("../../sw-cor24-x-assembler/src/examples/assembler/multiply.s"), config: DemoConfig::NONE },
    Demo { name: "Nested Calls", source: include_str!("../../sw-cor24-x-assembler/src/examples/assembler/nested_calls.s"), config: DemoConfig::NONE },
    Demo { name: "SPI Echo Ping", source: include_str!("examples/spi_echo_ping.s"), config: DemoConfig::TEST_SPI_ONLY },
    Demo { name: "SPI TMP125 Read", source: include_str!("examples/tmp125_read.s"), config: DemoConfig::TMP125_ONLY },
    Demo { name: "Stack Variables", source: include_str!("../../sw-cor24-x-assembler/src/examples/assembler/stack_variables.s"), config: DemoConfig::NONE },
    Demo { name: "UART Hello", source: include_str!("../../sw-cor24-x-assembler/src/examples/assembler/uart_hello.s"), config: DemoConfig::NONE },
    Demo { name: "Variables", source: include_str!("../../sw-cor24-x-assembler/src/examples/assembler/variables.s"), config: DemoConfig::NONE },
];

/// Look up a bundled example by display name.
pub fn lookup(name: &str) -> Option<&'static Demo> {
    EXAMPLES.iter().find(|d| d.name == name)
}
