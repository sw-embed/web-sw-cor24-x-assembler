Saga's "more I2C devices" arc is in good shape -- TMP101, test
device, DS1307 (with battery persistence) all shipping. Pick from
the future-step seeds:

1. Slider/input to set the RTC time directly from the panel (rather
   than typing 6 digits into UART). Probably three numeric inputs
   for HH/MM/SS that emit the same set_value/set_at_ms payload as
   the i2c_ds1307_set demo's UART path.
2. PCF8563 / DS3231 RTC as new emulator devices (cross-repo brief
   to dcemu) once DS1307 has settled.
3. I2C LCD (HD44780-ish) panel + demo: a 16x2 character display
   showing whatever the demo writes to it.
4. Adaptive run-loop budget (still on the queue) -- replace the
   100k/tick static budget with a wall-clock budget.
5. UI polish: keyboard shortcut Cmd/Ctrl+Enter to run, light/dark
   toggle, mobile layout pass.
