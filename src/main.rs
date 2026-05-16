//! web-sw-cor24-x-assembler — Yew app entry point.
//!
//! Layout (mirrors web-sw-cor24-x-tinyc): three columns — source editor,
//! assembled listing, emulator I/O panel. The "Assemble & Run" button
//! drives the cor24-assembler over the source, loads the bytes into a
//! cor24-emulator EmulatorCore, and ticks an Interval to advance the
//! run loop while keeping the UI responsive.

mod assembler;
mod demos;
mod editor;
mod highlight;
mod panels;

use std::cell::RefCell;
use std::rc::Rc;

use cor24_assembler::AssembledLine;
use cor24_emulator::EmulatorCore;
use cor24_emulator::peripherals::i2c::{
    Add1Device, I2cDevice, I2cHandle, Tmp101Device, Tmp101HandleExt, Tmp101Resolution,
};
use cor24_emulator::peripherals::spi::{EchoDevice, SpiHandle, Tmp125Device, Tmp125HandleExt};
use wasm_bindgen::JsCast;
use web_sys::{HtmlSelectElement, KeyboardEvent};
use yew::prelude::*;

use editor::Editor;
use panels::{
    BusSnapshot, EchoSnapshot, I2cPanel, LedPanel, RegistersPanel, SpiBusSnapshot, SpiPanel,
    SwitchPanel, TestDeviceSnapshot, Tmp101Snapshot, Tmp125Snapshot, UartPanel,
};

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
    let bus_snapshot = use_state(BusSnapshot::default);
    let tmp101_snapshot = use_state(|| None::<Tmp101Snapshot>);
    let test_device_snapshot = use_state(|| None::<TestDeviceSnapshot>);

    // SPI side: single-slave; either TMP125 or EchoDevice is
    // attached per the demo's config.
    let tmp125_handle: Rc<RefCell<Option<SpiHandle<Tmp125Device>>>> = use_mut_ref(|| None);
    let echo_handle: Rc<RefCell<Option<SpiHandle<EchoDevice>>>> = use_mut_ref(|| None);
    let spi_bus_snapshot = use_state(SpiBusSnapshot::default);
    let tmp125_snapshot = use_state(|| None::<Tmp125Snapshot>);
    let echo_snapshot = use_state(|| None::<EchoSnapshot>);

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
        let bus_snapshot = bus_snapshot.clone();
        let tmp101_snapshot = tmp101_snapshot.clone();
        let test_device_snapshot = test_device_snapshot.clone();
        let tmp125_handle = tmp125_handle.clone();
        let echo_handle = echo_handle.clone();
        let spi_bus_snapshot = spi_bus_snapshot.clone();
        let tmp125_snapshot = tmp125_snapshot.clone();
        let echo_snapshot = echo_snapshot.clone();
        let demo_config = demo_config.clone();

        Callback::from(move |_: MouseEvent| {
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
            let (h_tmp101, h_test, h_tmp125, h_echo) = {
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

                e.resume();
                (h_tmp101, h_test, h_tmp125, h_echo)
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
            let bus_snapshot = bus_snapshot.clone();
            let tmp101_snapshot = tmp101_snapshot.clone();
            let test_device_snapshot = test_device_snapshot.clone();
            let tmp125_handle = tmp125_handle.clone();
            let echo_handle = echo_handle.clone();
            let spi_bus_snapshot = spi_bus_snapshot.clone();
            let tmp125_snapshot = tmp125_snapshot.clone();
            let echo_snapshot = echo_snapshot.clone();

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

                // Instructions per 16 ms tick. 100k is the sweet spot for
                // the current demo mix: tight read-print loops (no idle
                // delay) stay snappy and the UI thread keeps up with
                // slider events. The earlier 1M tuning was needed for
                // the now-removed tmp101.lgo (which spun a 16M-iteration
                // delay between reads); with all bundled demos being
                // hand-tuned .s, that budget pegged CPU and made the
                // sliders unresponsive.
                let batch = e.run_batch(100_000);

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

    let on_poke_test_device = {
        let test_device_handle = test_device_handle.clone();
        let test_device_snapshot = test_device_snapshot.clone();
        Callback::from(move |byte: u8| {
            if let Some(h) = test_device_handle.borrow().as_ref() {
                h.with(|d| d.poke(byte));
                test_device_snapshot.set(Some(read_test_device_snapshot(h)));
            } else if let Some(mut snap) = *test_device_snapshot {
                snap.last_byte = byte;
                test_device_snapshot.set(Some(snap));
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

    let on_poke_echo = {
        let echo_handle = echo_handle.clone();
        let echo_snapshot = echo_snapshot.clone();
        Callback::from(move |byte: u8| {
            if let Some(h) = echo_handle.borrow().as_ref() {
                h.with(|d| d.poke(byte));
                echo_snapshot.set(Some(read_echo_snapshot(h)));
            } else if let Some(mut snap) = *echo_snapshot {
                snap.buffer = byte;
                echo_snapshot.set(Some(snap));
            }
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
                assemble_error.set(None);
                listing.set(Vec::new());
                status_msg.set(format!("Loaded: {value}"));
            }
        })
    };

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
                                  on_set_tmp101_temperature={on_set_tmp101_temperature}
                                  on_poke_test_device={on_poke_test_device} />

                        <SpiPanel bus={*spi_bus_snapshot}
                                  tmp125={*tmp125_snapshot}
                                  echo={*echo_snapshot}
                                  on_set_tmp125_temperature={on_set_tmp125_temperature}
                                  on_poke_echo={on_poke_echo} />

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
                <button onclick={on_run}
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

/// Snapshot the I2C test slave's UI-visible state.
fn read_test_device_snapshot(handle: &I2cHandle<Add1Device>) -> TestDeviceSnapshot {
    handle.with(|d| TestDeviceSnapshot {
        address: d.address(),
        last_byte: d.peek(),
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

fn main() {
    yew::Renderer::<App>::new().render();
}
