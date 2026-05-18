//! web-sw-cor24-x-assembler — Yew app entry point.
//!
//! Layout (mirrors web-sw-cor24-x-tinyc): three columns — source editor,
//! assembled listing, emulator I/O panel. The "Assemble & Run" button
//! drives the cor24-assembler over the source, loads the bytes into a
//! cor24-emulator EmulatorCore, and ticks an Interval to advance the
//! run loop while keeping the UI responsive.

mod assembler;
mod battery;
mod demos;
mod editor;
mod highlight;
mod idb;
mod panels;

use std::cell::RefCell;
use std::rc::Rc;

use cor24_assembler::AssembledLine;
use cor24_emulator::EmulatorCore;
use cor24_emulator::peripherals::i2c::{
    Add1Device, Ds1307Device, Ds1307HandleExt, I2cDevice, I2cHandle, Ssd1306Device, Tmp101Device,
    Tmp101HandleExt, Tmp101Resolution,
};
use cor24_emulator::peripherals::spi::{
    EchoDevice, SdCardDevice, SdCardHandleExt, SpiHandle, Tmp125Device, Tmp125HandleExt,
    W25q32Device, W25q32HandleExt,
};
use js_sys::Uint8Array;
use wasm_bindgen::JsCast;
use wasm_bindgen::closure::Closure;
use wasm_bindgen_futures::spawn_local;
use web_sys::{File, FileReader, HtmlSelectElement, KeyboardEvent};
use yew::prelude::*;

use editor::Editor;
use panels::{
    BusSnapshot, Ds1307Snapshot, EchoSnapshot, I2cPanel, LedPanel, RegistersPanel, SdCardSnapshot,
    SpiBusSnapshot, SpiPanel, Ssd1306Snapshot, SwitchPanel, TestDeviceSnapshot, Tmp101Snapshot,
    Tmp125Snapshot, UartPanel, W25q32Snapshot,
};

/// Bundled default SD card image: 4 KiB blob with recognizable
/// per-sector patterns (sector 0 starts with `00 01 02 ...`,
/// sector 1 with `10 11 12 ...`, etc.). Loaded as the initial
/// image when no user upload is persisted in IndexedDB.
const SDCARD_DEFAULT_IMAGE: &[u8] = include_bytes!("../static/sdcard-default.img");

/// IDB key for the persisted SD card image.
const SDCARD_IDB_KEY: &str = "sdcard.image";

/// IDB key for the persisted W25Q32 NOR flash image.
const W25Q32_IDB_KEY: &str = "w25q32.image";
/// W25Q32 sector size (4 KiB). Mirrors the constant in the emulator
/// but we don't import it to avoid a dependency on the device's
/// internal layout module.
const W25Q32_SECTOR_SIZE: usize = 4 * 1024;
/// 1024 sectors in 4 MiB.
const W25Q32_SECTOR_COUNT: usize = 1024;
/// Per-tick decay applied to each sector_activity counter. 4 ticks
/// = ~64ms means a freshly-written sector visibly cools over ~4s.
const W25Q32_HEAT_DECAY: u8 = 4;
/// Heat bump on a fresh write/erase: saturate to 255.
const W25Q32_HEAT_BUMP: u8 = 255;

