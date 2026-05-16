Implement an I2C RTC device + demo following the recommendations in
sw-cor24-x-assembler/docs/i2c-rt2-research.txt. The doc ranks DS1307
as the simplest popular I2C RTC and gives the register map +
read/write transactions to copy. Two-part work, only the second part
lives in this repo:

Part A (cross-repo, briefed to dcemu): add `Ds1307Device` to
sw-cor24-emulator under src/peripherals/i2c/devices/ds1307.rs.
Constructor takes (address: u8 = 0x68, seconds: u8, minutes: u8,
hours: u8, day_of_week: u8, date: u8, month: u8, year: u8) all in
BCD. Auto-incrementing pointer (0x00..=0x06). I2cDevice trait impl:
on_write_byte handles pointer-then-data; on_read_byte returns the
register at pointer and auto-advances; pointer wraps after 0x06.
Mask out CH bit on write to 0x00 to keep the clock running. Add a
'tick' helper that advances seconds and cascades into minutes/etc.
Brief: tools/briefs/dcemu-i2c-ds1307-device.md (drafted in this
session; pull the latest before consuming).

Part B (this repo, once dcemu's pr/ lands):
1. src/panels/i2c/rtc.rs: Ds1307Snapshot { address, hh:mm:ss, dow,
   date/month/year } and a Ds1307Panel showing the current time +
   optional 'set to system time' button (or a small editable input
   per field). Drop the panel into I2cPanel container.
2. DemoConfig::attach_rtc = true; default DemoConfig::RTC_ONLY const.
3. src/examples/i2c_rtc_read.s: read all 7 registers, print
   HH:MM:SS \n. Tight loop with the slider's bcd value driving
   what shows up. The demo serves as both functional test and a
   readable example of the auto-incrementing-pointer I2C read
   pattern.
4. demos.rs entry 'I2C RTC Read' at the alphabetical slot.
5. Snapshot polling each tick reads all 7 registers via
   handle.with; default-or-keep synth uses system time on first
   load.
6. pages/ rebuilt; clippy clean.

Reference reading before starting Part B: the docs/i2c-rt2-research.txt
file in sibling sw-cor24-x-assembler is the spec; cor24-emulator's
existing TMP101 device is the closest implementation template
(pointer + data register array; BCD instead of binary).
