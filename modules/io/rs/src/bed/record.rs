use super::validate;
use biobit_core_rs::loc::{Interval, Orientation};
use derive_more::Into;
use eyre::Result;
use paste::paste;

#[cfg(feature = "bitcode")]
use bitcode::{Decode, Encode};

// BedOp and BedMutOp traits are nested into each other. This is opposite to nesting structures,
// which would prohibit memory layout optimization.

// All BedN types implement all Bed(N-1) traits.
// All BedN types can be directly converted Into Bed(N-1) all the way down to Bed3.

macro_rules! define_bed_struct {
    ($(($_:ident: $__:ty = $___:expr; $____:expr),)+) => {};
    (
        $(($field:ident: $ftype:ty = $default:expr; $fvalidate:expr),)+ $Bed:ident, $($tail:tt)*
    ) => {
        define_bed_struct!(
            $(($field: $ftype = $default; $fvalidate),)+ $($tail)*
        );

        #[cfg_attr(feature = "bitcode", derive(Encode, Decode))]
        #[derive(Debug, Clone, Ord, PartialOrd, Eq, PartialEq, Hash, Into)]
        pub struct $Bed {
            $(
                $field: $ftype,
            )+
        }

        impl $Bed {
            #[allow(clippy::too_many_arguments)]
            pub fn new($($field: $ftype),*) -> Result<Self> {
                $(
                    $fvalidate?;
                )+
                Ok(Self { $($field, )* })
            }

            paste! {
                #[allow(clippy::too_many_arguments)]
                #[allow(clippy::needless_borrow)]
                pub fn set(&mut self, $([<new_ $field>]: Option<$ftype>),*) -> Result<&mut Self> {
                    // Technically, it's excessive to validate all fields again. But I don't see a
                    // simple way to avoid it.
                    $(
                        let $field = [<new_ $field>].as_ref().unwrap_or(&self.$field);
                        $fvalidate?;
                    )+
                    $(
                        if let Some($field) = [<new_ $field>] {
                            self.$field = $field;
                        }
                    )+
                    Ok(self)
                }
            }
        }

        impl Default for $Bed {
            fn default() -> Self {
                Self {
                    $(
                        $field: $default,
                    )+
                }
            }
        }
    }
}

define_bed_struct!(
    (seqid: String = "_".repeat(255); validate::seqid(&seqid)),
    (interval: Interval<u64> = Interval::default(); validate::interval(&interval)),
    Bed3,
    (name: String = "_".repeat(255); validate::name(&name)),
    Bed4,
    (score: u16 = 0; validate::score(&score)),
    Bed5,
    (orientation: Orientation = Orientation::Dual; validate::orientation(&orientation)),
    Bed6,
    (thick: Interval<u64> = Interval::default(); validate::thick(&interval, &thick)),
    Bed8,
    (rgb: (u8, u8, u8) = (0, 0, 0); validate::rgb(&rgb)),
    Bed9,
    (blocks: Vec<Interval<u64>> = Vec::new(); validate::blocks(&interval, &blocks)),
    Bed12,
);

macro_rules! define_bed_traits {
    ($_:ident, ) => {};
    ($(($field:ident, $getter:ty, $setter:ty),)+ $Current:ident, $($tail:tt)*) => {
        paste! {
            define_bed_traits!($Current, $($tail)*);

            pub trait [<$Current Op>] {
                $(
                    fn $field(&self) -> $getter;
                )+
            }

            pub trait [<$Current MutOp>]: [<$Current Op>] {
                $(
                    fn [<set_ $field>](&mut self, $field: $setter) -> Result<&mut Self>;
                )+
            }
        }
    };
    (
        $Previous: ident, $(($field:ident, $getter:ty, $setter:ty),)+ $Current:ident, $($tail:tt)*
    ) => {
        paste! {
            define_bed_traits!($Current, $($tail)*);

            pub trait [<$Current Op>]: [<$Previous Op>] {
                $(
                    fn $field(&self) -> $getter;
                )+
            }

            pub trait [<$Current MutOp>]: [<$Previous MutOp>] + [<$Current Op>] {
                $(
                    fn [<set_ $field>](&mut self, $field: $setter) -> Result<&mut Self>;
                )+
            }
        }
    };
}

define_bed_traits!(
    (seqid, &str, String),
    (interval, &Interval<u64>, Interval<u64>),
    Bed3,
    (name, &str, String),
    Bed4,
    (score, u16, u16),
    Bed5,
    (orientation, Orientation, Orientation),
    Bed6,
    (thick, &Interval<u64>, Interval<u64>),
    Bed8,
    (rgb, (u8, u8, u8), (u8, u8, u8)),
    Bed9,
    (blocks, &[Interval<u64>], Vec<Interval<u64>>),
    Bed12,
);