#[function_component(App)]
fn app() -> Html {
    let source = use_state(|| demos::DEFAULT_SOURCE.to_string());

    // Bus-device attach configuration. Starts at NONE (the default
    // demo, button_echo, doesn't touch any bus); each demo selected
    // from the dropdown updates this to its declared `DemoConfig`.
    // The user's own typed source uses whatever was last loaded.
    let demo_config = use_state(|| demos::DemoConfig::NONE);

    // Assembly state
    let listing = use_state(Vec::<AssembledLine>::new);
    let assemble_error = use_state(|| None::<assembler::AssembleError>);

    // Emulator (mutable ref, survives re-renders)
    let emu: Rc<RefCell<EmulatorCore>> = use_mut_ref(EmulatorCore::new);

    // Emulator display state (updated each tick)
    let uart_output = use_state(String::new);
    let registers = use_state(|| [0u32; 8]);
    let pc_val = use_state(|| 0u32);
    let cond_flag = use_state(|| false);
    let led_state = use_state(|| 0u8);
    let running = use_state(|| false);
    let halted = use_state(|| false);
    let instr_count = use_state(|| 0u64);
    let status_msg = use_state(|| String::from("Ready"));
    let runtime_error_line = use_state(|| None::<usize>);

    // Switch S2
    let switch_pressed = use_state(|| false);

    // I2C device handles + per-tick snapshots. A fresh TMP101 is
    // attached at each Assemble & Run so the typed handle's weak ref
    // to the bus's routing table doesn't outlive the EmulatorCore
    // that owns it.
    let tmp101_handle: Rc<RefCell<Option<I2cHandle<Tmp101Device>>>> = use_mut_ref(|| None);
    let test_device_handle: Rc<RefCell<Option<I2cHandle<Add1Device>>>> = use_mut_ref(|| None);
    let ds1307_handle: Rc<RefCell<Option<I2cHandle<Ds1307Device>>>> = use_mut_ref(|| None);
    let ssd1306_handle: Rc<RefCell<Option<I2cHandle<Ssd1306Device>>>> = use_mut_ref(|| None);
    let bus_snapshot = use_state(BusSnapshot::default);
    let tmp101_snapshot = use_state(|| None::<Tmp101Snapshot>);
    let test_device_snapshot = use_state(|| None::<TestDeviceSnapshot>);
    let ds1307_snapshot = use_state(|| None::<Ds1307Snapshot>);
    let ssd1306_snapshot = use_state(|| None::<Ssd1306Snapshot>);
    // Battery toggle is feature-shaped: lives in the web layer +
    // localStorage; never crosses the emulator boundary.
    let ds1307_battery_enabled = use_state(battery::load_enabled);
    // Last (h,m,s) tuple this tick saw — used by THE SYNCHRONIZATION
    // TRAP to detect i2c-write completion and persist to
    // localStorage as soon as it lands.
    let ds1307_last_seen: Rc<RefCell<Option<(u8, u8, u8)>>> = use_mut_ref(|| None);
    // Wall-clock anchor for the auto-tick pump. Each real-world
    // second elapsed during a Run drives one `tick_second()` on
    // the attached chip so the RTC behaves like a real powered
    // DS1307 (continuous tick) rather than a frozen-at-attach
    // snapshot. The previous behavior left the OLED RTC Clock demo
    // showing whatever value the chip held at attach, which read
    // as a panel-vs-OLED bug when wall-clock time advanced.
    let ds1307_last_tick_ms: Rc<RefCell<f64>> = use_mut_ref(|| 0.0_f64);

    // SPI side: single-slave; one of TMP125, EchoDevice, or SdCard
    // is attached per the demo's config.
    let tmp125_handle: Rc<RefCell<Option<SpiHandle<Tmp125Device>>>> = use_mut_ref(|| None);
    let echo_handle: Rc<RefCell<Option<SpiHandle<EchoDevice>>>> = use_mut_ref(|| None);
    let sdcard_handle: Rc<RefCell<Option<SpiHandle<SdCardDevice>>>> = use_mut_ref(|| None);
    let w25q32_handle: Rc<RefCell<Option<SpiHandle<W25q32Device>>>> = use_mut_ref(|| None);
    let spi_bus_snapshot = use_state(SpiBusSnapshot::default);
    let tmp125_snapshot = use_state(|| None::<Tmp125Snapshot>);
    let echo_snapshot = use_state(|| None::<EchoSnapshot>);
    let sdcard_snapshot = use_state(|| None::<SdCardSnapshot>);
    let w25q32_snapshot = use_state(|| None::<W25q32Snapshot>);
    // SD card image bytes that the next Run will attach the chip
    // with. Starts as the bundled default; an IDB load on mount may
    // replace it with the user's prior upload; a file-upload or
    // "Reset to default" replaces it inline. The persistence trap
    // writes through to IDB on each successful upload / reset.
    let sdcard_image: Rc<RefCell<Vec<u8>>> = use_mut_ref(|| SDCARD_DEFAULT_IMAGE.to_vec());
    // W25Q32 image bytes -- same shape as the SD card. Default is
    // 4 MiB of 0xFF (factory-erased flash).
    let w25q32_image: Rc<RefCell<Vec<u8>>> = use_mut_ref(|| vec![0xFFu8; 4 * 1024 * 1024]);
    // Per-tick heatmap state. activity[i] is the cooling counter
    // for sector i; last_addr is the previous tick's read of the
    // chip's last_accessed_address, used to detect rising-edge
    // changes for the heat bump. Kept as Rc<RefCell> so the tick
    // loop, the upload callback, and the reset callback all mutate
    // a single shared buffer.
    let w25q32_activity: Rc<RefCell<Vec<u8>>> =
        use_mut_ref(|| vec![0u8; W25Q32_SECTOR_COUNT]);
    let w25q32_last_addr: Rc<RefCell<Option<u32>>> = use_mut_ref(|| None);
    // Per-tick "is this sector all 0xFF?" cache. Initial: every
    // sector erased (matches the 0xFF default image). Recomputed
    // on writes via the tick-loop heat trap.
    let w25q32_erased: Rc<RefCell<Vec<bool>>> =
        use_mut_ref(|| vec![true; W25Q32_SECTOR_COUNT]);

    // One-time IDB restore on mount: if a prior session uploaded
    // an image, swap it into the corresponding image-bytes ref so
    // the next Assemble & Run picks it up. Cheap when IDB is empty
    // (single readonly transaction returns null).
    {
        let sdcard_image = sdcard_image.clone();
        let w25q32_image = w25q32_image.clone();
        let w25q32_erased = w25q32_erased.clone();
        use_effect_with((), move |_| {
            spawn_local(async move {
                if let Some(bytes) = idb::get(SDCARD_IDB_KEY).await {
                    *sdcard_image.borrow_mut() = bytes;
                }
                if let Some(bytes) = idb::get(W25Q32_IDB_KEY).await {
                    let erased = compute_w25q32_erased(&bytes);
                    *w25q32_image.borrow_mut() = bytes;
                    *w25q32_erased.borrow_mut() = erased;
                }
            });
            || ()
        });
    }

    // UART input buffer (keyboard → emulator, drained in run loop)
    let uart_input: Rc<RefCell<std::collections::VecDeque<u8>>> =
        use_mut_ref(std::collections::VecDeque::new);

    // Interval handle
    let interval_handle = use_mut_ref(|| None::<gloo_timers::callback::Interval>);

    // --- Callbacks ---

    let on_source_change = {
        let source = source.clone();
        Callback::from(move |value: String| source.set(value))
    };

    let on_run = {
        let source = source.clone();
        let listing = listing.clone();
        let assemble_error = assemble_error.clone();
        let emu = emu.clone();
        let uart_input = uart_input.clone();
        let uart_output = uart_output.clone();
        let registers = registers.clone();
        let pc_val = pc_val.clone();
        let cond_flag = cond_flag.clone();
        let led_state = led_state.clone();
        let running = running.clone();
        let halted = halted.clone();
        let instr_count = instr_count.clone();
        let status_msg = status_msg.clone();
        let runtime_error_line = runtime_error_line.clone();
        let interval_handle = interval_handle.clone();
        let switch_pressed = switch_pressed.clone();
        let tmp101_handle = tmp101_handle.clone();
        let test_device_handle = test_device_handle.clone();
        let ds1307_handle = ds1307_handle.clone();
        let ssd1306_handle = ssd1306_handle.clone();
        let bus_snapshot = bus_snapshot.clone();
        let tmp101_snapshot = tmp101_snapshot.clone();
        let test_device_snapshot = test_device_snapshot.clone();
        let ds1307_snapshot = ds1307_snapshot.clone();
        let ssd1306_snapshot = ssd1306_snapshot.clone();
        let ds1307_battery_enabled = ds1307_battery_enabled.clone();
        let ds1307_last_seen = ds1307_last_seen.clone();
        let ds1307_last_tick_ms = ds1307_last_tick_ms.clone();
        let tmp125_handle = tmp125_handle.clone();
        let echo_handle = echo_handle.clone();
        let sdcard_handle = sdcard_handle.clone();
        let w25q32_handle = w25q32_handle.clone();
        let spi_bus_snapshot = spi_bus_snapshot.clone();
        let tmp125_snapshot = tmp125_snapshot.clone();
        let echo_snapshot = echo_snapshot.clone();
        let sdcard_snapshot = sdcard_snapshot.clone();
        let w25q32_snapshot = w25q32_snapshot.clone();
        let sdcard_image = sdcard_image.clone();
        let w25q32_image = w25q32_image.clone();
        let w25q32_activity = w25q32_activity.clone();
        let w25q32_last_addr = w25q32_last_addr.clone();
        let w25q32_erased = w25q32_erased.clone();
        let demo_config = demo_config.clone();

        Callback::from(move |()| {
            // Stop any existing run loop.
            *interval_handle.borrow_mut() = None;
            runtime_error_line.set(None);
            let config = *demo_config;

            // Two source flavors: COR24 assembly (.s) goes through
            // cor24-assembler; an .lgo load-file (any line starting
            // with `L<6-hex>`) bypasses the assembler and loads
            // straight into the emulator. Listing is only meaningful
            // for the .s path.
            let lgo_mode = assembler::looks_like_lgo(&source);
            let bytes: Option<Vec<u8>> = if lgo_mode {
                listing.set(Vec::new());
                None
            } else {
                let output = assembler::assemble(&source);
                listing.set(output.listing.clone());
                if let Some(err) = output.error {
                    assemble_error.set(Some(err));
                    running.set(false);
                    halted.set(false);
                    status_msg.set("Assembly error".into());
                    return;
                }
                Some(output.bytes)
            };
            assemble_error.set(None);

            // Reset emulator, load program (assembled bytes or .lgo),
            // attach only the devices the current demo declares it
            // needs (via DemoConfig). Devices not attached this run
            // have their snapshot cleared so the matching panels
            // stay hidden -- so picking 'I2C TMP101 Read' shows only
            // the TMP101 card, not the unrelated test-device row
            // or the SPI bus.
            let (h_tmp101, h_test, h_ds1307, h_ssd1306, h_tmp125, h_echo, h_sdcard, h_w25q32) = {
                let mut e = emu.borrow_mut();
                *e = EmulatorCore::new();
                match &bytes {
                    Some(bs) => {
                        e.load_program(0, bs);
                        e.load_program_extent(bs.len() as u32);
                    }
                    None => {
                        if let Err(msg) = e.load_lgo(&source, None) {
                            assemble_error.set(Some(assembler::AssembleError {
                                message: format!(".lgo load failed: {msg}"),
                                line: None,
                            }));
                            running.set(false);
                            halted.set(false);
                            status_msg.set(".lgo error".into());
                            return;
                        }
                    }
                }
                e.set_button_pressed(*switch_pressed);

                let h_tmp101 = if config.attach_tmp101 {
                    let h = e
                        .attach_i2c_device(Tmp101Device::new(
                            cor24_emulator::peripherals::i2c::devices::tmp101::DEFAULT_ADDRESS,
                        ))
                        .expect("TMP101 default address free on a fresh bus");
                    // Seed the freshly-attached device from the slider's
                    // pre-Run value so dragging the slider before Run
                    // takes effect on the very first read.
                    if let Some(snap) = *tmp101_snapshot {
                        h.set_temperature(snap.temperature_c);
                    }
                    Some(h)
                } else {
                    None
                };
                let h_test = if config.attach_test_i2c {
                    // Add1 test slave at 0x50 — the slot the bundled
                    // 'I2C Test Device Ping' demo addresses. The chip
                    // is named "test device" in the UI because its
                    // eventual role is "exercise every bus path; gain
                    // registers as we go", not strictly add-one.
                    let h = e
                        .attach_i2c_device(Add1Device::new(0x50, 0))
                        .expect("test-device address 0x50 free on a fresh bus");
                    if let Some(snap) = *test_device_snapshot {
                        h.with(|d| d.poke(snap.last_byte));
                    }
                    Some(h)
                } else {
                    None
                };
                // DS1307 attach path matrix:
                //   battery off              -> all-zero regs.
                //   battery on, no persisted -> save fresh anchor
                //                                {00:00:00, now} and
                //                                attach from it.
                //                                Subsequent Runs (or
                //                                page reloads) then
                //                                see wall-clock
                //                                advance via
                //                                effective_now().
                //   battery on, persisted    -> effective time-of-day
                //                                from the anchor.
                //
                // The "save anchor on first Run" path is a deliberate
                // divergence from the original brief's matrix (which
                // said no-persisted-state -> all-zero regs). With the
                // brief's path, battery-on was indistinguishable from
                // battery-off until the user explicitly ran I2C RTC
                // Set -- which doesn't match a user's mental model of
                // a battery-backed RTC ("the chip is alive, time
                // passes, even if I never explicitly set it"). The
                // anchor lives in the same `ds1307.battery`
                // localStorage slot as an explicit Set, just seeded
                // at {0:0:0, attach-time} when nothing's there yet.
                let h_ds1307 = if config.attach_rtc {
                    let device = if *ds1307_battery_enabled {
                        let now_ms = js_sys::Date::now();
                        let persisted = battery::load().unwrap_or_else(|| {
                            let fresh = battery::Persisted {
                                set_value: battery::SetValue { h: 0, m: 0, s: 0 },
                                set_at_ms: now_ms,
                            };
                            battery::save(fresh);
                            fresh
                        });
                        let eff = battery::effective_now(persisted, now_ms);
                        let mut regs = [0u8; 8];
                        regs[0] = int_to_bcd(eff.s);
                        regs[1] = int_to_bcd(eff.m);
                        regs[2] = int_to_bcd(eff.h);
                        // date fields stay zero (persistence is
                        // time-of-day only per the brief's % 86400).
                        Ds1307Device::with_initial_registers(0x68, regs)
                    } else {
                        Ds1307Device::new(0x68)
                    };
                    let h = e
                        .attach_i2c_device(device)
                        .expect("DS1307 default address 0x68 free on a fresh bus");
                    Some(h)
                } else {
                    None
                };
                let h_ssd1306 = if config.attach_ssd1306 {
                    // SSD1306 default address; demos pair with the
                    // DS1307 (which is at 0x68) on the I2C OLED RTC
                    // Clock demo, so the two slaves coexist.
                    let h = e
                        .attach_i2c_device(Ssd1306Device::new(
                            cor24_emulator::peripherals::i2c::devices::ssd1306::DEFAULT_ADDRESS,
                        ))
                        .expect("SSD1306 default address 0x3C free on a fresh bus");
                    Some(h)
                } else {
                    None
                };
                // SPI is single-slave today, so TMP125 and the
                // echo test device are mutually exclusive.
                let h_tmp125 = if config.attach_tmp125 {
                    let h = e.attach_spi_device(Tmp125Device::new());
                    if let Some(snap) = *tmp125_snapshot {
                        h.set_temperature(snap.temperature_c);
                    }
                    Some(h)
                } else {
                    None
                };
                let h_echo = if config.attach_test_spi {
                    let seed = (*echo_snapshot).map(|s| s.buffer).unwrap_or(0);
                    Some(e.attach_spi_device(EchoDevice::new(seed)))
                } else {
                    None
                };
                // SD card: attach with whichever image bytes the panel
                // currently has (IDB-restored upload or bundled default).
                // The image is cloned because the device owns its
                // Vec<u8>; on subsequent uploads the panel pushes new
                // bytes into both `sdcard_image` and the live handle
                // via `replace_image`.
                let h_sdcard = if config.attach_sdcard {
                    let image = sdcard_image.borrow().clone();
                    Some(e.attach_spi_device(SdCardDevice::with_image(image, None, 2)))
                } else {
                    None
                };
                // W25Q32: attach with whichever 4 MiB image bytes the
                // panel currently has (IDB-restored or fresh-erased
                // 0xFF default). The emulator normalizes any length
                // to exactly 4 MiB so a short / overlong upload is
                // padded or truncated transparently.
                let h_w25q32 = if config.attach_w25q32 {
                    let image = w25q32_image.borrow().clone();
                    Some(e.attach_spi_device(W25q32Device::with_image(image, None, 3)))
                } else {
                    None
                };

                e.resume();
                (
                    h_tmp101, h_test, h_ds1307, h_ssd1306, h_tmp125, h_echo, h_sdcard, h_w25q32,
                )
            };

            // Seed each device-card snapshot if its device was
            // attached; otherwise clear the snapshot so the panel
            // hides for this run.
            if let Some(h) = &h_tmp101 {
                tmp101_snapshot.set(Some(read_tmp101_snapshot(h)));
            } else {
                tmp101_snapshot.set(None);
            }
            *tmp101_handle.borrow_mut() = h_tmp101;
            if let Some(h) = &h_test {
                test_device_snapshot.set(Some(read_test_device_snapshot(h)));
            } else {
                test_device_snapshot.set(None);
            }
            *test_device_handle.borrow_mut() = h_test;
            // RTC: seed snapshot + reset the synchronization-trap
            // last-seen tracker so the very first detected write
            // after Run reaches localStorage.
            if let Some(h) = &h_ds1307 {
                let snap = read_ds1307_snapshot(h);
                ds1307_snapshot.set(Some(snap));
                *ds1307_last_seen.borrow_mut() = Some((snap.hour, snap.minute, snap.second));
                *ds1307_last_tick_ms.borrow_mut() = js_sys::Date::now();
            } else {
                ds1307_snapshot.set(None);
                *ds1307_last_seen.borrow_mut() = None;
                *ds1307_last_tick_ms.borrow_mut() = 0.0;
            }
            *ds1307_handle.borrow_mut() = h_ds1307;
            if let Some(h) = &h_ssd1306 {
                ssd1306_snapshot.set(Some(read_ssd1306_snapshot(h)));
            } else {
                ssd1306_snapshot.set(None);
            }
            *ssd1306_handle.borrow_mut() = h_ssd1306;
            if let Some(h) = &h_tmp125 {
                tmp125_snapshot.set(Some(read_tmp125_snapshot(h)));
            } else {
                tmp125_snapshot.set(None);
            }
            *tmp125_handle.borrow_mut() = h_tmp125;
            if let Some(h) = &h_echo {
                echo_snapshot.set(Some(read_echo_snapshot(h)));
            } else {
                echo_snapshot.set(None);
            }
            *echo_handle.borrow_mut() = h_echo;
            if let Some(h) = &h_sdcard {
                sdcard_snapshot.set(Some(read_sdcard_snapshot(h)));
            } else {
                sdcard_snapshot.set(None);
            }
            *sdcard_handle.borrow_mut() = h_sdcard;
            // W25Q32: clear the per-tick heatmap state so a new Run
            // starts cool; the erased-flag cache stays correct
            // because it reflects image bytes, not per-Run history.
            if let Some(h) = &h_w25q32 {
                *w25q32_activity.borrow_mut() = vec![0u8; W25Q32_SECTOR_COUNT];
                *w25q32_last_addr.borrow_mut() = None;
                w25q32_snapshot.set(Some(read_w25q32_snapshot(
                    h,
                    &w25q32_activity.borrow(),
                    &w25q32_erased.borrow(),
                )));
            } else {
                w25q32_snapshot.set(None);
            }
            *w25q32_handle.borrow_mut() = h_w25q32;

            // Reset display state.
            uart_output.set(String::new());
            registers.set([0u32; 8]);
            pc_val.set(0);
            cond_flag.set(false);
            led_state.set(0);
            halted.set(false);
            instr_count.set(0);
            status_msg.set("Running".into());
            running.set(true);
            bus_snapshot.set(BusSnapshot::default());
            spi_bus_snapshot.set(SpiBusSnapshot::default());

            // Clear input buffer.
            uart_input.borrow_mut().clear();

            // Start run loop.
            let emu = emu.clone();
            let uart_input = uart_input.clone();
            let uart_output = uart_output.clone();
            let registers = registers.clone();
            let pc_val = pc_val.clone();
            let cond_flag = cond_flag.clone();
            let led_state = led_state.clone();
            let running = running.clone();
            let halted = halted.clone();
            let instr_count = instr_count.clone();
            let status_msg = status_msg.clone();
            let runtime_error_line = runtime_error_line.clone();
            let listing = listing.clone();
            let interval_handle2 = interval_handle.clone();
            let tmp101_handle = tmp101_handle.clone();
            let test_device_handle = test_device_handle.clone();
            let ds1307_handle = ds1307_handle.clone();
            let ssd1306_handle = ssd1306_handle.clone();
            let bus_snapshot = bus_snapshot.clone();
            let tmp101_snapshot = tmp101_snapshot.clone();
            let test_device_snapshot = test_device_snapshot.clone();
            let ds1307_snapshot = ds1307_snapshot.clone();
            let ssd1306_snapshot = ssd1306_snapshot.clone();
            let ds1307_battery_enabled = ds1307_battery_enabled.clone();
            let ds1307_last_seen = ds1307_last_seen.clone();
            let ds1307_last_tick_ms = ds1307_last_tick_ms.clone();
            let tmp125_handle = tmp125_handle.clone();
            let echo_handle = echo_handle.clone();
            let sdcard_handle = sdcard_handle.clone();
            let w25q32_handle = w25q32_handle.clone();
            let spi_bus_snapshot = spi_bus_snapshot.clone();
            let tmp125_snapshot = tmp125_snapshot.clone();
            let echo_snapshot = echo_snapshot.clone();
            let sdcard_snapshot = sdcard_snapshot.clone();
            let w25q32_snapshot = w25q32_snapshot.clone();
            let w25q32_image = w25q32_image.clone();
            let w25q32_activity = w25q32_activity.clone();
            let w25q32_last_addr = w25q32_last_addr.clone();
            let w25q32_erased = w25q32_erased.clone();

            let interval = gloo_timers::callback::Interval::new(16, move || {
                let mut e = emu.borrow_mut();

                // Drain keyboard input buffer into UART RX when free.
                {
                    let mut buf = uart_input.borrow_mut();
                    if !buf.is_empty()
                        && (e.read_byte(0xFF0101) & 0x01 == 0)
                        && let Some(byte) = buf.pop_front()
                    {
                        e.send_uart_byte(byte);
                    }
                }

                // Adaptive per-tick budget. The prior static 100k/tick
                // was right for the bundled demos but wrong for any
                // user-pasted source with a non-trivial idle loop
                // (the upstream tmp101.lgo's `t = -1; while (t--){}`
                // delay was the example that motivated the earlier
                // 1M tuning bump, which then made tight loops jank
                // -- two static values and neither worked everywhere).
                //
                // Instead: run 50k-instruction chunks back-to-back,
                // stop when ~8 ms of wall-clock has elapsed or the
                // emulator halts/faults. So tight loops complete
                // hundreds of thousands of instructions a tick while
                // slow ones still pump enough to make visible
                // progress, and the UI thread stays responsive
                // because no tick blocks for longer than the
                // deadline. Falls back to a single 100k-instruction
                // chunk if Performance is unavailable.
                const CHUNK: u64 = 50_000;
                const DEADLINE_MS: f64 = 8.0;
                let perf = web_sys::window().and_then(|w| w.performance());
                let deadline = perf.as_ref().map(|p| p.now() + DEADLINE_MS);
                let mut batch = e.run_batch(if deadline.is_some() {
                    CHUNK
                } else {
                    100_000
                });
                while matches!(batch.reason, cor24_emulator::StopReason::CycleLimit) {
                    if let (Some(p), Some(d)) = (perf.as_ref(), deadline)
                        && p.now() >= d
                    {
                        break;
                    } else if deadline.is_none() {
                        break;
                    }
                    let next = e.run_batch(CHUNK);
                    batch = cor24_emulator::BatchResult {
                        instructions_run: batch
                            .instructions_run
                            .saturating_add(next.instructions_run),
                        reason: next.reason,
                        uart_bytes_added: batch.uart_bytes_added + next.uart_bytes_added,
                        led_changed: batch.led_changed || next.led_changed,
                    };
                }

                // Update display state.
                uart_output.set(e.get_uart_output().to_string());
                let mut regs = [0u32; 8];
                for (i, reg) in regs.iter_mut().enumerate() {
                    *reg = e.get_reg(i as u8);
                }
                registers.set(regs);
                pc_val.set(e.pc());
                cond_flag.set(e.condition_flag());
                led_state.set(e.get_led());
                instr_count.set(e.instructions_count());

                // Update I2C bus header + per-device snapshots.
                let bus = e.i2c();
                bus_snapshot.set(BusSnapshot {
                    idle: bus.phase == cor24_emulator::cpu::i2c_bus::I2cPhase::Idle,
                    last_byte: bus.last_byte,
                    last_addressed: bus.last_addressed,
                    transactions: bus.transactions,
                    attached: bus.addresses.len(),
                });
                if let Some(h) = tmp101_handle.borrow().as_ref() {
                    tmp101_snapshot.set(Some(read_tmp101_snapshot(h)));
                }
                if let Some(h) = test_device_handle.borrow().as_ref() {
                    test_device_snapshot.set(Some(read_test_device_snapshot(h)));
                }
                if let Some(h) = ssd1306_handle.borrow().as_ref() {
                    ssd1306_snapshot.set(Some(read_ssd1306_snapshot(h)));
                }
                if let Some(h) = ds1307_handle.borrow().as_ref() {
                    // Order matters: trap fires on the *pre-tick*
                    // snapshot so external writes (Set demo, Set To
                    // System Time button) get persisted, then the
                    // auto-tick pump advances the chip silently, then
                    // last_seen syncs to the post-tick value so the
                    // next tick's trap sees no spurious change.
                    let pre = read_ds1307_snapshot(h);
                    let pre_tuple = (pre.hour, pre.minute, pre.second);
                    {
                        let mut last = ds1307_last_seen.borrow_mut();
                        if *last != Some(pre_tuple) {
                            if *ds1307_battery_enabled && last.is_some() {
                                // Boot-time guard preserved: only real
                                // user-write crossings persist.
                                battery::save(battery::Persisted {
                                    set_value: battery::SetValue {
                                        h: pre.hour,
                                        m: pre.minute,
                                        s: pre.second,
                                    },
                                    set_at_ms: js_sys::Date::now(),
                                });
                            }
                            *last = Some(pre_tuple);
                        }
                    }

                    // Auto-tick once per real-world second; catch up
                    // if the browser deprioritized us. tick_second()
                    // cascades through minutes/hours/etc.
                    let now_ms = js_sys::Date::now();
                    {
                        let mut anchor = ds1307_last_tick_ms.borrow_mut();
                        while now_ms - *anchor >= 1000.0 {
                            h.tick_second();
                            *anchor += 1000.0;
                        }
                    }

                    // Sync last_seen to post-tick value so the auto-
                    // tick advance isn't mistaken for a user write
                    // on the next tick.
                    let post = read_ds1307_snapshot(h);
                    *ds1307_last_seen.borrow_mut() = Some((post.hour, post.minute, post.second));
                    ds1307_snapshot.set(Some(post));
                }

                // Update SPI bus header + TMP125 snapshot.
                let spi = e.spi();
                spi_bus_snapshot.set(SpiBusSnapshot {
                    selected: !spi.last_seln,
                    last_mosi: spi.last_mosi_byte,
                    last_miso: spi.last_miso_byte,
                    bytes_exchanged: spi.bytes_exchanged,
                    attached: spi.device.is_some(),
                });
                if let Some(h) = tmp125_handle.borrow().as_ref() {
                    tmp125_snapshot.set(Some(read_tmp125_snapshot(h)));
                }
                if let Some(h) = echo_handle.borrow().as_ref() {
                    echo_snapshot.set(Some(read_echo_snapshot(h)));
                }
                if let Some(h) = sdcard_handle.borrow().as_ref() {
                    sdcard_snapshot.set(Some(read_sdcard_snapshot(h)));
                }
                if let Some(h) = w25q32_handle.borrow().as_ref() {
                    // W25Q32 trap: detect address-change crossings to
                    // bump the heatmap; decay all sectors each tick;
                    // on a write (image content changed), persist to
                    // IDB + recompute the erased-flag cache. The
                    // address watch is rising-edge-only because the
                    // emulator's `last_accessed_address` sticks
                    // forever after the first access -- without an
                    // edge detect we'd re-bump the same sector each
                    // tick and the heatmap would never cool.
                    let cur_addr = h.last_accessed_address();
                    let prev_addr = *w25q32_last_addr.borrow();
                    let address_changed = cur_addr != prev_addr;
                    if address_changed {
                        if let Some(a) = cur_addr {
                            let sector = (a as usize / W25Q32_SECTOR_SIZE).min(W25Q32_SECTOR_COUNT - 1);
                            w25q32_activity.borrow_mut()[sector] = W25Q32_HEAT_BUMP;
                        }
                        *w25q32_last_addr.borrow_mut() = cur_addr;
                    }
                    // Decay all sectors. Saturating subtract keeps
                    // cold sectors at 0.
                    for a in w25q32_activity.borrow_mut().iter_mut() {
                        *a = a.saturating_sub(W25Q32_HEAT_DECAY);
                    }
                    // Persist image if anything changed (any active
                    // sector means a recent program/erase). Cheap
                    // when WIP is idle and no sector is warm.
                    if h.wip()
                        || w25q32_activity
                            .borrow()
                            .iter()
                            .any(|&v| v > W25Q32_HEAT_BUMP - W25Q32_HEAT_DECAY)
                    {
                        let new_image = h.image();
                        let new_erased = compute_w25q32_erased(&new_image);
                        *w25q32_erased.borrow_mut() = new_erased;
                        *w25q32_image.borrow_mut() = new_image.clone();
                        spawn_local(async move {
                            idb::put(W25Q32_IDB_KEY, &new_image).await;
                        });
                    }
                    w25q32_snapshot.set(Some(read_w25q32_snapshot(
                        h,
                        &w25q32_activity.borrow(),
                        &w25q32_erased.borrow(),
                    )));
                }

                let stop = match batch.reason {
                    cor24_emulator::StopReason::Halted => {
                        halted.set(true);
                        status_msg.set("Halted".into());
                        true
                    }
                    cor24_emulator::StopReason::InvalidInstruction(op) => {
                        let pc = e.pc();
                        let line = assembler::pc_to_listing_line(&listing, pc);
                        runtime_error_line.set(line);
                        halted.set(true);
                        status_msg.set(format!(
                            "Invalid instruction: {op:#04x} at PC={pc:#06x}"
                        ));
                        true
                    }
                    cor24_emulator::StopReason::Paused => {
                        status_msg.set("Paused".into());
                        true
                    }
                    _ => false,
                };

                if stop {
                    running.set(false);
                    *interval_handle2.borrow_mut() = None;
                }
            });

            *interval_handle.borrow_mut() = Some(interval);
        })
    };

    let on_stop = {
        let emu = emu.clone();
        let interval_handle = interval_handle.clone();
        let running = running.clone();
        let status_msg = status_msg.clone();
        Callback::from(move |_: MouseEvent| {
            emu.borrow_mut().pause();
            *interval_handle.borrow_mut() = None;
            running.set(false);
            status_msg.set("Stopped".into());
        })
    };

    let on_key = {
        let uart_input = uart_input.clone();
        Callback::from(move |e: KeyboardEvent| {
            e.prevent_default();
            let key = e.key();
            let byte = if key.len() == 1 {
                key.as_bytes()[0]
            } else if key == "Enter" {
                b'\n'
            } else if key == "Backspace" {
                0x08
            } else {
                return;
            };
            uart_input.borrow_mut().push_back(byte);
        })
    };

    let on_switch_toggle = {
        let switch_pressed = switch_pressed.clone();
        let emu = emu.clone();
        Callback::from(move |_: MouseEvent| {
            let new_val = !*switch_pressed;
            switch_pressed.set(new_val);
            emu.borrow_mut().set_button_pressed(new_val);
        })
    };

    let on_set_tmp101_temperature = {
        let tmp101_handle = tmp101_handle.clone();
        let tmp101_snapshot = tmp101_snapshot.clone();
        Callback::from(move |celsius: f32| {
            // If a device is attached, drive it; the next tick will
            // re-read and any guest writes (config etc.) come through.
            // If not yet attached (pre-Run), still update the
            // snapshot so the slider position is remembered for when
            // Run attaches and seeds the device.
            if let Some(h) = tmp101_handle.borrow().as_ref() {
                h.set_temperature(celsius);
                tmp101_snapshot.set(Some(read_tmp101_snapshot(h)));
            } else if let Some(mut snap) = *tmp101_snapshot {
                snap.temperature_c = celsius;
                tmp101_snapshot.set(Some(snap));
            }
        })
    };

    let on_set_tmp125_temperature = {
        let tmp125_handle = tmp125_handle.clone();
        let tmp125_snapshot = tmp125_snapshot.clone();
        Callback::from(move |celsius: f32| {
            if let Some(h) = tmp125_handle.borrow().as_ref() {
                h.set_temperature(celsius);
                tmp125_snapshot.set(Some(read_tmp125_snapshot(h)));
            } else if let Some(mut snap) = *tmp125_snapshot {
                snap.temperature_c = celsius;
                tmp125_snapshot.set(Some(snap));
            }
        })
    };

    // SD card upload: read the picked File as an ArrayBuffer via
    // FileReader (only browser-async option for binary blobs), then
    // on success push the bytes into both the live device handle
    // (if a run is in progress) and the next-attach `sdcard_image`
    // ref, and write through to IDB so the upload survives reload.
    let on_upload_sdcard = {
        let sdcard_handle = sdcard_handle.clone();
        let sdcard_snapshot = sdcard_snapshot.clone();
        let sdcard_image = sdcard_image.clone();
        Callback::from(move |file: File| {
            let Ok(reader) = FileReader::new() else { return };
            let reader_clone = reader.clone();
            let sdcard_handle = sdcard_handle.clone();
            let sdcard_snapshot = sdcard_snapshot.clone();
            let sdcard_image = sdcard_image.clone();
            let onload = Closure::wrap(Box::new(move |_e: web_sys::Event| {
                let Ok(result) = reader_clone.result() else {
                    return;
                };
                let Ok(buffer) = result.dyn_into::<js_sys::ArrayBuffer>() else {
                    return;
                };
                let array = Uint8Array::new(&buffer);
                let mut bytes = vec![0u8; array.length() as usize];
                array.copy_to(&mut bytes);
                // Persist to IDB asynchronously; the in-memory state
                // updates immediately so the panel reflects the new
                // image even if the IDB write hasn't landed yet.
                let bytes_for_idb = bytes.clone();
                spawn_local(async move {
                    idb::put(SDCARD_IDB_KEY, &bytes_for_idb).await;
                });
                if let Some(h) = sdcard_handle.borrow().as_ref() {
                    h.replace_image(bytes.clone());
                    sdcard_snapshot.set(Some(read_sdcard_snapshot(h)));
                } else {
                    // No live attach (panel shown pre-Run): update
                    // the next-attach seed and the displayed size.
                    sdcard_snapshot.set(Some(SdCardSnapshot {
                        cs: 2,
                        size_bytes: bytes.len() as u32,
                        last_accessed_sector: None,
                    }));
                }
                *sdcard_image.borrow_mut() = bytes;
            }) as Box<dyn FnMut(web_sys::Event)>);
            reader.set_onload(Some(onload.as_ref().unchecked_ref()));
            onload.forget();
            let _ = reader.read_as_array_buffer(&file);
        })
    };

    // SD card reset: drop the IDB-persisted image, restore the
    // bundled default, and update the live device + panel.
    let on_reset_sdcard = {
        let sdcard_handle = sdcard_handle.clone();
        let sdcard_snapshot = sdcard_snapshot.clone();
        let sdcard_image = sdcard_image.clone();
        Callback::from(move |()| {
            let default_bytes = SDCARD_DEFAULT_IMAGE.to_vec();
            spawn_local(async {
                idb::delete(SDCARD_IDB_KEY).await;
            });
            if let Some(h) = sdcard_handle.borrow().as_ref() {
                h.replace_image(default_bytes.clone());
                sdcard_snapshot.set(Some(read_sdcard_snapshot(h)));
            } else {
                sdcard_snapshot.set(Some(SdCardSnapshot {
                    cs: 2,
                    size_bytes: default_bytes.len() as u32,
                    last_accessed_sector: None,
                }));
            }
            *sdcard_image.borrow_mut() = default_bytes;
        })
    };

    // W25Q32 upload: same shape as the SD card upload. The emulator
    // normalizes any image length to exactly 4 MiB so we don't have
    // to validate file size here.
    let on_upload_w25q32 = {
        let w25q32_handle = w25q32_handle.clone();
        let w25q32_snapshot = w25q32_snapshot.clone();
        let w25q32_image = w25q32_image.clone();
        let w25q32_activity = w25q32_activity.clone();
        let w25q32_erased = w25q32_erased.clone();
        Callback::from(move |file: File| {
            let Ok(reader) = FileReader::new() else { return };
            let reader_clone = reader.clone();
            let w25q32_handle = w25q32_handle.clone();
            let w25q32_snapshot = w25q32_snapshot.clone();
            let w25q32_image = w25q32_image.clone();
            let w25q32_activity = w25q32_activity.clone();
            let w25q32_erased = w25q32_erased.clone();
            let onload = Closure::wrap(Box::new(move |_e: web_sys::Event| {
                let Ok(result) = reader_clone.result() else {
                    return;
                };
                let Ok(buffer) = result.dyn_into::<js_sys::ArrayBuffer>() else {
                    return;
                };
                let array = Uint8Array::new(&buffer);
                let mut bytes = vec![0u8; array.length() as usize];
                array.copy_to(&mut bytes);
                // Normalize to exactly 4 MiB so the panel's flag cache
                // matches what the emulator will see at attach.
                if bytes.len() < 4 * 1024 * 1024 {
                    bytes.resize(4 * 1024 * 1024, 0xFF);
                } else if bytes.len() > 4 * 1024 * 1024 {
                    bytes.truncate(4 * 1024 * 1024);
                }
                let erased = compute_w25q32_erased(&bytes);
                let bytes_for_idb = bytes.clone();
                spawn_local(async move {
                    idb::put(W25Q32_IDB_KEY, &bytes_for_idb).await;
                });
                if let Some(h) = w25q32_handle.borrow().as_ref() {
                    h.replace_image(bytes.clone());
                    w25q32_snapshot.set(Some(read_w25q32_snapshot(
                        h,
                        &w25q32_activity.borrow(),
                        &erased,
                    )));
                }
                *w25q32_image.borrow_mut() = bytes;
                *w25q32_erased.borrow_mut() = erased;
            }) as Box<dyn FnMut(web_sys::Event)>);
            reader.set_onload(Some(onload.as_ref().unchecked_ref()));
            onload.forget();
            let _ = reader.read_as_array_buffer(&file);
        })
    };

    // W25Q32 Chip Erase: call the device's erase_chip (sets all
    // 4 MiB to 0xFF), drop the IDB blob, repaint the heatmap pale
    // (all sectors erased, activity cleared).
    let on_reset_w25q32 = {
        let w25q32_handle = w25q32_handle.clone();
        let w25q32_snapshot = w25q32_snapshot.clone();
        let w25q32_image = w25q32_image.clone();
        let w25q32_activity = w25q32_activity.clone();
        let w25q32_erased = w25q32_erased.clone();
        Callback::from(move |()| {
            spawn_local(async {
                idb::delete(W25Q32_IDB_KEY).await;
            });
            let erased_image = vec![0xFFu8; 4 * 1024 * 1024];
            let erased_flags = vec![true; W25Q32_SECTOR_COUNT];
            let cleared_activity = vec![0u8; W25Q32_SECTOR_COUNT];
            if let Some(h) = w25q32_handle.borrow().as_ref() {
                h.erase_chip();
                w25q32_snapshot.set(Some(read_w25q32_snapshot(
                    h,
                    &cleared_activity,
                    &erased_flags,
                )));
            }
            *w25q32_image.borrow_mut() = erased_image;
            *w25q32_erased.borrow_mut() = erased_flags;
            *w25q32_activity.borrow_mut() = cleared_activity;
        })
    };

    let on_toggle_ds1307_battery = {
        let ds1307_battery_enabled = ds1307_battery_enabled.clone();
        Callback::from(move |enabled: bool| {
            ds1307_battery_enabled.set(enabled);
            battery::save_enabled(enabled);
        })
    };

    let on_set_ds1307_system_time = {
        let ds1307_handle = ds1307_handle.clone();
        let ds1307_snapshot = ds1307_snapshot.clone();
        let ds1307_last_seen = ds1307_last_seen.clone();
        Callback::from(move |_: MouseEvent| {
            // Decompose JS Date into local HH:MM:SS. Using local time
            // rather than UTC because users intuitively expect their
            // wall clock; the chip itself is timezone-agnostic.
            let now = js_sys::Date::new_0();
            let h = now.get_hours() as u8;
            let m = now.get_minutes() as u8;
            let s = now.get_seconds() as u8;

            // Write to the device if a run is live so the running
            // demo immediately sees the new time on its next read.
            if let Some(h_dev) = ds1307_handle.borrow().as_ref() {
                h_dev.set_time(h, m, s);
                // Reset the last-seen tracker to None so the trap
                // fires exactly once for this synthetic write — and
                // the persisted state is updated even if the user
                // hasn't enabled battery yet (a follow-up enable
                // then has something meaningful to boot from).
                *ds1307_last_seen.borrow_mut() = None;
            }

            // Always persist (regardless of battery toggle) so that
            // flipping battery on after a system-time press boots
            // the next run from this value. The brief's snapshot
            // path uses effective_now() at attach, so this writes
            // the new set_value with `set_at_ms = Date.now()`.
            battery::save(battery::Persisted {
                set_value: battery::SetValue { h, m, s },
                set_at_ms: now.get_time(),
            });

            // Reflect immediately in the panel.
            ds1307_snapshot.set(Some(Ds1307Snapshot {
                address: 0x68,
                hour: h,
                minute: m,
                second: s,
            }));
        })
    };

    let on_demo_select = {
        let source = source.clone();
        let assemble_error = assemble_error.clone();
        let listing = listing.clone();
        let interval_handle = interval_handle.clone();
        let running = running.clone();
        let status_msg = status_msg.clone();
        let demo_config = demo_config.clone();
        let tmp101_snapshot = tmp101_snapshot.clone();
        let test_device_snapshot = test_device_snapshot.clone();
        let tmp125_snapshot = tmp125_snapshot.clone();
        let echo_snapshot = echo_snapshot.clone();
        let sdcard_snapshot = sdcard_snapshot.clone();
        let sdcard_image = sdcard_image.clone();
        let w25q32_snapshot = w25q32_snapshot.clone();
        let w25q32_erased = w25q32_erased.clone();
        let ds1307_snapshot = ds1307_snapshot.clone();
        let ds1307_battery_enabled = ds1307_battery_enabled.clone();
        let ssd1306_snapshot = ssd1306_snapshot.clone();
        Callback::from(move |e: Event| {
            let Some(select) = e
                .target()
                .and_then(|t| t.dyn_into::<HtmlSelectElement>().ok())
            else {
                return;
            };
            let value = select.value();
            if value.is_empty() {
                return;
            }
            select.set_value("");

            // Stop any running emulator.
            *interval_handle.borrow_mut() = None;
            running.set(false);

            if let Some(demo) = demos::lookup(&value) {
                source.set(demo.source.to_string());
                demo_config.set(demo.config);
                // Show the demo's device panels immediately, before
                // the user hits Run. Snapshots are synthesized from
                // device defaults; the slider value (if any) is
                // preserved across selections so users can drag it
                // ahead of time. Buses the demo doesn't touch clear
                // their snapshot, auto-hiding the matching panel.
                if demo.config.attach_tmp101 {
                    tmp101_snapshot.set(Some(default_or_keep_tmp101(&tmp101_snapshot)));
                } else {
                    tmp101_snapshot.set(None);
                }
                if demo.config.attach_test_i2c {
                    test_device_snapshot.set(Some(default_or_keep_test_device(
                        &test_device_snapshot,
                    )));
                } else {
                    test_device_snapshot.set(None);
                }
                if demo.config.attach_tmp125 {
                    tmp125_snapshot.set(Some(default_or_keep_tmp125(&tmp125_snapshot)));
                } else {
                    tmp125_snapshot.set(None);
                }
                if demo.config.attach_test_spi {
                    echo_snapshot.set(Some(default_or_keep_echo(&echo_snapshot)));
                } else {
                    echo_snapshot.set(None);
                }
                if demo.config.attach_rtc {
                    ds1307_snapshot.set(Some(default_or_keep_ds1307(
                        &ds1307_snapshot,
                        *ds1307_battery_enabled,
                    )));
                } else {
                    ds1307_snapshot.set(None);
                }
                if demo.config.attach_ssd1306 {
                    ssd1306_snapshot.set(Some(default_or_keep_ssd1306(&ssd1306_snapshot)));
                } else {
                    ssd1306_snapshot.set(None);
                }
                if demo.config.attach_sdcard {
                    // Preview the size/CS from the current image
                    // bytes (IDB-restored upload or bundled default).
                    let size = sdcard_image.borrow().len() as u32;
                    sdcard_snapshot.set(Some(SdCardSnapshot {
                        cs: 2,
                        size_bytes: size,
                        last_accessed_sector: None,
                    }));
                } else {
                    sdcard_snapshot.set(None);
                }
                if demo.config.attach_w25q32 {
                    // Preview the heatmap from the current erased-
                    // flag cache; activity starts cold pre-Run.
                    let activity = vec![0u8; W25Q32_SECTOR_COUNT];
                    let erased = w25q32_erased.borrow().clone();
                    w25q32_snapshot.set(Some(W25q32Snapshot {
                        cs: 3,
                        jedec_id: [0xEF, 0x40, 0x16],
                        wip: false,
                        wel: false,
                        last_accessed_address: None,
                        sector_activity: activity,
                        sector_erased: erased,
                    }));
                } else {
                    w25q32_snapshot.set(None);
                }
                assemble_error.set(None);
                listing.set(Vec::new());
                status_msg.set(format!("Loaded: {value}"));
            }
        })
    };

    // --- Global Cmd/Ctrl+Enter shortcut: fire Assemble & Run from
    // anywhere on the page (editor, panels, even with no focus). The
    // listener registers once on mount and forgets the closure so it
    // lives for the page lifetime; cleanup would require tracking
    // the JS function pointer, which is overkill for a single-shot
    // listener that never gets re-attached.
    {
        let on_run = on_run.clone();
        use_effect_with((), move |_| {
            use wasm_bindgen::closure::Closure;
            let cb = Closure::wrap(Box::new(move |e: KeyboardEvent| {
                if e.key() == "Enter" && (e.meta_key() || e.ctrl_key()) {
                    e.prevent_default();
                    on_run.emit(());
                }
            }) as Box<dyn FnMut(KeyboardEvent)>);
            if let Some(window) = web_sys::window() {
                let _ = window.add_event_listener_with_callback(
                    "keydown",
                    cb.as_ref().unchecked_ref(),
                );
            }
            cb.forget();
            || ()
        });
    }

    // --- Error lines for highlighting ---
    let asm_error_line = assemble_error
        .as_ref()
        .and_then(|e| e.line)
        .or(*runtime_error_line);

    // --- Render ---
    html! {
        <main style="display:flex; flex-direction:column; height:100vh; padding:16px; gap:12px;">
            // GitHub corner.
            <a href="https://github.com/sw-embed/web-sw-cor24-x-assembler"
               aria-label="View source on GitHub"
               target="_blank"
               style="position:absolute; top:0; right:0; z-index:100;">
                <svg width="80" height="80" viewBox="0 0 250 250"
                     style="fill:#89b4fa; color:#1e1e2e;" aria-hidden="true">
                    <path d="M0,0 L115,115 L130,115 L142,142 L250,250 L250,0 Z" />
                    <path d="M128.3,109.0 C113.8,99.7 119.0,89.6 119.0,89.6 \
                        C122.0,82.7 120.5,78.6 120.5,78.6 C119.2,72.0 123.4,76.3 123.4,76.3 \
                        C127.3,80.9 125.5,87.3 125.5,87.3 C122.9,97.6 130.6,101.9 134.4,103.2"
                        fill="currentColor" style="transform-origin:130px 106px;" />
                    <path d="M115.0,115.0 C114.9,115.1 118.7,116.5 119.8,115.4 L133.7,101.6 \
                        C136.9,99.2 139.9,98.4 142.2,98.6 C133.8,88.0 127.5,74.4 143.8,58.0 \
                        C148.5,53.4 154.0,51.2 159.7,51.0 C160.3,49.4 163.2,43.6 171.4,40.1 \
                        C171.4,40.1 176.1,42.5 178.8,56.2 C183.1,58.6 187.2,61.8 190.9,65.4 \
                        C194.5,69.0 197.7,73.2 200.1,77.6 C213.8,80.2 216.3,84.9 216.3,84.9 \
                        C212.7,93.1 206.9,96.0 205.4,96.6 C205.1,102.4 203.0,107.8 198.3,112.5 \
                        C181.9,128.9 168.3,122.5 157.7,114.1 C157.9,116.9 156.7,120.9 152.7,124.9 \
                        L141.0,136.5 C139.8,137.7 141.6,141.9 141.8,141.8 Z"
                        fill="currentColor" />
                </svg>
            </a>

            <h1 style="font-size:1.4rem; color:#89b4fa;">
                {"web-sw-cor24-x-assembler"}
                <span style="font-size:0.8rem; color:#bac2de; margin-left:8px;">
                    {"COR24 cross-assembler in your browser"}
                </span>
            </h1>

            <div style="display:flex; flex:1; gap:12px; min-height:0;">
                // Assembly source editor.
                <div style="flex:1; min-width:0; display:flex; flex-direction:column; gap:8px;">
                    <label style="font-size:0.9rem; color:#cdd6f4; font-weight:600;">
                        {"Assembly Source"}
                    </label>
                    <Editor value={AttrValue::from((*source).clone())}
                            on_change={on_source_change}
                            error_line={asm_error_line} />
                </div>

                // Listing.
                <div style="flex:1; min-width:0; display:flex; flex-direction:column; gap:8px;">
                    <label style="font-size:0.9rem; color:#cdd6f4; font-weight:600;">
                        {"Listing"}
                    </label>
                    { panels::listing::render(&listing, asm_error_line) }
                </div>

                // Emulator panel.
                <div style="flex:1; min-width:0; display:flex; flex-direction:column; gap:8px;">
                    <label style="font-size:0.9rem; color:#cdd6f4; font-weight:600;">
                        {"Emulator"}
                    </label>
                    <div style="flex:1; display:flex; flex-direction:column; gap:8px; \
                                background:#181825; border:1px solid #313244; border-radius:6px; \
                                padding:12px; overflow:auto;">

                        if let Some(err) = assemble_error.as_ref() {
                            <div style="margin-bottom:8px;">
                                <div style="color:#f38ba8; font-weight:600; font-size:0.8rem; \
                                            margin-bottom:2px;">
                                    {"Assembly error"}
                                    if let Some(line) = err.line {
                                        {format!(" line {line}")}
                                    }
                                </div>
                                <pre style="color:#f38ba8; margin:0; white-space:pre-wrap; \
                                            font-size:0.8rem;">
                                    {&err.message}
                                </pre>
                            </div>
                        }

                        <UartPanel
                            output={AttrValue::from((*uart_output).clone())}
                            running={*running}
                            halted={*halted}
                            on_key={on_key}
                        />

                        <RegistersPanel regs={*registers} pc={*pc_val} cond={*cond_flag} />

                        <div style="display:flex; gap:16px; align-items:center;">
                            <LedPanel state={*led_state} />
                            <SwitchPanel pressed={*switch_pressed} on_toggle={on_switch_toggle} />
                        </div>

                        <I2cPanel bus={*bus_snapshot}
                                  tmp101={*tmp101_snapshot}
                                  test_device={*test_device_snapshot}
                                  ds1307={*ds1307_snapshot}
                                  ds1307_battery_enabled={*ds1307_battery_enabled}
                                  ssd1306={(*ssd1306_snapshot).clone()}
                                  on_set_tmp101_temperature={on_set_tmp101_temperature}
                                  on_toggle_ds1307_battery={on_toggle_ds1307_battery}
                                  on_set_ds1307_system_time={on_set_ds1307_system_time} />

                        <SpiPanel bus={*spi_bus_snapshot}
                                  tmp125={*tmp125_snapshot}
                                  echo={*echo_snapshot}
                                  sdcard={(*sdcard_snapshot).clone()}
                                  w25q32={(*w25q32_snapshot).clone()}
                                  on_set_tmp125_temperature={on_set_tmp125_temperature}
                                  on_upload_sdcard={on_upload_sdcard}
                                  on_reset_sdcard={on_reset_sdcard}
                                  on_upload_w25q32={on_upload_w25q32}
                                  on_reset_w25q32={on_reset_w25q32} />

                        <div style="display:flex; justify-content:space-between; align-items:center; \
                                    font-size:0.8rem; color:#bac2de; border-top:1px solid #313244; \
                                    padding-top:6px;">
                            <span>{&*status_msg}</span>
                            <span>{format!("{} instructions", *instr_count)}</span>
                        </div>
                    </div>
                </div>
            </div>

            // Button bar.
            <div style="display:flex; gap:12px; align-items:center;">
                <button onclick={
                        let on_run = on_run.clone();
                        Callback::from(move |_: MouseEvent| on_run.emit(()))
                    }
                    title="Assemble & Run (Cmd/Ctrl+Enter)"
                    style="padding:8px 24px; background:#89b4fa; color:#1e1e2e; \
                           border:none; border-radius:6px; font-size:1rem; font-weight:600; \
                           cursor:pointer;">
                    {"Assemble & Run"}
                </button>

                if *running {
                    <button onclick={on_stop}
                        style="padding:8px 24px; background:#f38ba8; color:#1e1e2e; \
                               border:none; border-radius:6px; font-size:1rem; font-weight:600; \
                               cursor:pointer;">
                        {"Stop"}
                    </button>
                }

                <select onchange={on_demo_select}
                    style="padding:6px 12px; background:#313244; color:#cdd6f4; \
                           border:1px solid #585b70; border-radius:6px; font-size:0.85rem; \
                           cursor:pointer;">
                    <option value="" selected=true disabled=true>{"Load demo..."}</option>
                    { for demos::EXAMPLES.iter().map(|d| html! {
                        <option value={d.name}>{d.name}</option>
                    }) }
                </select>
            </div>

            // Footer.
            <div style="display:flex; gap:8px; align-items:center; flex-wrap:wrap; \
                        font-size:0.9rem; color:#bac2de; padding-top:4px;">
                <span>{"\u{00a9} 2026 Michael A. Wright"}</span>
                <span>{"\u{00b7}"}</span>
                <span>{"MIT License"}</span>
                <span>{"\u{00b7}"}</span>
                <a href="https://makerlisp.com" target="_blank"
                   style="color:#89b4fa; text-decoration:none;">{"COR24-TB"}</a>
                <span>{"\u{00b7}"}</span>
                <a href="https://software-wrighter-lab.github.io/" target="_blank"
                   style="color:#89b4fa; text-decoration:none;">{"Blog"}</a>
                <span>{"\u{00b7}"}</span>
                <a href="https://discord.com/invite/Ctzk5uHggZ" target="_blank"
                   style="color:#89b4fa; text-decoration:none;">{"Discord"}</a>
                <span>{"\u{00b7}"}</span>
                <a href="https://www.youtube.com/@SoftwareWrighter" target="_blank"
                   style="color:#89b4fa; text-decoration:none;">{"YouTube"}</a>
                <span>{"\u{00b7}"}</span>
                <span>{ format!("{} \u{00b7} {} \u{00b7} {}",
                    env!("BUILD_HOST"),
                    env!("BUILD_SHA"),
                    env!("BUILD_TIMESTAMP"),
                ) }</span>
            </div>
        </main>
    }
}

