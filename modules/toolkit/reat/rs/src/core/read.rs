use biobit_core_rs::loc::Strand;
#[cfg(test)]
use mockall::{mock, predicate::*};
use noodles::bam::Record;
use noodles::sam::alignment::record::cigar::Op;

#[allow(clippy::len_without_is_empty)]
pub trait SequencedRead {
    fn name(&self) -> &[u8];
    fn strand(&self) -> &Strand;

    fn seq(&self) -> Vec<u8>;
    fn base(&self, i: usize) -> u8 {
        self.seq()[i]
    }

    fn qual(&self) -> &[u8];
    fn base_qual(&self, i: usize) -> u8 {
        self.qual()[i]
    }

    fn is_first(&self) -> bool;

    fn len(&self) -> usize;
}

pub trait AlignedRead: SequencedRead {
    fn cigar(&self) -> &[Op];
    fn mapq(&self) -> u8;
    fn pos(&self) -> i64;
    fn contig(&self) -> &str;
    fn flags(&self) -> u16;
}

#[cfg(test)]
mock! {
    pub Read {}
    impl AlignedRead for Read {
        fn cigar(&self) -> &[Op];
        fn mapq(&self) -> u8;
        fn contig(&self) -> &str;
        fn pos(&self) -> i64;
        fn flags(&self) -> u16;
    }

    impl SequencedRead for Read {
        fn name(&self) -> &[u8];
        fn strand(&self) -> &Strand;

        fn seq(&self) -> Vec<u8>;
        fn base(&self, i: usize) -> u8;

        fn qual(&self) -> &[u8];
        fn base_qual(&self, i: usize) -> u8;

        fn is_first(&self) -> bool;
        fn len(&self) -> usize;
    }
}

impl SequencedRead for Record {
    #[inline]
    fn name(&self) -> &[u8] {
        Record::name(self).unwrap().iter().as_slice()
    }

    fn strand(&self) -> &Strand {
        todo!()
    }

    #[inline]
    fn seq(&self) -> Vec<u8> {
        // Record::sequence(self).sour
        todo!()
    }

    #[inline]
    fn qual(&self) -> &[u8] {
        // self.quality_scores().as_ref()
        todo!()
    }

    #[inline]
    fn is_first(&self) -> bool {
        self.flags().is_first_segment()
    }

    #[inline]
    fn len(&self) -> usize {
        self.seq().len()
    }
}

impl AlignedRead for Record {
    #[inline]
    fn cigar(&self) -> &[Op] {
        // self.cigar().iter().collect::<Vec<_>>()
        todo!()
    }

    #[inline]
    fn mapq(&self) -> u8 {
        self.mapping_quality().unwrap().into()
    }
    #[inline]
    fn pos(&self) -> i64 {
        self.alignment_start().unwrap().unwrap().get() as i64 - 1
    }

    fn contig(&self) -> &str {
        todo!()
    }

    #[inline]
    fn flags(&self) -> u16 {
        self.flags().bits()
    }
}
