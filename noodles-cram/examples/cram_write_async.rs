//! Creates a new CRAM file.
//!
//! This writes a file definition, a header container built from a SAM header, one mapped record,
//! and one unmapped record to stdout.
//!
//! Verify the output by piping to `samtools view --no-PG --with-header`.

use noodles_core::Position;
use noodles_cram as cram;
use noodles_fasta as fasta;
use noodles_sam::{
    self as sam,
    header::record::value::{
        map::{Program, ReferenceSequence},
        Map,
    },
};
use tokio::io;

fn build_reference_sequences() -> Vec<fasta::Record> {
    use fasta::record::{Definition, Sequence};

    vec![
        fasta::Record::new(
            Definition::new("sq0", None),
            Sequence::from(b"TTCACCCA".to_vec()),
        ),
        fasta::Record::new(
            Definition::new("sq1", None),
            Sequence::from(b"GATCTTACTTTTT".to_vec()),
        ),
        fasta::Record::new(
            Definition::new("sq2", None),
            Sequence::from(b"GGCGCCCCGCTGTGCAAAAAT".to_vec()),
        ),
    ]
}

fn build_header(
    reference_sequence_records: &[fasta::Record],
) -> Result<sam::Header, Box<dyn std::error::Error>> {
    let mut builder = sam::Header::builder()
        .set_header(Default::default())
        .add_program(Map::<Program>::new("noodles-cram"))
        .add_comment("an example CRAM written by noodles-cram");

    for record in reference_sequence_records {
        let reference_sequence =
            Map::<ReferenceSequence>::new(record.name().parse()?, record.sequence().len())?;
        builder = builder.add_reference_sequence(reference_sequence);
    }

    Ok(builder.build())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let reference_sequences = build_reference_sequences();
    let header = build_header(&reference_sequences)?;

    let repository = fasta::Repository::new(reference_sequences);
    let mut writer = cram::r#async::writer::Builder::default()
        .set_reference_sequence_repository(repository)
        .build_with_writer(io::stdout());

    writer.write_file_definition().await?;
    writer.write_file_header(&header).await?;

    let record = sam::alignment::Record::builder()
        .set_flags(sam::record::Flags::empty())
        .set_reference_sequence_id(1)
        .set_alignment_start(Position::MIN)
        .set_cigar("4M".parse()?)
        .set_sequence("TTCA".parse()?)
        .set_quality_scores("NDLS".parse()?)
        .build();

    let cram_record = cram::Record::try_from_alignment_record(&header, &record)?;
    writer.write_record(&header, cram_record).await?;

    let record = sam::alignment::Record::default();
    let cram_record = cram::Record::try_from_alignment_record(&header, &record)?;
    writer.write_record(&header, cram_record).await?;

    writer.shutdown().await?;

    Ok(())
}