/// Snapshot the TMP101's UI-visible state through its typed handle.
/// Called from both the initial attach (to seed the panel) and from
/// the run-loop tick (to refresh it).
fn read_tmp101_snapshot(handle: &I2cHandle<Tmp101Device>) -> Tmp101Snapshot {
    let temperature_c = handle.temperature();
    let resolution = handle.resolution();
    handle.with(|d| Tmp101Snapshot {
        address: d.address(),
        temperature_c,
        config: d.config(),
        resolution,
    })
}

// Default-or-keep synthesizers: when a demo is selected, give the
// matching panel something to show immediately. If the user has
// already dragged a slider on a prior selection, preserve that value
// (don't surprise them by resetting to 0).
fn default_or_keep_tmp101(state: &UseStateHandle<Option<Tmp101Snapshot>>) -> Tmp101Snapshot {
    (**state).unwrap_or(Tmp101Snapshot {
        address: cor24_emulator::peripherals::i2c::devices::tmp101::DEFAULT_ADDRESS,
        temperature_c: 0.0,
        config: 0,
        resolution: Tmp101Resolution::Bits9,
    })
}

fn default_or_keep_test_device(
    state: &UseStateHandle<Option<TestDeviceSnapshot>>,
) -> TestDeviceSnapshot {
    (**state).unwrap_or(TestDeviceSnapshot {
        address: 0x50,
        last_byte: 0,
    })
}

