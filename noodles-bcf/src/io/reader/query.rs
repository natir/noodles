use std::io::{self, Read, Seek};

use noodles_bgzf as bgzf;
use noodles_core::region::Interval;
use noodles_csi::{self as csi, binning_index::index::reference_sequence::bin::Chunk};
use noodles_vcf::{self as vcf, variant::Record as _};

use super::read_record;
use crate::Record;

/// An iterator over records of a BCF reader that intersects a given region.
///
/// This is created by calling [`super::Reader::query`].
pub struct Query<'r, 'h, R>
where
    R: Read + Seek,
{
    reader: csi::io::Query<'r, R>,
    header: &'h vcf::Header,
    chromosome_id: usize,
    interval: Interval,
    record: Record,
}

impl<'r, 'h, R> Query<'r, 'h, R>
where
    R: Read + Seek,
{
    pub(super) fn new(
        reader: &'r mut bgzf::Reader<R>,
        header: &'h vcf::Header,
        chunks: Vec<Chunk>,
        chromosome_id: usize,
        interval: Interval,
    ) -> Self {
        Self {
            reader: csi::io::Query::new(reader, chunks),
            header,
            chromosome_id,
            interval,
            record: Record::default(),
        }
    }

    fn next_record(&mut self) -> io::Result<Option<Record>> {
        read_record(&mut self.reader, &mut self.record).map(|n| match n {
            0 => None,
            _ => Some(self.record.clone()),
        })
    }
}

impl<'r, 'h, R> Iterator for Query<'r, 'h, R>
where
    R: Read + Seek,
{
    type Item = io::Result<Record>;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            match self.next_record() {
                Ok(Some(record)) => {
                    match intersects(self.header, &record, self.chromosome_id, self.interval) {
                        Ok(true) => return Some(Ok(record)),
                        Ok(false) => {}
                        Err(e) => return Some(Err(e)),
                    }
                }
                Ok(None) => return None,
                Err(e) => return Some(Err(e)),
            }
        }
    }
}

fn intersects(
    header: &vcf::Header,
    record: &Record,
    chromosome_id: usize,
    region_interval: Interval,
) -> io::Result<bool> {
    let chromosome = record.reference_sequence_name(header.string_maps())?;

    let id = header
        .string_maps()
        .contigs()
        .get_index_of(chromosome)
        .ok_or_else(|| {
            io::Error::new(
                io::ErrorKind::InvalidInput,
                format!("chromosome does not exist in contigs: {chromosome}"),
            )
        })?;

    let Some(start) = record.variant_start().transpose()? else {
        return Ok(false);
    };

    let end = record.variant_end(header)?;
    let record_interval = Interval::from(start..=end);

    Ok(id == chromosome_id && record_interval.intersects(region_interval))
}