macro_rules! impl_bed3_mut_op_trait {
    ([$_:tt],) => {};
    ([$checking:tt], $Bed:ident, $($tail:tt)*) => {
        impl_bed3_mut_op_trait!([$checking], $($tail)*);

        impl Bed3MutOp for $Bed {
            fn set_seqid(&mut self, seqid: String) -> Result<&mut Self> {
                validate::seqid(&seqid)?;
                self.seqid = seqid;
                Ok(self)
            }

            fn set_interval(&mut self, interval: Interval<u64>) -> Result<&mut Self> {
                impl_bed3_mut_op_trait!([self, interval, $checking]);
                self.interval = interval;
                Ok(self)
            }
        }
    };
    ([$slf:ident, $interval:ident, "plain"]) => {
        validate::interval(&$interval)?;
    };
    ([$slf:ident, $interval:ident, "with_thick"]) => {
        validate::interval(&$interval)?;
        validate::thick(&$interval, &$slf.thick)?;
    };
    ([$slf:ident, $interval:ident, "with_blocks"]) => {
        validate::interval(&$interval)?;
        validate::thick(&$interval, &$slf.thick)?;
        validate::blocks(&$interval, &$slf.blocks)?;
    };
}

impl_bed3_mut_op_trait!(["plain"], Bed3, Bed4, Bed5, Bed6,);
impl_bed3_mut_op_trait!(["with_thick"], Bed9, Bed8,);
impl_bed3_mut_op_trait!(["with_blocks"], Bed12,);

macro_rules! impl_bed_traits {
    ($Bed:ident, 3) => {
        impl Bed3Op for $Bed {
            fn seqid(&self) -> &str {
                &self.seqid
            }
            fn interval(&self) -> &Interval<u64> {
                &self.interval
            }
        }
    };
    ($Bed:ident, 4) => {
        impl_bed_traits!($Bed, 3);

        impl Bed4Op for $Bed {
            fn name(&self) -> &str {
                &self.name
            }
        }

        impl Bed4MutOp for $Bed {
            fn set_name(&mut self, name: String) -> Result<&mut Self> {
                validate::name(&name)?;
                self.name = name;
                Ok(self)
            }
        }
    };
    ($Bed:ident, 5) => {
        impl_bed_traits!($Bed, 4);

        impl Bed5Op for $Bed {
            fn score(&self) -> u16 {
                self.score
            }
        }

        impl Bed5MutOp for $Bed {
            fn set_score(&mut self, score: u16) -> Result<&mut Self> {
                validate::score(&score)?;
                self.score = score;
                Ok(self)
            }
        }
    };
    ($Bed:ident, 6) => {
        impl_bed_traits!($Bed, 5);

        impl Bed6Op for $Bed {
            fn orientation(&self) -> Orientation {
                self.orientation
            }
        }

        impl Bed6MutOp for $Bed {
            fn set_orientation(&mut self, orientation: Orientation) -> Result<&mut Self> {
                validate::orientation(&orientation)?;
                self.orientation = orientation;
                Ok(self)
            }
        }
    };
    ($Bed:ident, 8) => {
        impl_bed_traits!($Bed, 6);

        impl Bed8Op for $Bed {
            fn thick(&self) -> &Interval<u64> {
                &self.thick
            }
        }

        impl Bed8MutOp for $Bed {
            fn set_thick(&mut self, thick: Interval<u64>) -> Result<&mut Self> {
                validate::thick(&self.interval, &thick)?;
                self.thick = thick;
                Ok(self)
            }
        }
    };
    ($Bed:ident, 9) => {
        impl_bed_traits!($Bed, 8);

        impl Bed9Op for $Bed {
            fn rgb(&self) -> (u8, u8, u8) {
                self.rgb
            }
        }

        impl Bed9MutOp for $Bed {
            fn set_rgb(&mut self, rgb: (u8, u8, u8)) -> Result<&mut Self> {
                validate::rgb(&rgb)?;
                self.rgb = rgb;
                Ok(self)
            }
        }
    };
    ($Bed:ident, 12) => {
        impl_bed_traits!($Bed, 9);

        impl Bed12Op for $Bed {
            fn blocks(&self) -> &[Interval<u64>] {
                &self.blocks
            }
        }

        impl Bed12MutOp for $Bed {
            fn set_blocks(&mut self, blocks: Vec<Interval<u64>>) -> Result<&mut Self> {
                validate::blocks(&self.interval, &blocks)?;
                self.blocks = blocks;
                Ok(self)
            }
        }
    };
}

