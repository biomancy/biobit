use biobit_core_rs::loc::{Interval, IntervalOp};
use biobit_core_rs::num::PrimInt;
use derive_getters::{Dissolve, Getters};
use eyre::{Context, Result, ensure, eyre};
use impl_tools::autoimpl;
use std::collections::HashMap;
use std::io::{BufRead, Read, Seek};

const READ_CHUNK_SIZE: usize = 512 * 1024; // 512 KB chunks for sequential reads

/// An indexed FASTA reader that can fetch sequences by reference sequence ID and interval.
#[autoimpl(for<T: trait + ?Sized> Box<T>)]
pub trait IndexedReaderMutOp {
    /// Fetch the length of all sequences in the indexed FASTA file(s).
    fn lengths(&self) -> HashMap<String, u64>;

    /// Fetch the sequence for the given reference sequence ID and interval.
    fn fetch(&mut self, seqid: &str, interval: Interval<u64>, buffer: &mut Vec<u8>) -> Result<()>;

    /// Fetch the full reference sequence with the given ID.
    fn fetch_full_seq(&mut self, seqid: &str, buffer: &mut Vec<u8>) -> Result<()>;
}

#[derive(Debug, Clone, PartialEq, Eq, Dissolve, Getters)]
pub struct IndexedReader<R> {
    readers: Vec<R>, // FASTA file readers

    reader_idx: Vec<usize>, // IDs of the reference sequences as they appear in FASTA files
    lengths: Vec<u64>,      // Total length of each reference sequence, in bases
    offsets: Vec<u64>,      // Offset in the FASTA file of the first base of each reference sequence
    bases_per_line: Vec<u64>, // Number of bases per line for each reference sequence
    bytes_per_line: Vec<u64>, // Number of bytes per line for each reference sequence (including line ending character[s])

    index: HashMap<String, usize>, // Map from reference sequence ID to its index in the vectors above
}

pub(crate) trait IndexedRead: Read + Seek + Send + Sync + 'static {}
impl<T: Read + Seek + Send + Sync + 'static> IndexedRead for T {}

impl<R: Read + Seek> IndexedReader<R> {
    /// Create a new indexed reader by parsing given FASTA/Index pairs.
    pub fn new<I: BufRead>(mut indexed: Vec<(R, I)>) -> Result<IndexedReader<R>> {
        let mut readers = Vec::with_capacity(indexed.len());

        let mut seqids = HashMap::new();
        let mut reader_idx = Vec::new();
        let mut lengths = Vec::new();
        let mut offsets = Vec::new();
        let mut bases_per_line = Vec::new();
        let mut bytes_per_line = Vec::new();

        let mut buffer = String::new();
        for (reader, mut index) in indexed.drain(..) {
            let ridx = readers.len();
            readers.push(reader);

            while index.read_line(&mut buffer)? > 0 {
                let err = || eyre!("Invalid FASTA index line: {}", buffer);
                let mut parts = buffer.split('\t');

                let id = parts.next().ok_or_else(err)?;
                ensure!(
                    !id.is_empty(),
                    "Reference sequence ID cannot be empty, line: {}",
                    buffer
                );
                match seqids.entry(id.to_string()) {
                    std::collections::hash_map::Entry::Vacant(e) => {
                        e.insert(lengths.len());
                    }
                    std::collections::hash_map::Entry::Occupied(_) => {
                        return Err(eyre!(
                            "Duplicate reference sequence ID in the FASTA index: {}",
                            id
                        ));
                    }
                }

                let length = parts
                    .next()
                    .ok_or_else(err)?
                    .parse::<u64>()
                    .wrap_err_with(err)?;
                ensure!(
                    length > 0,
                    "Length of the reference sequence must be greater than zero, line: {}",
                    buffer
                );
                lengths.push(length);

                let offset = parts
                    .next()
                    .ok_or_else(err)?
                    .parse::<u64>()
                    .wrap_err_with(err)?;
                ensure!(
                    offset > 0,
                    "Offset of the reference sequence must be greater than zero, line: {}",
                    buffer
                );
                offsets.push(offset);

                let _bases_per_line = parts
                    .next()
                    .ok_or_else(err)?
                    .parse::<u64>()
                    .wrap_err_with(err)?;
                ensure!(
                    _bases_per_line > 0,
                    "Bases per line must be greater than zero, line: {}",
                    buffer
                );
                bases_per_line.push(_bases_per_line);

                let _bytes_per_line = parts
                    .next()
                    .ok_or_else(err)?
                    .trim_end_matches(&['\r', '\n'] as &[char])
                    .parse::<u64>()
                    .wrap_err_with(err)?;
                ensure!(
                    _bytes_per_line >= _bases_per_line,
                    "Bytes per line must be at least bases per line, line: {}",
                    buffer
                );
                bytes_per_line.push(_bytes_per_line);

                ensure!(
                    parts.next().is_none(),
                    "Extra fields in the FASTA index, line: {}",
                    buffer
                );

                reader_idx.push(ridx);

                buffer.clear();
            }
        }

        Ok(Self {
            readers,
            reader_idx,
            lengths,
            offsets,
            bases_per_line,
            bytes_per_line,
            index: seqids,
        })
    }

