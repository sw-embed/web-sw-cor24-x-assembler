; TMP125 SPI-read demo (web tight loop, no idle delay).
;
; Continuously selects the SPI slave at SELN, exchanges 2 dummy bytes
; to clock out the chip's 16-bit temperature register, deselects, and
; prints the high byte as two ASCII hex digits + newline. The slider
; on the TMP125 panel drives the printed value within a tick or two.
;
; This demo is web-specific (lives in this repo) because it loops
; forever (no halt) — the CLI/test runners wouldn't want that.
;
; MMIO:
;     0xFF0030 = SPI MISO (read) / MOSI (write); same byte (bit 0)
;     0xFF0031 = SCLK     (bit 0; 0 = low, 1 = high)
;     0xFF0032 = SELN     (bit 0; active-low: 0 = selected, 1 = idle)
;     0xFF0100 = UART data; 0xFF0101 = UART status (bit 7 = TX busy)
;
; Calling convention (matches sibling examples):
;   r0 = first arg / return value; r1 = link (caller-set via `la r1,...`);
;   r2 = scratch. Subroutines with deeper calls push r1.

        ; --- main ---
_main:
        la      r0, 0FEEC00h    ; top of EBR
        mov     sp, r0

loop:
        ; --- select slave: SELN = 0 ---
        la      r1, 0FF0032h
        lc      r0, 0
        sb      r0, 0(r1)

        ; --- exchange first byte (high temp byte into r0) ---
        ; Input byte (MOSI) is 0; output (MISO) returns slave's byte.
        lc      r0, 0
        la      r1, .ret_hb
        la      r2, spi_xchg
        jal     r1, (r2)
.ret_hb:
        push    r0              ; save high byte across the next call

        ; --- exchange second byte (low temp byte, discarded) ---
        lc      r0, 0
        la      r1, .ret_lb
        la      r2, spi_xchg
        jal     r1, (r2)
.ret_lb:

        ; --- deselect slave: SELN = 1 ---
        la      r1, 0FF0032h
        lcu     r0, 1
        sb      r0, 0(r1)

        ; --- print high byte as two hex digits + newline ---
        pop     r0              ; saved high byte
        push    r0              ; keep for second nibble
        lc      r1, 4
        srl     r0, r1          ; upper nibble
        la      r1, .ret_hi
        la      r2, print_hex_nibble
        jal     r1, (r2)
.ret_hi:

        pop     r0              ; full byte again
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

        bra     loop            ; flat-out loop (no idle delay)

; ============================================================================
; SPI 8-bit exchange.
; Input:  r0 = byte to drive on MOSI (low 8 bits significant).
; Output: r0 = byte clocked in from MISO.
; Bit-bang via the SCLK / MOSI MMIO lines; SELN is driven by the caller.
;
; Uses the same MSB-walk-with-mask approach as i2cwrite: r2 is a mask
; that walks 0x80 -> 0x40 -> ... -> 0x01, and we test (byte & mask) to
; pick the MOSI bit. Earlier draft tried (byte<<24-shift) + cls/mov-c
; -- that's wrong for a u8 sign-extended to 24 bits because the sign
; bit (bit 23) was always 0.
;
; Frame: pushed r1, pushed fp, pushed (byte_in = r0), pushed (acc = 0).
; Locals: byte_in at 3(fp), acc at 0(fp).
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
        ; SCLK low
        la      r1, 0FF0031h
        lc      r0, 0
        sb      r0, 0(r1)

        ; Pick MOSI bit: (byte_in & mask) ? 1 : 0
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

        ; SCLK high (slave drives MISO on the rising edge)
        la      r1, 0FF0031h
        lcu     r0, 1
        sb      r0, 0(r1)

        ; Sample MISO into acc.
        la      r1, 0FF0030h
        lbu     r0, 0(r1)
        push    r0              ; bit on stack temporarily
        lw      r0, 4(fp)       ; current acc (fp shifted by push)
        add     r0, r0          ; acc <<= 1
        pop     r1              ; bit into r1
        or      r0, r1
        sw      r0, 0(fp)       ; updated acc

        ; mask >>= 1; loop until mask hits 0
        lc      r1, 1
        srl     r2, r1
        ceq     r2, z
        brf     .sx_loop

        ; --- teardown ---
        pop     r0              ; r0 = acc (return value)
        add     sp, 3           ; discard byte_in slot
        pop     fp
        pop     r1
        jmp     (r1)

; ============================================================================
; UART
; ============================================================================

; putc(r0): poll TX-busy clear, then write low byte of r0 to UART data.
putc:
        push    r1
        push    r0
        la      r1, -65280      ; UART base = 0xFF0100
.putc_wait:
        lb      r2, 1(r1)       ; status (bit 7 = TX busy)
        cls     r2, z
        brt     .putc_wait
        pop     r0
        sb      r0, 0(r1)
        pop     r1
        jmp     (r1)

; print_hex_nibble(r0): emit r0 (low 4 bits) as '0'-'9' or 'A'-'F'.
print_hex_nibble:
        push    r1
        lcu     r2, 10
        clu     r0, r2          ; r0 < 10?
        brt     .phn_digit
        add     r0, 55          ; 'A' - 10
        bra     .phn_emit
.phn_digit:
        add     r0, 48          ; '0'
.phn_emit:
        la      r2, putc
        jal     r1, (r2)
        pop     r1
        jmp     (r1)
