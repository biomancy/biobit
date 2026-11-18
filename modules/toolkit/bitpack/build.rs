use std::env;
use std::fs;
use std::path::Path;

fn generate_bitpack_len_impls() -> String {
    let mut len = String::new();

    for n in 1..=128 {
        len.push_str(&format!(
            "impl crate::len::sealed::Supported for crate::len::Len<{n}> {{}}\n"
        ));
    }

    for n in 1..128 {
        let next = n + 1;
        len.push_str(&format!(
            "impl crate::len::Next for crate::len::Len<{n}> {{
                type Next = crate::len::Len<{next}>;
            }}\n"
        ));
    }

    for n in 2..=128 {
        let prev = n - 1;
        len.push_str(&format!(
            "impl crate::len::Prev for crate::len::Len<{n}> {{
                type Prev = crate::len::Len<{prev}>;
            }}\n"
        ));
    }

    len
}

fn generate_bitpack_scalar_layout_impls() -> String {
    let mut scalar = String::new();

    for n in 1..=64 {
        let (scalar_store, scalar_store_bits) = match n {
            1..=8 => ("u8", 8),
            9..=16 => ("u16", 16),
            17..=32 => ("u32", 32),
            33..=64 => ("u64", 64),
            _ => unreachable!(),
        };
        let scalar_mask = if n == scalar_store_bits {
            format!("<{scalar_store}>::MAX")
        } else {
            format!("<{scalar_store}>::MAX >> {}", scalar_store_bits - n)
        };

        // SAFETY: The selected integer fits n bits and the mask selects exactly its low n bits.
        scalar.push_str(&format!(
            "unsafe impl crate::scalar::Layout for crate::codec::Bits<{n}> {{
                type Store = {scalar_store};
                const BITS: std::num::NonZeroUsize = std::num::NonZeroUsize::new({n}).unwrap();
                const MASK: Self::Store = {scalar_mask};
            }}\n"
        ));
    }

    scalar
}

fn generate_bitpack_array_layout_impls() -> String {
    let mut array = String::new();

    for bits in 1..=64 {
        for len in 1..=128 {
            let total_bits: usize = bits * len;
            let array_store = match total_bits {
                1..=8 => "u8".to_owned(),
                9..=16 => "u16".to_owned(),
                17..=32 => "u32".to_owned(),
                33..=64 => "u64".to_owned(),
                _ => {
                    let words = total_bits.div_ceil(64usize);
                    format!("[u64; {words}]")
                }
            };

            // SAFETY: The capacity ranges and rounded word count select the smallest fitting store.
            array.push_str(&format!(
                "unsafe impl crate::array::Layout<crate::len::Len<{len}>> for crate::codec::Bits<{bits}> {{
                    type Store = {array_store};
                }}\n"
            ));
        }
    }

    array
}

fn generate_bitpack_word_store_impls() -> String {
    let mut store = String::new();

    for words in 2..=128 {
        let capacity_bits = words * 64;

        store.push_str(&format!(
            "impl crate::store::Store for [u64; {words}] {{}}\n"
        ));

        store.push_str(&format!(
            "impl crate::store::StaticStore for [u64; {words}] {{
                const ZERO: Self = [0; {words}];
                const CAPACITY_BITS: std::num::NonZeroUsize =
                    std::num::NonZeroUsize::new({capacity_bits}).unwrap();
            }}\n"
        ));
    }

    store
}

fn main() {
    let out_dir = env::var_os("OUT_DIR").unwrap();

    fs::write(
        Path::new(&out_dir).join("bitpack_len_impls.rs"),
        generate_bitpack_len_impls(),
    )
    .unwrap();
    fs::write(
        Path::new(&out_dir).join("bitpack_scalar_layout_impls.rs"),
        generate_bitpack_scalar_layout_impls(),
    )
    .unwrap();
    fs::write(
        Path::new(&out_dir).join("bitpack_array_layout_impls.rs"),
        generate_bitpack_array_layout_impls(),
    )
    .unwrap();
    fs::write(
        Path::new(&out_dir).join("bitpack_word_store_impls.rs"),
        generate_bitpack_word_store_impls(),
    )
    .unwrap();

    println!("cargo:rerun-if-changed=build.rs");
}