fn default_or_keep_tmp125(state: &UseStateHandle<Option<Tmp125Snapshot>>) -> Tmp125Snapshot {
    (**state).unwrap_or(Tmp125Snapshot {
        temperature_c: 0.0,
    })
}

fn default_or_keep_echo(state: &UseStateHandle<Option<EchoSnapshot>>) -> EchoSnapshot {
    (**state).unwrap_or(EchoSnapshot { buffer: 0 })
}

/// Synthesize an empty SSD1306 snapshot (display off, all pixels
/// cleared) pre-Run so the panel shows the dark module right away
/// when the user picks an OLED demo.
fn default_or_keep_ssd1306(state: &UseStateHandle<Option<Ssd1306Snapshot>>) -> Ssd1306Snapshot {
    if let Some(snap) = state.as_ref() {
        return snap.clone();
    }
    Ssd1306Snapshot {
        address: cor24_emulator::peripherals::i2c::devices::ssd1306::DEFAULT_ADDRESS,
        width: 128,
        height: 64,
        display_on: false,
        framebuffer: vec![0u8; panels::SSD1306_FRAMEBUFFER_LEN],
    }
}

/// Synthesize a DS1307 snapshot pre-Run. If the battery toggle is on
/// and there's persisted state, the synthesized time is
/// `(set_value + elapsed) % 86400` so the user sees the time the
/// device will boot at *before* hitting Run. Otherwise 00:00:00.
fn default_or_keep_ds1307(
    state: &UseStateHandle<Option<Ds1307Snapshot>>,
    battery_enabled: bool,
) -> Ds1307Snapshot {
    if let Some(snap) = **state {
        return snap;
    }
    if battery_enabled
        && let Some(p) = battery::load()
    {
        let eff = battery::effective_now(p, js_sys::Date::now());
        return Ds1307Snapshot {
            address: 0x68,
            hour: eff.h,
            minute: eff.m,
            second: eff.s,
        };
    }
    Ds1307Snapshot {
        address: 0x68,
        hour: 0,
        minute: 0,
        second: 0,
    }
}

