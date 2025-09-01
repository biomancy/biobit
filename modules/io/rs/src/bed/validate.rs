use biobit_core_rs::loc::{Interval, IntervalOp, Orientation};
use eyre::{Result, ensure};

pub fn seqid(s: &str) -> Result<()> {
    ensure!(!s.is_empty(), "BED seqid can't be empty");
    ensure!(
        s.len() <= 255,
        "BED seqid can't be longer than 255 characters"
    );

    for c in s.chars() {
        ensure!(
            !c.is_ascii_whitespace(),
            "BED seqid can only contain non-whitespace ASCII characters, got: {}",
            s
        );
    }

    Ok(())
}

pub fn interval(_: &Interval<u64>) -> Result<()> {
    Ok(())
}

pub fn name(s: &str) -> Result<()> {
    ensure!(!s.is_empty(), "BED name can't be empty");
    ensure!(
        s.len() <= 255,
        "BED name can't be longer than 255 characters"
    );

    for c in s.chars() {
        ensure!(
            matches!(c, '\x20'..='\x7e'),
            "BED name can only contain printable ASCII characters"
        );
    }

    Ok(())
}

pub fn score(score: &u16) -> Result<()> {
    ensure!(*score <= 1000, "BED score must be between 0 and 1000");
    Ok(())
}

pub fn orientation(_: &Orientation) -> Result<()> {
    Ok(())
}

pub fn thick(interval: &Interval<u64>, thick: &Interval<u64>) -> Result<()> {
    ensure!(
        interval.envelops(thick),
        "BED thick interval must be within the main interval, but got {:?} and {:?}",
        interval,
        thick
    );
    Ok(())
}

pub fn rgb(_: &(u8, u8, u8)) -> Result<()> {
    Ok(())
}

pub fn blocks(interval: &Interval<u64>, blocks: &[Interval<u64>]) -> Result<()> {
    let count = blocks.len() as u64;
    ensure!(count > 0, "BED blockCount must be greater than 0");
    ensure!(
        count <= interval.len(),
        "BED blockCount must be less than or equal to the interval length"
    );

    // The first block must start at the beginning of the interval
    ensure!(
        blocks[0].start() == 0,
        "BED blocks must start at the beginning of the interval"
    );

    let mut position = 0;
    for block in blocks {
        ensure!(
            position <= block.start(),
            "BED blocks must be in ascending order"
        );
        position = block.end();
    }
    ensure!(
        position == interval.len(),
        "BED blocks must end at the end of the interval"
    );

    Ok(())
}
