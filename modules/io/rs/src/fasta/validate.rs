use eyre::{ensure, Result};

pub fn id(id: &str) -> Result<()> {
    ensure!(!id.is_empty(), "FASTA ID cannot be empty");
    ensure!(
        !id.contains(&['\n', '\r'] as &[char]),
        "Newline characters are not allowed in the FASTA ID: {id}"
    );
    Ok(())
}

pub fn seq(seq: &[u8]) -> Result<()> {
    ensure!(!seq.is_empty(), "FASTA sequence cannot be empty");
    for (i, &x) in seq.iter().enumerate() {
        ensure!(
            x.is_ascii_alphabetic(),
            "Non-alphabetic character at index {i} = {x:?}"
        );
    }
    Ok(())
}
