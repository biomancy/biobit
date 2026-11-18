#![allow(dead_code)]

#[path = "ops/mod.rs"]
pub mod ops;

use biobit_bitpack::Scalar;
use ops::{Op, TestCodec};

#[derive(Clone, Copy, Debug)]
pub struct TinyRng {
    state: u64,
}

impl TinyRng {
    pub const fn new(seed: u64) -> Self {
        Self { state: seed }
    }

    #[inline(always)]
    pub fn u64(&mut self) -> u64 {
        let mut value = self.state;

        value ^= value >> 12;
        value ^= value << 25;
        value ^= value >> 27;
        self.state = value;

        value.wrapping_mul(0x2545_f491_4f6c_dd1d)
    }

    #[inline(always)]
    pub fn usize(&mut self) -> usize {
        self.u64() as usize
    }

    #[inline(always)]
    pub fn range(&mut self, end: usize) -> usize {
        assert!(end > 0);

        self.usize() % end
    }

    #[inline(always)]
    pub fn scalar<C>(&mut self) -> Scalar<C>
    where
        C: TestCodec,
    {
        C::scalar(self.usize())
    }

    pub fn op<C>(&mut self, len: usize) -> Op<C>
    where
        C: TestCodec,
    {
        match self.range(12) {
            0 => Op::push_left(self.scalar()),
            1 | 2 => Op::push_right(self.scalar()),
            3 => Op::drop_left(),
            4 => Op::drop_right(),
            5 | 6 => Op::set(self.position(len), self.scalar()),
            7 => Op::slide_left(self.scalar()),
            8 => Op::slide_right(self.scalar()),
            9 | 10 => Op::reverse(),
            _ => Op::complement(),
        }
    }

    #[inline(always)]
    fn position(&mut self, len: usize) -> usize {
        if len == 0 || self.range(5) == 0 {
            len + self.range(3)
        } else {
            self.range(len)
        }
    }
}
