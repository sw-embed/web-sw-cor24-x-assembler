; I2C OLED Thermometer: two-device i2c demo combining TMP101 (temperature
; sensor at 0x4A) and SSD1306 (OLED at 0x3C). Initializes the OLED, then
; loops forever reading the TMP101 temperature high byte and rendering it
; on the display as a 5-character "+NN\xB0C" / "-NN\xB0C" readout at page 0
; column 0. Redraw every ~10k spin iterations.
;
; Web-local (lives in this repo, not the sibling sw-cor24-x-assembler
; example set) for the same reasons as tmp101_read.s:
;   - Never halts; CLI runs would spin forever.
;   - Bit-bang primitives are kept inline rather than .include'd because
;     the assembler has no include directive and a single-file demo
;     reads better in the web editor.
;
; The TMP101's slider on the panel drives the displayed value within a
; tick or two -- drag it and watch the OLED follow.
;
; MMIO:
;     0xFF0020 = SCL  (bit 0 significant; open-drain)
;     0xFF0021 = SDA  (same)
;
; SSD1306 protocol (per i2c_ssd1306_hello.s for full detail):
;   - addr 0x3C << 1, write byte = 0x78
;   - control 0x00 = subsequent bytes are commands; 0x40 = data
;
; TMP101 read protocol (per tmp101_read.s for full detail):
;   - addr 0x4A << 1 | R = 0x95
;   - pointer register defaults to 0 (temperature) at reset, so a plain
;     read returns the high byte = whole degrees C in 8-bit two's
;     complement (-128..+127)
;
; Sign and magnitude:
;   - bit 7 set → negative; magnitude = -raw (mod 2^8)
;   - bit 7 clear → positive; magnitude = raw
;   - magnitude is clipped to 99 for display so "+99\xB0C" is the largest
;     readable readout. -128 thus renders as "-99\xB0C" rather than
;     overflowing the digit field.
;
; This is loop-forever by design.

        ; --- main ---
        la      r0, 0FEEC00h
        mov     sp, r0

        ; ===== SSD1306 init burst (verbatim from i2c_ssd1306_rtc_clock.s) =====
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
        lcu     r0, 0AFh        ; display on
        la      r1, .ret_i10
        la      r2, i2cwrite
        jal     r1, (r2)
.ret_i10:
        la      r1, .ret_init_sp
        la      r2, i2cstop
        jal     r1, (r2)
.ret_init_sp:

; ============================================================================
; Forever loop: read TMP101 -> sign-split -> reposition OLED -> render -> delay
; ============================================================================

main_loop:
        ; ----- TMP101 read: one byte -----
        la      r1, .ml_rs
        la      r2, i2cstart
        jal     r1, (r2)
.ml_rs:
        lcu     r0, 95h         ; 0x4A << 1 | R
        la      r1, .ml_rw
        la      r2, i2cwrite
        jal     r1, (r2)
.ml_rw:
        la      r1, .ml_rr
        la      r2, i2cread
        jal     r1, (r2)
.ml_rr:
        push    r0              ; T_raw at sp+0 across the stop
        la      r1, .ml_rsp
        la      r2, i2cstop
        jal     r1, (r2)
.ml_rsp:
        pop     r0              ; r0 = T_raw (0..255)

        ; ----- Sign-split: r0 = |T|, r2 = sign_glyph addr -----
        ; If bit 7 set, T is negative: |T| = (-T) & 0xFF; emit '-'.
        ; Else positive: |T| = T; emit '+'.
        push    r0              ; save raw across the test
        lcu     r1, 80h
        and     r0, r1          ; r0 = T_raw & 0x80
        ceq     r0, z
        brt     .ml_pos
        ; negative branch: r0 = 0 - T_raw, then mask to 8 bits
        pop     r1              ; r1 = T_raw
        lc      r0, 0
        sub     r0, r1          ; r0 = -T_raw (mod 2^24)
        lcu     r1, 0FFh
        and     r0, r1          ; r0 = |T| in 0..128
        la      r2, sign_minus
        bra     .ml_have_sign
.ml_pos:
        pop     r0              ; r0 = T_raw (already non-negative)
        la      r2, sign_plus
.ml_have_sign:

        ; ----- Clamp |T| to <= 99 so digit extraction stays 2-wide -----
        push    r2              ; preserve sign glyph addr
        lcu     r1, 100
        clu     r0, r1          ; r0 < 100?
        brt     .ml_ok
        lcu     r0, 99