/// Integer (0..=99) to BCD byte. Used by the attach path when the
/// battery toggle seeds the DS1307 from localStorage.
fn int_to_bcd(n: u8) -> u8 {
    ((n / 10) << 4) | (n % 10)
}

/// Snapshot the I2C test slave's UI-visible state.
fn read_test_device_snapshot(handle: &I2cHandle<Add1Device>) -> TestDeviceSnapshot {
    handle.with(|d| TestDeviceSnapshot {
        address: d.address(),
        last_byte: d.peek(),
    })
}

/// Snapshot the DS1307 RTC's UI-visible state (binary HH:MM:SS).
fn read_ds1307_snapshot(handle: &I2cHandle<Ds1307Device>) -> Ds1307Snapshot {
    Ds1307Snapshot {
        address: handle.address(),
        hour: handle.hour(),
        minute: handle.minute(),
        second: handle.second(),
    }
}

/// Snapshot the SSD1306 OLED's framebuffer + display-on bit.
fn read_ssd1306_snapshot(handle: &I2cHandle<Ssd1306Device>) -> Ssd1306Snapshot {
    handle.with(|d| Ssd1306Snapshot {
        address: d.address(),
        width: d.width(),
        height: d.height(),
        display_on: d.display_on(),
        framebuffer: d.framebuffer().to_vec(),
    })
}

