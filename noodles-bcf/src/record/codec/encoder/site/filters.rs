use std::io::{self, Write};

use noodles_vcf::{self as vcf, variant::record::Filters};

pub(super) fn write_filters<W, F>(
    writer: &mut W,
    header: &vcf::Header,
    filters: F,
) -> io::Result<()>
where
    W: Write,
    F: Filters,
{
    use crate::record::codec::encoder::string_map::write_string_map_indices;

    let indices: Vec<_> = filters
        .iter(header)
        .map(|result| {
            let id = result?;

            header
                .string_maps()
                .strings()
                .get_index_of(id)
                .ok_or_else(|| {
                    io::Error::new(
                        io::ErrorKind::InvalidInput,
                        format!("filter missing from string map: {id}"),
                    )
                })
        })
        .collect::<Result<_, _>>()?;

    write_string_map_indices(writer, &indices)
}

#[cfg(test)]
mod tests {
    use noodles_vcf::header::StringMaps;

    use super::*;

    #[test]
    fn test_write_filters() -> Result<(), Box<dyn std::error::Error>> {
        use vcf::{
            header::record::value::{map::Filter, Map},
            variant::record_buf::Filters,
        };

        fn t(
            buf: &mut Vec<u8>,
            header: &vcf::Header,
            filters: &Filters,
            expected: &[u8],
        ) -> io::Result<()> {
            buf.clear();
            write_filters(buf, header, filters)?;
            assert_eq!(buf, expected);
            Ok(())
        }

        let mut header = vcf::Header::builder()
            .add_filter("PASS", Map::<Filter>::pass())
            .add_filter(
                "s50",
                Map::<Filter>::new("Less than 50% of samples have data"),
            )
            .add_filter("q10", Map::<Filter>::new("Quality below 10"))
            .build();
        *header.string_maps_mut() = StringMaps::try_from(&header)?;

        let mut buf = Vec::new();

        let filters = Filters::default();
        t(&mut buf, &header, &filters, &[0x00])?;

        let filters = Filters::pass();
        t(&mut buf, &header, &filters, &[0x11, 0x00])?;

        let filters = [String::from("q10")].into_iter().collect();
        t(&mut buf, &header, &filters, &[0x11, 0x02])?;

        let filters = [String::from("q10"), String::from("s50")]
            .into_iter()
            .collect();
        t(&mut buf, &header, &filters, &[0x21, 0x02, 0x01])?;

        Ok(())
    }
}
