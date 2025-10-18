pub struct HexDisplay([u8; 32]);

impl core::fmt::Debug for HexDisplay {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        std::fmt::Display::fmt(self, f)
    }
}

impl core::fmt::Display for HexDisplay {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        // this implementation avoids any heap allocations and is optimized by
        // the compiler to use vectorized instructions where available.
        //
        // with O3, the loop is unrolled and vectorized to use SIMD instructions
        //
        // https://rust.godbolt.org/z/seM19zEfv

        let mut buf = [0u8; 64];
        // SAFETY: 64 is evenly divisible by 2
        unsafe { buf.as_chunks_unchecked_mut::<2>() }
            .iter_mut()
            .zip(self.0.as_ref())
            .for_each(|(slot, &byte)| {
                *slot = byte_to_hex(byte);
            });

        // SAFETY: buf only contains valid ASCII hex characters
        let buf = unsafe { core::str::from_utf8_unchecked(&buf) };

        f.pad(buf)
    }
}

const fn byte_to_hex(byte: u8) -> [u8; 2] {
    const unsafe fn nibble_to_hex(nibble: u8) -> u8 {
        match nibble {
            0..=9 => b'0' + nibble,
            10..=15 => b'a' + (nibble - 10),
            // SAFETY: invariant held by caller that nibble is in 0..=15
            _ => unsafe { core::hint::unreachable_unchecked() },
        }
    }

    // SAFETY: shifting and masking ensures nibble is in 0..=15 for both calls
    unsafe { [nibble_to_hex(byte >> 4), nibble_to_hex(byte & 0x0F)] }
}

pub trait HexDisplayExt {
    fn hex_display(self) -> HexDisplay;
}

impl<T: Into<[u8; 32]>> HexDisplayExt for T {
    fn hex_display(self) -> HexDisplay {
        HexDisplay(self.into())
    }
}