/// Snapshot the TMP125's UI-visible state through its SPI handle.
fn read_tmp125_snapshot(handle: &SpiHandle<Tmp125Device>) -> Tmp125Snapshot {
    Tmp125Snapshot {
        temperature_c: handle.temperature(),
    }
}

/// Snapshot the SPI EchoDevice's UI-visible state.
fn read_echo_snapshot(handle: &SpiHandle<EchoDevice>) -> EchoSnapshot {
    handle.with(|d| EchoSnapshot { buffer: d.peek() })
}

/// Snapshot the SPI SD card's UI-visible state. The image bytes are
/// pulled on demand from the panel's upload/reset callbacks rather
/// than carried in the snapshot -- a 1 MiB+ blob in every tick's
/// snapshot would defeat Yew's PartialEq short-circuit.
fn read_sdcard_snapshot(handle: &SpiHandle<SdCardDevice>) -> SdCardSnapshot {
    SdCardSnapshot {
        cs: handle.with(|d| d.cs()),
        size_bytes: handle.size() as u32,
        last_accessed_sector: handle.last_accessed_sector(),
    }
}

/// Snapshot the W25Q32's UI-visible state. The 4 MiB image is not
/// in the snapshot; instead the per-sector "activity" cooling counter
/// and the per-sector "is erased (0xFF)" flag are -- both small
/// (1024 bytes + 1024 bools) and PartialEq-friendly.
fn read_w25q32_snapshot(
    handle: &SpiHandle<W25q32Device>,
    activity: &[u8],
    erased: &[bool],
) -> W25q32Snapshot {
    W25q32Snapshot {
        cs: handle.with(|d| d.cs()),
        jedec_id: handle.jedec_id(),
        wip: handle.wip(),
        wel: handle.wel(),
        last_accessed_address: handle.last_accessed_address(),
        sector_activity: activity.to_vec(),
        sector_erased: erased.to_vec(),
    }
}

/// Recompute the per-sector "all 0xFF" flag cache from a fresh
/// image. O(4 MiB) -- only called on uploads / reset / write-trap,
/// not per-tick.
fn compute_w25q32_erased(image: &[u8]) -> Vec<bool> {
    let mut flags = Vec::with_capacity(W25Q32_SECTOR_COUNT);
    for sector in 0..W25Q32_SECTOR_COUNT {
        let start = sector * W25Q32_SECTOR_SIZE;
        let end = (start + W25Q32_SECTOR_SIZE).min(image.len());
        let all_ff = image[start..end].iter().all(|&b| b == 0xFF);
        flags.push(all_ff);
    }
    flags
}

fn main() {
    yew::Renderer::<App>::new().render();
}
