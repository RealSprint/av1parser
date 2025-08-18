use std::io;

/// numeric cast helper (u32 as T)
pub trait FromU32 {
    fn from_u32(v: u32) -> Self;
}

impl FromU32 for bool {
    #[inline]
    fn from_u32(v: u32) -> Self {
        v != 0
    }
}

macro_rules! impl_from_u32 {
    ($($ty:ty)*) => {
        $(
            impl FromU32 for $ty {
            #[inline]
                fn from_u32(v: u32) -> $ty {
                    v as $ty
                }
            }
        )*
    }
}

impl_from_u32!(u8 u16 u32 u64 usize);

///
/// Bitwise reader
///
pub struct BitReader<R> {
    inner: R,
    bbuf: u8,
    bpos: u8,
    pos: usize, // current bit position
}

impl<R: io::Read> BitReader<R> {
    pub fn new(inner: R) -> BitReader<R> {
        BitReader {
            inner,
            bbuf: 0,
            bpos: 0,
            pos: 0,
        }
    }

    pub fn get_position(&self) -> usize {
        self.pos
    }

    /// read_bit: read 1 bit
    pub fn read_bit(&mut self) -> Option<u8> {
        if self.bpos == 0 {
            let mut bbuf = [0; 1];
            match self.inner.read(&mut bbuf) {
                Ok(0) | Err(_) => return None, // EOF or IOErr
                Ok(n) => assert_eq!(n, 1),
            }
            self.bbuf = bbuf[0];
            self.bpos = 8;
        }
        self.bpos -= 1;
        self.pos += 1;
        Some((self.bbuf >> self.bpos) & 1)
    }

    pub fn skip(&mut self, n: usize) -> Option<()> {
        for _ in 0..n {
            if self.read_bit().is_none() {
                return None; // EOF
            }
        }
        Some(())
    }

    /// f(n): read n-bits
    pub fn f<T: FromU32>(&mut self, nbit: usize) -> Option<T> {
        assert!(nbit <= 32);
        let mut x: u32 = 0;
        for _ in 0..nbit {
            x = (x << 1) | self.read_bit()? as u32;
        }
        Some(FromU32::from_u32(x))
    }

    /// su(n)
    pub fn su(&mut self, n: usize) -> Option<i32> {
        let mut value = self.f::<u32>(n)? as i32;
        let sign_mask = 1 << (n - 1);
        if value & sign_mask != 0 {
            value -= 2 * sign_mask
        }
        Some(value)
    }

    /// ns(n)
    pub fn ns(&mut self, n: u32) -> Option<u32> {
        let w = Self::floor_log2(n) + 1;
        let m = (1 << w) - n;
        let v = self.f::<u32>(w as usize - 1)?; // f(w - 1)
        if v < m {
            return Some(v);
        }
        let extra_bit = self.f::<u32>(1)?; // f(1)
        Some((v << 1) - m + extra_bit)
    }

    pub fn uvlc(&mut self) -> Option<u64> {
        let mut leading_zeros = 0;
        loop {
            let done = self.read_bit()? > 0;
            if done {
                break;
            }
            leading_zeros += 1;
        }

        if leading_zeros >= 32 {
            return Some((1 << 32) - 1);
        }

        let value = self.f::<u64>(leading_zeros as usize)?;

        Some(value + (1 << leading_zeros) - 1)
    }

    pub fn le<T: FromU32>(&mut self, n: usize) -> Option<T> {
        let mut t = 0;
        for i in 0..n {
            let byte: u32 = self.f(8).unwrap();
            t += byte << (i * 8)
        }
        return Some(FromU32::from_u32(t));
    }

    // FloorLog2(x)
    fn floor_log2(mut x: u32) -> u32 {
        let mut s = 0;
        while x != 0 {
            x >>= 1;
            s += 1;
        }
        s - 1
    }
}