    #[inline(always)]
    fn sanitize<Iv, Idx>(&self, seqid: &str, interval: &Iv) -> Result<(usize, u64, u64)>
    where
        Iv: IntervalOp<Idx = Idx>,
        Idx: PrimInt,
    {
        // Find the index of the reference sequence
        let index = self
            .index
            .get(seqid)
            .ok_or_else(|| eyre!("Reference sequence ID not found in the index: {}", seqid))?;

        // Get required coordinates in sequence space
        let start = interval
            .start()
            .to_u64()
            .ok_or_else(|| eyre!("Invalid start coordinate: {:?}", interval.start()))?;
        let end = interval
            .end()
            .to_u64()
            .ok_or_else(|| eyre!("Invalid end coordinate: {:?}", interval.end()))?;

        // Validate the coordinates
        ensure!(start < end, "Invalid interval for {seqid}: {start}..{end}");
        let length = self.lengths[*index];
        ensure!(
            start < length,
            "Start coordinate for {} is out of bounds: {} >= {}",
            seqid,
            start,
            length
        );
        ensure!(
            end <= length,
            "End coordinate for {} is out of bounds: {} > {}",
            seqid,
            end,
            length
        );

        Ok((*index, start, end))
    }

    /// Fetch the sequence for the given reference sequence ID and zero-based,
    /// half-open interval, replacing the contents of `buffer`.
    pub fn fetch_interval<Iv, Idx>(
        &mut self,
        seqid: &str,
        interval: &Iv,
        buffer: &mut Vec<u8>,
    ) -> Result<()>
    where
        Iv: IntervalOp<Idx = Idx>,
        Idx: PrimInt,
    {
        let (index, start, end) = self.sanitize(seqid, interval)?;

        let ridx = self.reader_idx[index];
        let reader = &mut self.readers[ridx];
        let offset = self.offsets[index];
        let bases_per_line = self.bases_per_line[index];
        let bytes_per_line = self.bytes_per_line[index];
        let endline_bytes = bytes_per_line - bases_per_line;

        let first_line = start / bases_per_line;
        let last_line = (end - 1) / bases_per_line;
        let length = usize::try_from(end - start)?;

        // Include only line endings between requested bases, never the one after
        // the final base. This also supports files without a final newline.
        let raw_length = (last_line - first_line)
            .checked_mul(endline_bytes)
            .and_then(|gaps| (end - start).checked_add(gaps))
            .and_then(|bytes| usize::try_from(bytes).ok())
            .ok_or_else(|| {
                eyre!(
                    "FASTA read buffer size overflow. This should not normally happen and likely indicates a logical bug."
                )
            })?;

        let start_byte = first_line
            .checked_mul(bytes_per_line)
            .and_then(|bytes| offset.checked_add(bytes))
            .and_then(|position| position.checked_add(start % bases_per_line))
            .ok_or_else(|| eyre!("FASTA byte offset overflow for {seqid}; check the .fai index"))?;

        buffer.clear();
        buffer.try_reserve(length)?;
        reader.seek(std::io::SeekFrom::Start(start_byte))?;

        // Read into the space still needed for the final sequence, then compact
        // only the new chunk. Near the end, shrink reads to the remaining capacity
        // to avoid growing the allocation just to accommodate temporary line endings.
        let mut remaining = raw_length;
        let mut line_position = start % bases_per_line;
        while remaining > 0 {
            // 1. Read into the buffer, up to the remaining capacity or the chunk size.
            let chunk_start = buffer.len();
            let chunk_length = remaining
                .min(READ_CHUNK_SIZE)
                .min(buffer.capacity() - chunk_start);
            let read = reader
                .by_ref()
                .take(chunk_length as u64)
                .read_to_end(buffer)?;
            if read != chunk_length {
                return Err(std::io::Error::new(
                    std::io::ErrorKind::UnexpectedEof,
                    "truncated FASTA sequence",
                )
                .into());
            }

            // 2. Compact the chunk in the buffer, one line at a time.
            let chunk_end = buffer.len();
            let mut source = chunk_start;
            let mut destination = chunk_start;
            while source < chunk_end {
                let count = bases_per_line
                    .saturating_sub(line_position)
                    .min((chunk_end - source) as u64) as usize;
                if source != destination {
                    buffer.copy_within(source..source + count, destination);
                }
                source += count;
                destination += count;
                line_position += count as u64;

                // A chunk may stop anywhere, including between CR and LF.
                let skip = (bytes_per_line - line_position).min((chunk_end - source) as u64);
                source += skip as usize;
                line_position += skip;
                if line_position == bytes_per_line {
                    line_position = 0;
                }
            }
            buffer.truncate(destination);
            remaining -= chunk_length;
        }
        debug_assert_eq!(buffer.len(), length);

        Ok(())
    }

