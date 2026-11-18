use crate::scalar;
use crate::store::{Complement, StaticStore};

macro_rules! impl_complement {
    ($array_store:ty => $($scalar_store:ty),+ $(,)?) => {
        $(
            unsafe impl Complement<$scalar_store> for $array_store
            {
                #[inline(always)]
                unsafe fn complement_unchecked<ScalarLayout>(&mut self, len: usize)
                where
                    ScalarLayout: scalar::Layout<Store = $scalar_store>
                {
                    let used = len * ScalarLayout::BITS.get();
                    let padding = <$array_store as StaticStore>::CAPACITY_BITS.get() - used;

                    *self ^= <$array_store>::MAX << padding;
                }
            }
        )+
    };
}

impl_complement!(u8 => u8);
impl_complement!(u16 => u8, u16);
impl_complement!(u32 => u8, u16, u32);
impl_complement!(u64 => u8, u16, u32, u64);

#[cfg(test)]
mod tests {
    use super::Complement;
    use crate::Bits;

    #[test]
    fn complement_flips_only_active_bits() {
        let mut store = 0b00_01_10_00u8;

        unsafe { store.complement_unchecked::<Bits<2>>(3) };

        assert_eq!(store, 0b11_10_01_00);
    }

    #[test]
    fn complement_flips_full_store() {
        let mut store = 0b00_01_10_11u8;

        unsafe { store.complement_unchecked::<Bits<2>>(4) };

        assert_eq!(store, 0b11_10_01_00);
    }

    #[test]
    fn complement_supports_wider_scalar_stores() {
        let mut store = 0x0001_e000u32;

        unsafe { store.complement_unchecked::<Bits<9>>(2) };

        assert_eq!(store, 0xfffe_2000);
    }
}
