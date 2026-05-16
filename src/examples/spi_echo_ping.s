; SPI Echo Ping demo (web tight loop, no idle delay).
;
; Exercises the SPI bus against the emulator's `EchoDevice` test
; slave. The chip's contract: on every 8-clock exchange it drives
; its buffered byte on MISO, then latches the just-clocked MOSI as
; the buffer for the NEXT exchange. Each loop iteration sends two
; bytes:
;
;   xchg(0xA5)   ; clocks out the buffer (whatever the slider poked,
;                ; or 0x00 by default); latches 0xA5 for next time.
;   xchg(0x00)   ; clocks out 0xA5 -- that's the byte we print.
;
; So the printed value is 0xA5 every iteration regardless of the
; slider, because we always re-seed it ourselves. The slider's value
; shows up on the FIRST iteration's first exchange (which we discard
; by overwriting with 0xA5 on the next clock). To watch the slider's
; effect in real time, watch the SPI Test Device card's `buffer`
; readout -- the panel snapshot reflects the live device state.
;
; MMIO:
;   0xFF0030 = SPI MISO / MOSI (bit 0); 0xFF0031 = SCLK; 0xFF0032 = SELN.
;   0xFF0100 = UART data; 0xFF0101 = UART status (bit 7 = TX busy).

        ; --- main ---
_main:
        la      r0, 0FEEC00h    ; top of EBR
        mov     sp, r0

loop:
        ; --- select slave: SELN = 0 ---
        la      r1, 0FF0032h
        lc      r0, 0
        sb      r0, 0(r1)

        ; --- xchg(0xA5): clocks out the buffer, latches 0xA5 ---
        lcu     r0, 0A5h
        la      r1, .ret_seed
        la      r2, spi_xchg
        jal     r1, (r2)
.ret_seed:

        ; --- xchg(0x00): clocks out 0xA5 (the byte we just latched) ---
        lc      r0, 0
        la      r1, .ret_echo
        la      r2, spi_xchg
        jal     r1, (r2)
.ret_echo:
        push    r0              ; save the echoed byte

        ; --- deselect slave: SELN = 1 ---
        la      r1, 0FF0032h
        lcu     r0, 1
        sb      r0, 0(r1)

        ; --- print echoed byte as two hex digits + newline ---
        pop     r0
        push    r0
        lc      r1, 4
        srl     r0, r1          ; upper nibble
        la      r1, .ret_hi
        la      r2, print_hex_nibble
        jal     r1, (r2)
.ret_hi:

        pop     r0
        lcu     r2, 0Fh
        and     r0, r2          ; lower nibble
        la      r1, .ret_lo
        la      r2, print_hex_nibble
        jal     r1, (r2)
.ret_lo:

        lc      r0, 10          ; '\n'
        la      r1, .ret_nl
        la      r2, putc
        jal     r1, (r2)
.ret_nl:

        bra     loop

; ============================================================================
; SPI 8-bit exchange (MSB-walk-with-mask, same as the I2C test device
; demo's i2cwrite).
; Input:  r0 = byte to drive on MOSI.
; Output: r0 = byte clocked in from MISO.
; ============================================================================
spi_xchg:
        push    r1
        push    fp
        push    r0              ; byte_in at offset 3(fp)
        lc      r0, 0
        push    r0              ; acc = 0 at offset 0(fp)
        mov     fp, sp
        lcu     r2, 80h         ; mask walks 0x80 -> 0x01
.sx_loop:
        la      r1, 0FF0031h
        lc      r0, 0
        sb      r0, 0(r1)       ; SCLK = 0

        lbu     r0, 3(fp)
        and     r0, r2
        ceq     r0, z
        brt     .sx_zero
        lc      r0, 1
        bra     .sx_emit
.sx_zero:
        lc      r0, 0
.sx_emit:
        la      r1, 0FF0030h
        sb      r0, 0(r1)       ; MOSI = bit

        la      r1, 0FF0031h
        lcu     r0, 1
        sb      r0, 0(r1)       ; SCLK = 1

        la      r1, 0FF0030h
        lbu     r0, 0(r1)
        push    r0
        lw      r0, 4(fp)       ; acc
        add     r0, r0
        pop     r1
        or      r0, r1
        sw      r0, 0(fp)

        lc      r1, 1
        srl     r2, r1
        ceq     r2, z
        brf     .sx_loop

        pop     r0
        add     sp, 3
        pop     fp
        pop     r1
        jmp     (r1)

; ============================================================================
; UART
; ============================================================================

putc:
        push    r1
        push    r0
        la      r1, -65280
.putc_wait:
        lb      r2, 1(r1)
        cls     r2, z
        brt     .putc_wait
        pop     r0
        sb      r0, 0(r1)
        pop     r1
        jmp     (r1)

print_hex_nibble:
        push    r1
        lcu     r2, 10
        clu     r0, r2
        brt     .phn_digit
        add     r0, 55
        bra     .phn_emit
.phn_digit:
        add     r0, 48
.phn_emit:
        la      r2, putc
        jal     r1, (r2)
        pop     r1
        jmp     (r1)