.ml_ok:
        pop     r2

        ; ----- Stash sign glyph + abs value on fp frame -----
        ; Stack grows down (push decrements sp), so the LAST push has
        ; the LOWEST address -- i.e. fp+0.
        ; Frame layout (relative to fp set below):
        ;   fp+0  = |T|              (pushed last)
        ;   fp+3  = sign glyph addr
        ;   fp+6  = saved fp         (pushed first)
        push    fp
        push    r2              ; sign glyph addr
        push    r0              ; |T|
        mov     fp, sp

        ; ----- OLED: reposition pointer to (page 0, col 0) -----
        la      r1, .ml_pos_st
        la      r2, i2cstart
        jal     r1, (r2)
.ml_pos_st:
        lcu     r0, 78h
        la      r1, .ml_pos_a
        la      r2, i2cwrite
        jal     r1, (r2)
.ml_pos_a:
        lc      r0, 0           ; control: commands
        la      r1, .ml_pos_c
        la      r2, i2cwrite
        jal     r1, (r2)
.ml_pos_c:
        lcu     r0, 0B0h        ; page = 0
        la      r1, .ml_pos_p
        la      r2, i2cwrite
        jal     r1, (r2)
.ml_pos_p:
        lc      r0, 0           ; col low = 0
        la      r1, .ml_pos_cl
        la      r2, i2cwrite
        jal     r1, (r2)
.ml_pos_cl:
        lcu     r0, 10h         ; col high = 0
        la      r1, .ml_pos_ch
        la      r2, i2cwrite
        jal     r1, (r2)
.ml_pos_ch:
        la      r1, .ml_pos_sp
        la      r2, i2cstop
        jal     r1, (r2)
.ml_pos_sp:

        ; ----- OLED: data burst, "<sign><tens><ones>\xB0C" = 5 glyphs x 5 cols -----
        la      r1, .ml_dat_st
        la      r2, i2cstart
        jal     r1, (r2)
.ml_dat_st:
        lcu     r0, 78h
        la      r1, .ml_dat_a
        la      r2, i2cwrite
        jal     r1, (r2)
.ml_dat_a:
        lcu     r0, 40h         ; control: data
        la      r1, .ml_dat_c
        la      r2, i2cwrite
        jal     r1, (r2)
.ml_dat_c:

        ; (1) sign glyph (at fp+3 per frame layout above)
        lw      r0, 3(fp)
        la      r1, .ml_w_sign
        la      r2, write5
        jal     r1, (r2)
.ml_w_sign:

        ; (2,3) Decompose |T| into tens, ones via subtract-by-10. After the
        ; loop r0 = ones, the count of subtractions = tens. |T| is at fp+0.
        lw      r0, 0(fp)
        lc      r1, 0           ; tens counter
.ml_tens:
        lcu     r2, 10
        clu     r0, r2          ; r0 < 10?
        brt     .ml_tens_done
        sub     r0, r2
        add     r1, 1
        bra     .ml_tens
.ml_tens_done:
        ; r0 = ones, r1 = tens. Save ones first so render_digit can clobber.
        push    r0              ; ones at sp+0
        mov     r0, r1
        la      r1, .ml_w_tens
        la      r2, render_digit
        jal     r1, (r2)
.ml_w_tens:
        pop     r0              ; r0 = ones
        la      r1, .ml_w_ones
        la      r2, render_digit
        jal     r1, (r2)
.ml_w_ones:

        ; (4) degree glyph
        la      r0, degree_glyph
        la      r1, .ml_w_deg
        la      r2, write5
        jal     r1, (r2)
.ml_w_deg:

        ; (5) 'C' glyph
        la      r0, c_glyph
        la      r1, .ml_w_c
        la      r2, write5
        jal     r1, (r2)
.ml_w_c:

        la      r1, .ml_dat_sp
        la      r2, i2cstop
        jal     r1, (r2)
.ml_dat_sp:

        ; ----- Teardown frame: discard |T| + sign-glyph-addr + saved fp -----
        mov     sp, fp
        add     sp, 6           ; discard |T| (3) + sign glyph (3)
        pop     fp

        ; Delay between samples (matches rtc_clock cadence)
        la      r1, .ml_dly
        la      r2, delay
        jal     r1, (r2)
.ml_dly:

        la      r2, main_loop
        jmp     (r2)

; ============================================================================
; render_digit(r0 = digit 0..9): write 5 font bytes through OPEN i2c data
; transaction. Compute glyph addr = digit_font + digit*5, then call write5.
; ============================================================================

render_digit:
        push    r1
        ; r0 = digit * 5 via 4 adds
        mov     r1, r0
        add     r0, r1
        add     r0, r1
        add     r0, r1
        add     r0, r1          ; r0 = 5*digit
        la      r2, digit_font
        add     r0, r2          ; r0 = glyph addr
        la      r1, .rd_ret
        la      r2, write5
        jal     r1, (r2)