impl_bed_traits!(Bed3, 3);
impl_bed_traits!(Bed4, 4);
impl_bed_traits!(Bed5, 5);
impl_bed_traits!(Bed6, 6);
impl_bed_traits!(Bed8, 8);
impl_bed_traits!(Bed9, 9);
impl_bed_traits!(Bed12, 12);

macro_rules! impl_from_casts {
    ($Bed:ident, 3) => {
        impl From<$Bed> for Bed3 {
            fn from(bed: $Bed) -> Bed3 {
                Bed3 {
                    seqid: bed.seqid,
                    interval: bed.interval,
                }
            }
        }
    };
    ($Bed:ident, 4) => {
        impl_from_casts!($Bed, 3);

        impl From<$Bed> for Bed4 {
            fn from(bed: $Bed) -> Bed4 {
                Bed4 {
                    seqid: bed.seqid,
                    interval: bed.interval,
                    name: bed.name,
                }
            }
        }
    };
    ($Bed:ident, 5) => {
        impl_from_casts!($Bed, 4);

        impl From<$Bed> for Bed5 {
            fn from(bed: $Bed) -> Bed5 {
                Bed5 {
                    seqid: bed.seqid,
                    interval: bed.interval,
                    name: bed.name,
                    score: bed.score,
                }
            }
        }
    };
    ($Bed:ident, 6) => {
        impl_from_casts!($Bed, 5);

        impl From<$Bed> for Bed6 {
            fn from(bed: $Bed) -> Bed6 {
                Bed6 {
                    seqid: bed.seqid,
                    interval: bed.interval,
                    name: bed.name,
                    score: bed.score,
                    orientation: bed.orientation,
                }
            }
        }
    };
    ($Bed:ident, 8) => {
        impl_from_casts!($Bed, 6);

        impl From<$Bed> for Bed8 {
            fn from(bed: $Bed) -> Bed8 {
                Bed8 {
                    seqid: bed.seqid,
                    interval: bed.interval,
                    name: bed.name,
                    score: bed.score,
                    orientation: bed.orientation,
                    thick: bed.thick,
                }
            }
        }
    };
    ($Bed:ident, 9) => {
        impl_from_casts!($Bed, 8);

        impl From<$Bed> for Bed9 {
            fn from(bed: $Bed) -> Bed9 {
                Bed9 {
                    seqid: bed.seqid,
                    interval: bed.interval,
                    name: bed.name,
                    score: bed.score,
                    orientation: bed.orientation,
                    thick: bed.thick,
                    rgb: bed.rgb,
                }
            }
        }
    };
}

impl_from_casts!(Bed4, 3);
impl_from_casts!(Bed5, 4);
impl_from_casts!(Bed6, 5);
impl_from_casts!(Bed8, 6);
impl_from_casts!(Bed9, 8);
impl_from_casts!(Bed12, 9);

// Impl to_bedN() for each BedN-1 type
impl Bed3 {
    pub fn to_bed4(self, name: String) -> Result<Bed4> {
        validate::name(&name)?;
        Ok(Bed4 {
            seqid: self.seqid,
            interval: self.interval,
            name,
        })
    }
}

impl Bed4 {
    pub fn to_bed5(self, score: u16) -> Result<Bed5> {
        validate::score(&score)?;
        Ok(Bed5 {
            seqid: self.seqid,
            interval: self.interval,
            name: self.name,
            score,
        })
    }
}

impl Bed5 {
    pub fn to_bed6(self, orientation: Orientation) -> Result<Bed6> {
        validate::orientation(&orientation)?;
        Ok(Bed6 {
            seqid: self.seqid,
            interval: self.interval,
            name: self.name,
            score: self.score,
            orientation,
        })
    }
}

impl Bed6 {
    pub fn to_bed8(self, thick: Interval<u64>) -> Result<Bed8> {
        validate::thick(&self.interval, &thick)?;
        Ok(Bed8 {
            seqid: self.seqid,
            interval: self.interval,
            name: self.name,
            score: self.score,
            orientation: self.orientation,
            thick,
        })
    }
}

impl Bed8 {
    pub fn to_bed9(self, rgb: (u8, u8, u8)) -> Result<Bed9> {
        validate::rgb(&rgb)?;
        Ok(Bed9 {
            seqid: self.seqid,
            interval: self.interval,
            name: self.name,
            score: self.score,
            orientation: self.orientation,
            thick: self.thick,
            rgb,
        })
    }
}

