; I2C SSD1306 Hello: bit-bang i2c demo against the emulator's SSD1306 OLED.
;
; Web-local variant of the upstream sw-cor24-x-assembler/.../i2c_ssd1306_hello.s.
; Forked here to add inter-glyph spacer columns (29 data bytes instead
; of 25) so the rendered "HELLO" reads with visible letter separation on
; the simulated 0.96" two-color OLED panel. The upstream version packs
; glyphs contiguously, which is denser and matches the framebuffer's
; native 1:1 column layout but reads as crowded text in the web demo.
;
; Initializes the display, positions the GDDRAM pointer at page 0 column 0,
; writes 29 bytes of 5×8 font data spelling "HELLO" (with 1-column gaps)
; to the data stream, and halts. The 8 vertical pixels per column are
; encoded LSB-at-top in one byte, matching the SSD1306's GDDRAM wire
; format exactly — the font bytes flow directly through with no
; transformation.
;
; Pair with `cor24-emu --i2c-device ssd1306@0x3C`:
;     cor24-asm src/examples/assembler/i2c_ssd1306_hello.s -o /tmp/hello.lgo
;     cor24-emu --lgo /tmp/hello.lgo --i2c-device ssd1306@0x3C --quiet
;     # use --dump-i2c to see the protocol traffic; the framebuffer itself
;     # is observable via the web RTC panel (when dwxas's saga lands) or via
;     # the emulator's tests/i2c.rs unit tests.
;
; Without an i2c device attached, the init writes go silently nowhere and
; the demo halts cleanly — that's the no-display fallback.
;
; MMIO:
;     0xFF0020 = SCL  (bit 0 significant; open-drain: 1 = release, 0 = pull)
;     0xFF0021 = SDA  (same)
;
; SSD1306 protocol shape (per
; sw-cor24-emulator/src/peripherals/i2c/devices/ssd1306.rs):
;     - 7-bit addr 0x3C → master write byte 0x78. (0x3D is an alternate
;       strap; this demo targets the default 0x3C.)
;     - Control byte after addr: 0x00 = subsequent bytes are commands;
;       0x40 = subsequent bytes are GDDRAM data. Stays in that mode until
;       STOP. (Co=1 variants exist but standard drivers don't use them.)
;     - The emulator lenient-consumes unmodeled init opcodes (contrast,
;       clock divide, multiplex, COM pins, charge pump), so a real-driver
;       25-byte init sequence isn't necessary — the 8-byte minimum init
;       below is sufficient.
;
; Init sequence used:
;     0xAE                  ; display off (clean start)
;     0x20 0x00             ; horizontal addressing — auto-increment col,
;                           ; wrap to next page at col 127
;     0x21 0   127          ; col range 0..127
;     0x22 0   7            ; page range 0..7
;     0xB0                  ; page pointer = 0
;     0x00                  ; col-low nibble = 0
;     0x10                  ; col-high nibble = 0
;     0xAF                  ; display on
;
; Bit-bang primitives (i2cstart / i2cstop / i2cwrite) copied verbatim
; from i2c_ds1307_set.s — no .include in this assembler.

        ; --- main ---
        la      r0, 0FEEC00h
        mov     sp, r0

        ; ===== init burst =====
        la      r1, .ret_init_st
        la      r2, i2cstart
        jal     r1, (r2)
.ret_init_st:
        lcu     r0, 78h         ; addr 0x3C << 1, W
        la      r1, .ret_init_a
        la      r2, i2cwrite
        jal     r1, (r2)
.ret_init_a:
        lc      r0, 0           ; control: commands
        la      r1, .ret_init_c
        la      r2, i2cwrite
        jal     r1, (r2)
.ret_init_c:
        lcu     r0, 0AEh        ; display off
        la      r1, .ret_i1
        la      r2, i2cwrite
        jal     r1, (r2)
.ret_i1:
        lcu     r0, 20h         ; addressing mode setter
        la      r1, .ret_i2
        la      r2, i2cwrite
        jal     r1, (r2)
.ret_i2:
        lc      r0, 0           ; horizontal
        la      r1, .ret_i3
        la      r2, i2cwrite
        jal     r1, (r2)
.ret_i3:
        lcu     r0, 21h         ; col range setter
        la      r1, .ret_i4
        la      r2, i2cwrite
        jal     r1, (r2)
.ret_i4:
        lc      r0, 0           ; col start
        la      r1, .ret_i5
        la      r2, i2cwrite
        jal     r1, (r2)
.ret_i5:
        lcu     r0, 7Fh         ; col end = 127
        la      r1, .ret_i6
        la      r2, i2cwrite
        jal     r1, (r2)
.ret_i6:
        lcu     r0, 22h         ; page range setter
        la      r1, .ret_i7
        la      r2, i2cwrite
        jal     r1, (r2)
.ret_i7:
        lc      r0, 0           ; page start
        la      r1, .ret_i8
        la      r2, i2cwrite
        jal     r1, (r2)
.ret_i8:
        lc      r0, 7           ; page end
        la      r1, .ret_i9
        la      r2, i2cwrite
        jal     r1, (r2)
.ret_i9:
        lcu     r0, 0B0h        ; page = 0
        la      r1, .ret_i10
        la      r2, i2cwrite
        jal     r1, (r2)
.ret_i10:
        lc      r0, 0           ; col low = 0
        la      r1, .ret_i11
        la      r2, i2cwrite
        jal     r1, (r2)
.ret_i11:
        lcu     r0, 10h         ; col high = 0
        la      r1, .ret_i12
        la      r2, i2cwrite
        jal     r1, (r2)
.ret_i12:
        lcu     r0, 0AFh        ; display on
        la      r1, .ret_i13
        la      r2, i2cwrite
        jal     r1, (r2)
.ret_i13:
        la      r1, .ret_init_sp
        la      r2, i2cstop
        jal     r1, (r2)
.ret_init_sp:

        ; ===== data burst: write "HELLO" pixels =====
        la      r1, .ret_data_st
        la      r2, i2cstart
        jal     r1, (r2)
.ret_data_st:
        lcu     r0, 78h         ; addr + W
        la      r1, .ret_data_a
        la      r2, i2cwrite
        jal     r1, (r2)
.ret_data_a:
        lcu     r0, 40h         ; control: data
        la      r1, .ret_data_c
        la      r2, i2cwrite
        jal     r1, (r2)
.ret_data_c:

        ; Loop: write 25 bytes from hello_data .. hello_end.
        ; Use fp-frame to hold the byte pointer (24-bit address); r2 is the
        ; remaining-byte counter.
        la      r0, hello_data
        push    fp
        push    r0              ; ptr at 0(fp)
        mov     fp, sp
        lcu     r2, 29          ; 5×5-col glyphs + 4 inter-glyph spacer cols
.write_loop:
        lw      r1, 0(fp)       ; current ptr (24-bit)
        lbu     r0, 0(r1)       ; byte at ptr
        push    r2              ; save counter across i2cwrite
        la      r1, .ret_wb
        la      r2, i2cwrite
        jal     r1, (r2)
.ret_wb:
        pop     r2
        lw      r0, 0(fp)       ; advance ptr by 1
        add     r0, 1
        sw      r0, 0(fp)
        add     r2, -1
        ceq     r2, z
        brf     .write_loop

        ; teardown
        mov     sp, fp
        add     sp, 3           ; discard ptr local
        pop     fp

        la      r1, .ret_data_sp
        la      r2, i2cstop
        jal     r1, (r2)
.ret_data_sp:

halt:
        bra     halt

; ============================================================================
; Font data: 5×8 glyphs for H, E, L, O (LSB-at-top column encoding).
; "HELLO" = H, E, L, L, O. Each non-leading glyph carries a leading
; 0x00 spacer column so the rendered letters don't run together at
; the chip's native 1:1 column density — the upstream demo packs the
; glyphs contiguously (25 bytes), this web-local variant adds the
; four inter-glyph gaps (29 bytes total) for readability.
; ============================================================================

hello_data:
        .byte 7Fh, 08h, 08h, 08h, 7Fh        ; H
        .byte 00h, 7Fh, 49h, 49h, 49h, 41h   ; (sp) E
        .byte 00h, 7Fh, 40h, 40h, 40h, 40h   ; (sp) L
        .byte 00h, 7Fh, 40h, 40h, 40h, 40h   ; (sp) L
        .byte 00h, 3Eh, 41h, 41h, 41h, 3Eh   ; (sp) O
hello_end:

; ============================================================================
; I2C bit-bang primitives (copied verbatim from i2c_ds1307_set.s)
; ============================================================================

i2cstart:
        push    r1
        la      r1, -65504      ; I2C base = 0xFF0020 (SCL @ +0, SDA @ +1)
        lcu     r0, 1
        sb      r0, 1(r1)
        sb      r0, 0(r1)
        lc      r0, 0
        sb      r0, 1(r1)
        sb      r0, 0(r1)
        pop     r1
        jmp     (r1)

i2cstop:
        push    r1
        la      r1, -65504
        lc      r0, 0
        sb      r0, 1(r1)
        lcu     r0, 1
        sb      r0, 0(r1)
        sb      r0, 1(r1)
        lc      r0, 0
        sb      r0, 0(r1)
        pop     r1
        jmp     (r1)

i2cwrite:
        push    r1
        push    fp
        push    r0
        mov     fp, sp
        lcu     r2, 80h
.iw_loop:
        lbu     r0, 0(fp)
        and     r0, r2
        ceq     r0, z
        brt     .iw_zero
        lc      r0, 1
        bra     .iw_set
.iw_zero:
        lc      r0, 0
.iw_set:
        la      r1, -65504
        sb      r0, 1(r1)
        lcu     r0, 1
        sb      r0, 0(r1)
        lc      r0, 0
        sb      r0, 0(r1)
        lc      r1, 1
        srl     r2, r1
        ceq     r2, z
        brf     .iw_loop
        la      r1, -65504
        lcu     r0, 1
        sb      r0, 1(r1)
        sb      r0, 0(r1)
        lbu     r0, 1(r1)
        lcu     r2, 1
        and     r0, r2
        push    r0
        lc      r2, 0
        la      r1, -65504
        sb      r2, 0(r1)
        pop     r0
        add     sp, 3
        pop     fp
        pop     r1
        jmp     (r1)