.rd_ret:
        pop     r1
        jmp     (r1)

; ============================================================================
; write5(r0 = glyph addr): write 5 bytes from r0 through OPEN i2c data txn.
; Generic 5-byte writer used by render_digit and the fixed glyphs (sign,
; degree, C).
; ============================================================================

write5:
        push    r1
        push    fp
        push    r0              ; ptr at 0(fp)
        mov     fp, sp
        lc      r2, 5
.w5_loop:
        lw      r1, 0(fp)
        lbu     r0, 0(r1)
        push    r2              ; save counter across i2cwrite
        la      r1, .w5_ret
        la      r2, i2cwrite
        jal     r1, (r2)
.w5_ret:
        pop     r2
        lw      r0, 0(fp)
        add     r0, 1
        sw      r0, 0(fp)
        add     r2, -1
        ceq     r2, z
        brf     .w5_loop
        mov     sp, fp
        add     sp, 3
        pop     fp
        pop     r1
        jmp     (r1)

; ============================================================================
; delay: ~10k spin iterations between samples.
; ============================================================================

delay:
        push    r1
        la      r0, 10000
.dly_loop:
        ceq     r0, z
        brt     .dly_done
        add     r0, -1
        bra     .dly_loop
.dly_done:
        pop     r1
        jmp     (r1)

; ============================================================================
; Font tables -- 5x8 glyphs, LSB-at-top column encoding (matches SSD1306
; GDDRAM wire format).
;   digit_font : 0..9, 50 bytes  (verbatim from i2c_ssd1306_rtc_clock.s)
;   sign_plus  : '+'   5 bytes
;   sign_minus : '-'   5 bytes
;   degree_glyph : 0xB0 5 bytes  (small circle in top-left)
;   c_glyph    : 'C'   5 bytes
; ============================================================================

digit_font:
        .byte 3Eh, 51h, 49h, 45h, 3Eh   ; 0
        .byte 00h, 42h, 7Fh, 40h, 00h   ; 1
        .byte 42h, 61h, 51h, 49h, 46h   ; 2
        .byte 21h, 41h, 45h, 4Bh, 31h   ; 3
        .byte 18h, 14h, 12h, 7Fh, 10h   ; 4
        .byte 27h, 45h, 45h, 45h, 39h   ; 5
        .byte 3Ch, 4Ah, 49h, 49h, 30h   ; 6
        .byte 01h, 71h, 09h, 05h, 03h   ; 7
        .byte 36h, 49h, 49h, 49h, 36h   ; 8
        .byte 06h, 49h, 49h, 29h, 1Eh   ; 9

sign_plus:
        .byte 08h, 08h, 3Eh, 08h, 08h   ; +

sign_minus:
        .byte 08h, 08h, 08h, 08h, 08h   ; -

degree_glyph:
        .byte 06h, 09h, 09h, 06h, 00h   ; \xB0 -- small circle, top-left

c_glyph:
        .byte 3Eh, 41h, 41h, 41h, 22h   ; C

; ============================================================================
; I2C bit-bang primitives (copied verbatim from i2c_ds1307_read.s).
; ============================================================================

i2cstart:
        push    r1
        la      r1, -65504
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

i2cread:
        push    r1
        push    fp
        lc      r0, 0
        push    r0              ; acc at 0(fp)
        mov     fp, sp
        lcu     r2, 8           ; counter
.ir_loop:
        la      r1, -65504
        lcu     r0, 1
        sb      r0, 1(r1)       ; SDA = 1 (release for slave)
        sb      r0, 0(r1)       ; SCL = 1 (slave puts bit on line)
        lbu     r0, 1(r1)       ; r0 = bit
        push    r0
        lw      r0, 0(fp)
        add     r0, r0
        pop     r1
        or      r0, r1
        sw      r0, 0(fp)
        la      r1, -65504
        lc      r0, 0
        sb      r0, 0(r1)       ; SCL = 0
        add     r2, -1
        ceq     r2, z
        brf     .ir_loop
        ; --- master NAK (9th clock): SDA=1 says "I'm done; slave please
        ; release SDA before STOP". Without the dcemu fix at 0736978
        ; this hung the bus state machine; with it, the slave clears
        ; its SDA pull on the 9th SCL fall and the subsequent i2cstop
        ; fires a clean STOP. ---
        la      r1, -65504
        lcu     r0, 1
        sb      r0, 1(r1)       ; SDA = 1 (NAK)
        sb      r0, 0(r1)       ; SCL = 1
        lc      r0, 0
        sb      r0, 0(r1)       ; SCL = 0
        pop     r0
        pop     fp
        pop     r1
        jmp     (r1)