impl Bed9 {
    pub fn to_bed12(self, blocks: Vec<Interval<u64>>) -> Result<Bed12> {
        validate::blocks(&self.interval, &blocks)?;
        Ok(Bed12 {
            seqid: self.seqid,
            interval: self.interval,
            name: self.name,
            score: self.score,
            orientation: self.orientation,
            thick: self.thick,
            rgb: self.rgb,
            blocks,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use itertools::iproduct;

    #[test]
    fn test_correct_bed_records() -> Result<()> {
        // All combinations produce correct BED records
        let blocks = vec![
            vec![Interval::new(0, 10)?, Interval::new(20, 100)?],
            vec![
                Interval::new(0, 1)?,
                Interval::new(5, 9)?,
                Interval::new(99, 100)?,
            ],
        ];

        for cmb in iproduct!(
            vec!["1", "chr", &("A".repeat(255))],
            vec![Interval::new(0, 100)?, Interval::new(1, 101)?],
            vec!["B", "name", &("B".repeat(255)), " "],
            vec![0, 500, 1000],
            vec![
                Orientation::Forward,
                Orientation::Reverse,
                Orientation::Dual
            ],
            vec![Interval::new(10, 50)?, Interval::new(49, 100)?],
            vec![(0, 0, 0), (0, 255, 0), (255, 255, 255)],
            blocks
        ) {
            let bed12 = Bed12::new(
                cmb.0.to_owned(),
                cmb.1,
                cmb.2.to_owned(),
                cmb.3,
                cmb.4,
                cmb.5,
                cmb.6,
                cmb.7,
            )?;

            // Test that all as_ref casts work
            let bed3 = Bed3::new(cmb.0.to_owned(), cmb.1)?;
            assert_eq!(&bed3, &bed12.clone().into());

            let bed4 = Bed4::new(cmb.0.to_owned(), cmb.1, cmb.2.to_owned())?;
            assert_eq!(&bed4, &bed12.clone().into());

            let bed5 = Bed5::new(cmb.0.to_owned(), cmb.1, cmb.2.to_owned(), cmb.3)?;
            assert_eq!(&bed5, &bed12.clone().into());

            let bed6 = Bed6::new(cmb.0.to_owned(), cmb.1, cmb.2.to_owned(), cmb.3, cmb.4)?;
            assert_eq!(&bed6, &bed12.clone().into());

            let bed8 = Bed8::new(
                cmb.0.to_owned(),
                cmb.1,
                cmb.2.to_owned(),
                cmb.3,
                cmb.4,
                cmb.5,
            )?;
            assert_eq!(&bed8, &bed12.clone().into());

            let bed9 = Bed9::new(
                cmb.0.to_owned(),
                cmb.1,
                cmb.2.to_owned(),
                cmb.3,
                cmb.4,
                cmb.5,
                cmb.6,
            )?;
            assert_eq!(&bed9, &bed12.clone().into());

            // Test from_bed(N-1) constructors
            assert_eq!(bed4, bed3.to_bed4(bed12.name().to_owned())?);
            assert_eq!(bed5, bed4.to_bed5(bed12.score())?);
            assert_eq!(bed6, bed5.to_bed6(bed12.orientation())?);
            assert_eq!(bed8, bed6.to_bed8(bed12.thick().to_owned())?);
            assert_eq!(bed9, bed8.to_bed9(bed12.rgb())?);
            assert_eq!(bed12, bed9.to_bed12(bed12.blocks().to_owned())?);
        }

        Ok(())
    }

    #[test]
    fn test_incorrect_bed_records() -> Result<()> {
        let mut template = Bed12::new(
            "1".to_owned(),
            Interval::new(10, 100)?,
            "name".to_owned(),
            500,
            Orientation::Forward,
            Interval::new(20, 50)?,
            (0, 0, 0),
            vec![Interval::new(0, 10)?, Interval::new(20, 90)?],
        )?;

        // Incorrect seqid
        for seqid in ["", &"A".repeat(256), " A"] {
            assert!(template.set_seqid(seqid.to_string()).is_err());
        }

        // Incorrect interval [impossible to construct -> skip]

        // Incorrect name
        for name in ["", &"A".repeat(256), "\0"] {
            assert!(template.set_name(name.to_string()).is_err());
        }

        // Incorrect score
        for score in ["1001", "10000"] {
            assert!(template.set_score(score.parse()?).is_err());
        }

        // Incorrect orientation [impossible to construct -> skip]

        // Incorrect thick coordinates
        for thick in [Interval::new(0, 9)?, Interval::new(51, 101)?] {
            assert!(template.set_thick(thick).is_err());
        }

        // Incorrect itemRgb [impossible to construct -> skip]

        // Incorrect blocks
        for blocks in [
            vec![],
            vec![Interval::new(0, 10)?, Interval::new(20, 100)?],
            vec![Interval::new(0, 89)?],
            vec![Interval::new(1, 90)?],
        ] {
            assert!(template.set_blocks(blocks).is_err());
        }

        Ok(())
    }
}