    /// Fetch the full sequence for the given reference sequence ID.
    pub fn fetch_full_seq(&mut self, seqid: &str, buffer: &mut Vec<u8>) -> Result<()> {
        let index = self
            .index
            .get(seqid)
            .ok_or_else(|| eyre!("Reference sequence ID not found in the index: {}", seqid))?;

        let interval = Interval::new(0, self.lengths[*index])?;
        self.fetch_interval(seqid, &interval, buffer)
    }
}

impl<R: Read + Seek> IndexedReaderMutOp for IndexedReader<R> {
    fn lengths(&self) -> HashMap<String, u64> {
        let mut lengths = HashMap::new();
        for (seqid, &index) in self.index.iter() {
            lengths.insert(seqid.clone(), self.lengths[index]);
        }
        lengths
    }

    fn fetch(&mut self, seqid: &str, interval: Interval<u64>, buffer: &mut Vec<u8>) -> Result<()> {
        Self::fetch_interval(self, seqid, &interval, buffer)
    }

    fn fetch_full_seq(&mut self, seqid: &str, buffer: &mut Vec<u8>) -> Result<()> {
        Self::fetch_full_seq(self, seqid, buffer)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::fasta::IndexedSources;
    use std::io::{Cursor, ErrorKind};
    use std::path::PathBuf;
    use substratum_compress::Decoder;

    #[test]
    fn fetches_intervals_across_line_endings() -> Result<()> {
        for (fasta, sequence, line_bases, line_width) in [
            (">id\nACGT\nTGCA\nACGT\n", "ACGTTGCAACGT", 4, 5),
            (">id\nACGT\nTGCA\nACGT", "ACGTTGCAACGT", 4, 5),
            (">id\nACGT\r\nTGCA\r\nACGT\r\n", "ACGTTGCAACGT", 4, 6),
            (">id\nACGT\r\nTGCA\r\nACGT", "ACGTTGCAACGT", 4, 6),
            (">id\nACGT\nTGCA\nAC\n", "ACGTTGCAAC", 4, 5),
            (">id\nACGT\nTGCA\nAC", "ACGTTGCAAC", 4, 5),
            (">id\nACGT\r\nTGCA\r\nAC\r\n", "ACGTTGCAAC", 4, 6),
            (">id\nACGT\r\nTGCA\r\nAC", "ACGTTGCAAC", 4, 6),
            (">id\nA\nC\nG\nT", "ACGT", 1, 2),
            (">id\nA\r\nC\r\nG\r\nT", "ACGT", 1, 3),
            (">id\nACGT", "ACGT", 4, 4),
            (">id\nACGT", "ACGT", 4, 5),
        ] {
            let sequence = sequence.as_bytes();
            let index = format!("id\t{}\t4\t{line_bases}\t{line_width}\n", sequence.len());
            let mut reader = IndexedReader::new(vec![(Cursor::new(fasta), Cursor::new(index))])?;
            // Reuse a buffer while the requested interval length grows and shrinks.
            let mut buffer = vec![b'X'; 8];
            for start in 0..sequence.len() {
                for end in start + 1..=sequence.len() {
                    let interval = Interval::new(start as u64, end as u64)?;
                    reader.fetch_interval("id", &interval, &mut buffer)?;
                    assert_eq!(buffer, sequence[start..end]);
                }
            }
            reader.fetch_full_seq("id", &mut buffer)?;
            assert_eq!(buffer, sequence);
        }
        Ok(())
    }

    #[test]
    fn truncated_sequences_return_unexpected_eof() -> Result<()> {
        for (fasta, end) in [
            (">id\nAC", 3),              // In a single line.
            (">id\nAC", 12),             // In the first line.
            (">id\nACGT\nTG", 12),       // In a middle line.
            (">id\nACGT\nTGCA\nAC", 12), // In the last line.
            (">id\nACGT", 12),           // Before a line ending.
            (">id\n", 12),               // Before any bases.
        ] {
            let mut reader =
                IndexedReader::new(vec![(Cursor::new(fasta), Cursor::new("id\t12\t4\t4\t5\n"))])?;
            let error = reader
                .fetch_interval("id", &Interval::new(0, end)?, &mut Vec::new())
                .unwrap_err();
            assert_eq!(
                error.downcast_ref::<std::io::Error>().unwrap().kind(),
                ErrorKind::UnexpectedEof,
                "FASTA: {fasta:?}"
            );
        }
        Ok(())
    }

    #[test]
    fn invalid_line_widths_are_rejected() {
        for (bases, bytes) in [(0, 5), (4, 0), (4, 3)] {
            let index = format!("id\t4\t4\t{bases}\t{bytes}\n");
            assert!(
                IndexedReader::new(vec![(Cursor::new(">id\nACGT\n"), Cursor::new(index))]).is_err()
            );
        }
    }

    #[test]
    fn test_indexed_fa() -> Result<()> {
        fn test(mut reader: impl IndexedReaderMutOp) -> Result<()> {
            // Check that we can fetch full sequences
            let sequences = vec![
                (
                    "sp|O95786|RIGI_HUMAN",
                    "MTTEQRRSLQAFQDYIRKTLDPTYILSYMAPWFREEEVQYIQAEKNNKGPMEAATLFLKF\
                     LLELQEEGWFRGFLDALDHAGYSGLYEAIESWDFKKIEKLEEYRLLLKRLQPEFKTRIIP\
                     TDIISDLSECLINQECEEILQICSTKGMMAGAEKLVECLLRSDKENWPKTLKLALEKERN\
                     KFSELWIVEKGIKDVETEDLEDKMETSDIQIFYQEDPECQNLSENSCPPSEVSDTNLYSP\
                     FKPRNYQLELALPAMKGKNTIICAPTGCGKTFVSLLICEHHLKKFPQGQKGKVVFFANQI\
                     PVYEQQKSVFSKYFERHGYRVTGISGATAENVPVEQIVENNDIIILTPQILVNNLKKGTI\
                     PSLSIFTLMIFDECHNTSKQHPYNMIMFNYLDQKLGGSSGPLPQVIGLTASVGVGDAKNT\
                     DEALDYICKLCASLDASVIATVKHNLEELEQVVYKPQKFFRKVESRISDKFKYIIAQLMR\
                     DTESLAKRICKDLENLSQIQNREFGTQKYEQWIVTVQKACMVFQMPDKDEESRICKALFL\
                     YTSHLRKYNDALIISEHARMKDALDYLKDFFSNVRAAGFDEIEQDLTQRFEEKLQELESV\
                     SRDPSNENPKLEDLCFILQEEYHLNPETITILFVKTRALVDALKNWIEGNPKLSFLKPGI\
                     LTGRGKTNQNTGMTLPAQKCILDAFKASGDHNILIATSVADEGIDIAQCNLVILYEYVGN\
                     VIKMIQTRGRGRARGSKCFLLTSNAGVIEKEQINMYKEKMMNDSILRLQTWDEAVFREKI\
                     LHIQTHEKFIRDSQEKPKPVPDKENKKLLCRKCKALACYTADVRVIEECHYTVLGDAFKE\
                     CFVSRPHPKPKQFSSFEKRAKIFCARQNCSHDWGIHVKYKTFEIPVIKIESFVVEDIATG\
                     VQTLYSKWKDFHFEKIPFDPAEMSK",
                ),
                (
                    "sp|P55265|DSRAD_HUMAN",
                    "MNPRQGYSLSGYYTHPFQGYEHRQLRYQQPGPGSSPSSFLLKQIEFLKGQLPEAPVIGKQ\
                     TPSLPPSLPGLRPRFPVLLASSTRGRQVDIRGVPRGVHLRSQGLQRGFQHPSPRGRSLPQ\
                     RGVDCLSSHFQELSIYQDQEQRILKFLEELGEGKATTAHDLSGKLGTPKKEINRVLYSLA\
                     KKGKLQKEAGTPPLWKIAVSTQAWNQHSGVVRPDGHSQGAPNSDPSLEPEDRNSTSVSED\
                     LLEPFIAVSAQAWNQHSGVVRPDSHSQGSPNSDPGLEPEDSNSTSALEDPLEFLDMAEIK\
                     EKICDYLFNVSDSSALNLAKNIGLTKARDINAVLIDMERQGDVYRQGTTPPIWHLTDKKR\
                     ERMQIKRNTNSVPETAPAAIPETKRNAEFLTCNIPTSNASNNMVTTEKVENGQEPVIKLE\
                     NRQEARPEPARLKPPVHYNGPSKAGYVDFENGQWATDDIPDDLNSIRAAPGEFRAIMEMP\
                     SFYSHGLPRCSPYKKLTECQLKNPISGLLEYAQFASQTCEFNMIEQSGPPHEPRFKFQVV\
                     INGREFPPAEAGSKKVAKQDAAMKAMTILLEEAKAKDSGKSEESSHYSTEKESEKTAESQ\
                     TPTPSATSFFSGKSPVTTLLECMHKLGNSCEFRLLSKEGPAHEPKFQYCVAVGAQTFPSV\
                     SAPSKKVAKQMAAEEAMKALHGEATNSMASDNQPEGMISESLDNLESMMPNKVRKIGELV\
                     RYLNTNPVGGLLEYARSHGFAAEFKLVDQSGPPHEPKFVYQAKVGGRWFPAVCAHSKKQG\
                     KQEAADAALRVLIGENEKAERMGFTEVTPVTGASLRRTMLLLSRSPEAQPKTLPLTGSTF\
                     HDQIAMLSHRCFNTLTNSFQPSLLGRKILAAIIMKKDSEDMGVVVSLGTGNRCVKGDSLS\
                     LKGETVNDCHAEIISRRGFIRFLYSELMKYNSQTAKDSIFEPAKGGEKLQIKKTVSFHLY\
                     ISTAPCGDGALFDKSCSDRAMESTESRHYPVFENPKQGKLRTKVENGEGTIPVESSDIVP\
                     TWDGIRLGERLRTMSCSDKILRWNVLGLQGALLTHFLQPIYLKSVTLGYLFSQGHLTRAI\
                     CCRVTRDGSAFEDGLRHPFIVNHPKVGRVSIYDSKRQSGKTKETSVNWCLADGYDLEILD\
                     GTRGTVDGPRNELSRVSKKNIFLLFKKLCSFRYRRDLLRLSYGEAKKAARDYETAKNYFK\
                     KGLKDMGYGNWISKPQEEKNFYLCPV",
                ),
                (
                    "sp|Q7Z434|MAVS_HUMAN",
                    "MPFAEDKTYKYICRNFSNFCNVDVVEILPYLPCLTARDQDRLRATCTLSGNRDTLWHLFN\
                     TLQRRPGWVEYFIAALRGCELVDLADEVASVYQSYQPRTSDRPPDPLEPPSLPAERPGPP\
                     TPAAAHSIPYNSCREKEPSYPMPVQETQAPESPGENSEQALQTLSPRAIPRNPDGGPLES\
                     SSDLAALSPLTSSGHQEQDTELGSTHTAGATSSLTPSRGPVSPSVSFQPLARSTPRASRL\
                     PGPTGSVVSTGTSFSSSSPGLASAGAAEGKQGAESDQAEPIICSSGAEAPANSLPSKVPT\
                     TLMPVNTVALKVPANPASVSTVPSKLPTSSKPPGAVPSNALTNPAPSKLPINSTRAGMVP\
                     SKVPTSMVLTKVSASTVPTDGSSRNEETPAAPTPAGATGGSSAWLDSSSENRGLGSELSK\
                     PGVLASQVDSPFSGCFEDLAISASTSLGMGPCHGPEENEYKSEGTFGIHVAENPSIQLLE\
                     GNPGPPADPDGGPRPQADRKFQEREVPCHRPSPGALWLQVAVTGVLVVTLLVVLYRRRLH",
                ),
                (
                    "sp|Q8NB16|MLKL_HUMAN",
                    "MENLKHIITLGQVIHKRCEEMKYCKKQCRRLGHRVLGLIKPLEMLQDQGKRSVPSEKLTT\
                     AMNRFKAALEEANGEIEKFSNRSNICRFLTASQDKILFKDVNRKLSDVWKELSLLLQVEQ\
                     RMPVSPISQGASWAQEDQQDADEDRRAFQMLRRDNEKIEASLRRLEINMKEIKETLRQYL\
                     PPKCMQEIPQEQIKEIKKEQLSGSPWILLRENEVSTLYKGEYHRAPVAIKVFKKLQAGSI\
                     AIVRQTFNKEIKTMKKFESPNILRIFGICIDETVTPPQFSIVMEYCELGTLRELLDREKD\
                     LTLGKRMVLVLGAARGLYRLHHSEAPELHGKIRSSNFLVTQGYQVKLAGFELRKTQTSMS\
                     LGTTREKTDRVKSTAYLSPQELEDVFYQYDVKSEIYSFGIVLWEIATGDIPFQGCNSEKI\
                     RKLVAVKRQQEPLGEDCPSELREIIDECRAHDPSVRPSVDEILKKLSTFSK",
                ),
                (
                    "sp|Q96C10|DHX58_HUMAN",
                    "MELRSYQWEVIMPALEGKNIIIWLPTGAGKTRAAAYVAKRHLETVDGAKVVVLVNRVHLV\
                     TQHGEEFRRMLDGRWTVTTLSGDMGPRAGFGHLARCHDLLICTAELLQMALTSPEEEEHV\
                     ELTVFSLIVVDECHHTHKDTVYNVIMSQYLELKLQRAQPLPQVLGLTASPGTGGASKLDG\
                     AINHVLQLCANLDTWCIMSPQNCCPQLQEHSQQPCKQYNLCHRRSQDPFGDLLKKLMDQI\
                     HDHLEMPELSRKFGTQMYEQQVVKLSEAAALAGLQEQRVYALHLRRYNDALLIHDTVRAV\
                     DALAALQDFYHREHVTKTQILCAERRLLALFDDRKNELAHLATHGPENPKLEMLEKILQR\
                     QFSSSNSPRGIIFTRTRQSAHSLLLWLQQQQGLQTVDIRAQLLIGAGNSSQSTHMTQRDQ\
                     QEVIQKFQDGTLNLLVATSVAEEGLDIPHCNVVVRYGLLTNEISMVQARGRARADQSVYA\
                     FVATEGSRELKRELINEALETLMEQAVAAVQKMDQAEYQAKIRDLQQAALTKRAAQAAQR\
                     ENQRQQFPVEHVQLLCINCMVAVGHGSDLRKVEGTHHVNVNPNFSNYYNVSRDPVVINKV\
                     FKDWKPGGVISCRNCGEVWGLQMIYKSVKLPVLKVRSMLLETPQGRIQAKKWSRVPFSVP\
                     DFDFLQHCAENLSDLSLD",
                ),
                (
                    "sp|Q9BYX4|IFIH1_HUMAN",
                    "MSNGYSTDENFRYLISCFRARVKMYIQVEPVLDYLTFLPAEVKEQIQRTVATSGNMQAVE\
                     LLLSTLEKGVWHLGWTREFVEALRRTGSPLAARYMNPELTDLPSPSFENAHDEYLQLLNL\
                     LQPTLVDKLLVRDVLDKCMEEELLTIEDRNRIAAAENNGNESGVRELLKRIVQKENWFSA\
                     FLNVLRQTGNNELVQELTGSDCSESNAEIENLSQVDGPQVEEQLLSTTVQPNLEKEVWGM\
                     ENNSSESSFADSSVVSESDTSLAEGSVSCLDESLGHNSNMGSDSGTMGSDSDEENVAARA\
                     SPEPELQLRPYQMEVAQPALEGKNIIICLPTGSGKTRVAVYIAKDHLDKKKKASEPGKVI\
                     VLVNKVLLVEQLFRKEFQPFLKKWYRVIGLSGDTQLKISFPEVVKSCDIIISTAQILENS\
                     LLNLENGEDAGVQLSDFSLIIIDECHHTNKEAVYNNIMRHYLMQKLKNNRLKKENKPVIP\
                     LPQILGLTASPGVGGATKQAKAEEHILKLCANLDAFTIKTVKENLDQLKNQIQEPCKKFA\
                     IADATREDPFKEKLLEIMTRIQTYCQMSPMSDFGTQPYEQWAIQMEKKAAKEGNRKERVC\
                     AEHLRKYNEALQINDTIRMIDAYTHLETFYNEEKDKKFAVIEDDSDEGGDDEYCDGDEDE\
                     DDLKKPLKLDETDRFLMTLFFENNKMLKRLAENPEYENEKLTKLRNTIMEQYTRTEESAR\
                     GIIFTKTRQSAYALSQWITENEKFAEVGVKAHHLIGAGHSSEFKPMTQNEQKEVISKFRT\
                     GKINLLIATTVAEEGLDIKECNIVIRYGLVTNEIAMVQARGRARADESTYVLVAHSGSGV\
                     IEHETVNDFREKMMYKAIHCVQNMKPEEYAHKILELQMQSIMEKKMKTKRNIAKHYKNNP\
                     SLITFLCKNCSVLACSGEDIHVIEKMHHVNMTPEFKELYIVRENKALQKKCADYQINGEI\
                     ICKCGQAWGTMMVHKGLDLPCLKIRNFVVVFKNNSTKKQYKKWVELPITFPNLDYSECCL\
                     FSDED",
                ),
                (
                    "sp|Q9H171|ZBP1_HUMAN",
                    "MAQAPADPGREGHLEQRILQVLTEAGSPVKLAQLVKECQAPKRELNQVLYRMKKELKVSL\
                     TSPATWCLGGTDPEGEGPAELALSSPAERPQQHAATIPETPGPQFSQQREEDIYRFLKDN\
                     GPQRALVIAQALGMRTAKDVNRDLYRMKSRHLLDMDEQSKAWTIYRPEDSGRRAKSASII\
                     YQHNPINMICQNGPNSWISIANSEAIQIGHGNIITRQTVSREDGSAGPRHLPSMAPGDSS\
                     TWGTLVDPWGPQDIHMEQSILRRVQLGHSNEMRLHGVPSEGPAHIPPGSPPVSATAAGPE\
                     ASFEARIPSPGTHPEGEAAQRIHMKSCFLEDATIGNSNKMSISPGVAGPGGVAGSGEGEP\
                     GEDAGRRPADTQSRSHFPRDIGQPITPSHSKLTPKLETMTLGNRSHKAAEGSHYVDEASH\
                     EGSWWGGGI",
                ),
                (
                    "sp|Q9Y572|RIPK3_HUMAN",
                    "MSCVKLWPSGAPAPLVSIEELENQELVGKGGFGTVFRAQHRKWGYDVAVKIVNSKAISRE\
                     VKAMASLDNEFVLRLEGVIEKVNWDQDPKPALVTKFMENGSLSGLLQSQCPRPWPLLCRL\
                     LKEVVLGMFYLHDQNPVLLHRDLKPSNVLLDPELHVKLADFGLSTFQGGSQSGTGSGEPG\
                     GTLGYLAPELFVNVNRKASTASDVYSFGILMWAVLAGREVELPTEPSLVYEAVCNRQNRP\
                     SLAELPQAGPETPGLEGLKELMQLCWSSEPKDRPSFQECLPKTDEVFQMVENNMNAAVST\
                     VKDFLSQLRSSNRRFSIPESGQGGTEMDGFRRTIENQHSRNDVMVSEWLNKLNLEEPPSS\
                     VPKKCPSLTKRSRAQEEQVPQAWTAGTSSDSMAQPPQTPETSTFRNQMPSPTSTGTPSPG\
                     PRGNQGAERQGMNWSCRTPEPNPVTGRPLVNIYNCSGVQVGDNNYLTMQQTTALPTWGLA\
                     PSGKGRGLQHPPPVGSQEGPKDPEAWSRPQGWYNHSGK",
                ),
            ];

            let buffer = &mut Vec::new();
            let seqlens = reader.lengths();
            for (id, seq) in sequences.iter() {
                buffer.clear();
                reader.fetch_full_seq(id, buffer)?;

                let fetched = String::from_utf8(buffer.clone())?;
                assert_eq!(fetched, *seq, "ID: {}", id);

                let length = seqlens.get(*id).ok_or_else(|| eyre!("ID not found"))?;
                assert_eq!(*length as usize, seq.len(), "ID: {}", id);
            }
            let sequences: HashMap<_, _> = sequences.into_iter().collect();

            // Query individual intervals from the sequence
            for (seqid, intervals) in [
                (
                    "sp|Q7Z434|MAVS_HUMAN",
                    vec![
                        0..24,
                        0..100,
                        35..120,
                        59..60,
                        60..61,
                        61..62,
                        119..120,
                        120..121,
                        121..122,
                    ],
                ),
                (
                    "sp|Q9Y572|RIPK3_HUMAN",
                    vec![1..375, 60..180, 1..517, 180..518],
                ),
            ] {
                for interval in intervals {
                    buffer.clear();
                    reader.fetch(seqid, Interval::try_from(interval.clone())?, buffer)?;
                    let fetched = String::from_utf8(buffer.clone())?;

                    let expected =
                        &sequences[seqid][interval.start as usize..interval.end as usize];
                    assert_eq!(fetched, expected, "ID: {}, Interval: {:?}", seqid, interval);
                }
            }

            // Query invalid intervals
            for (seqid, intervals) in [
                ("sp|Q7Z434|MAVS_HUMAN", vec![0..1000, 1000..10000, 987..988]),
                ("sp|Q9Y572|RIPK3_HUMAN", vec![1..1000]),
            ] {
                for interval in intervals {
                    let interval = Interval::try_from(interval.clone())?;
                    let result = reader.fetch(seqid, interval, buffer);
                    assert!(result.is_err(), "ID: {}, Interval: {:?}", seqid, interval);
                }
            }

            Ok(())
        }

        for path in ["indexed.fa", "indexed.fa.bgz"] {
            let path = PathBuf::from(env!("BIOBIT_RESOURCES"))
                .join("fasta")
                .join(path);
            test(
                IndexedSources::from_path(
                    &path,
                    Decoder::from_path(&path, crate::fasta::EXTENSIONS).unwrap(),
                )
                .open()?,
            )?
        }

        Ok(())
    }

    #[test]
    fn test_multiple_indexed_fa() -> Result<()> {
        let mut indexed = Vec::new();
        for path in ["indexed.fa", "CHM13v2.M-21-22.fa.bgz"] {
            let path = PathBuf::from(env!("BIOBIT_RESOURCES"))
                .join("fasta")
                .join(path);
            indexed.push((
                path.clone(),
                Decoder::from_path(&path, crate::fasta::EXTENSIONS).unwrap(),
            ));
        }

        let mut reader = IndexedSources::from_paths(&indexed).open()?;
        for (seqid, interval, expected) in [
            ("sp|O95786|RIGI_HUMAN", 0..24, "MTTEQRRSLQAFQDYIRKTLDPTY"),
            ("sp|Q9Y572|RIPK3_HUMAN", 510..518, "GWYNHSGK"),
            ("chr21", 23997387..23997416, "tcctcctgctgctgctgcgcTACCTGGTG"),
            ("chrM", 2830..2849, "CAAAGGCCCCAACGTTGTA"),
        ] {
            let mut buffer = Vec::new();
            reader.fetch(seqid, Interval::try_from(interval.clone())?, &mut buffer)?;
            let fetched = String::from_utf8(buffer.clone())?;
            assert_eq!(fetched, expected, "ID: {}, Interval: {:?}", seqid, interval);
        }
        Ok(())
    }
}
