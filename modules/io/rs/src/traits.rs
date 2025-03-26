use eyre::Result;

/// A trait for reading structured records. Modeled after the `Read` trait in the std.
/// TODO: Better documentation following the std::Read trait.
pub trait ReadRecord {
    /// The type of the records that will be read.
    type Record;

    /// Read a single record from the input into the provided buffer.
    /// Returns `true` if a record was read and `false` if the end of the input was reached.
    fn read_record(&mut self, into: &mut Self::Record) -> Result<bool>;

    /// Fill a buffer with records from the input. Returns the number of records read, which could
    /// be less than the length of the buffer or equals 0 if the end of the input is reached.
    fn read_records(&mut self, into: &mut [Self::Record]) -> Result<usize>;

    /// Read all records from the input into the provided buffer.
    fn read_to_end(&mut self, into: &mut Vec<Self::Record>) -> Result<usize>;
}

/// A trait for writing structured records. Modeled after the `Write` trait in the std.
/// TODO: Better documentation following the std::Write trait.
pub trait WriteRecord {
    type Record;

    /// Write a single record.
    fn write_record(&mut self, record: &Self::Record) -> Result<()>;

    /// Write a slice of records. Returns the number of records written, which could be less than
    /// the length of the slice if an error occurs.
    fn write_records(&mut self, records: &[Self::Record]) -> Result<()> {
        for record in records {
            self.write_record(record)?;
        }
        Ok(())
    }

    /// Flush the output.
    fn flush(&mut self) -> Result<()>;
}
