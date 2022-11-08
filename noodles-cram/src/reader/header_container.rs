mod header;

use std::{
    io::{self, Read},
    str,
};

use bytes::{Buf, Bytes, BytesMut};

use self::header::read_header;
use crate::container::Block;

pub fn read_header_container<R>(reader: &mut R, buf: &mut BytesMut) -> io::Result<String>
where
    R: Read,
{
    let len = read_header(reader)?;

    buf.resize(len, 0);
    reader.read_exact(buf)?;
    let mut buf = buf.split().freeze();

    read_raw_sam_header_from_block(&mut buf)
}

pub fn read_raw_sam_header_from_block(src: &mut Bytes) -> io::Result<String> {
    use super::container::read_block;

    let block = read_block(src)?;
    read_raw_sam_header(&block)
}

fn read_raw_sam_header(block: &Block) -> io::Result<String> {
    use crate::container::block::{CompressionMethod, ContentType};

    const EXPECTED_CONTENT_TYPE: ContentType = ContentType::FileHeader;

    if !matches!(
        block.compression_method(),
        CompressionMethod::None | CompressionMethod::Gzip
    ) {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            format!(
                "invalid block compression method: expected {:?} or {:?}, got {:?}",
                CompressionMethod::None,
                CompressionMethod::Gzip,
                block.compression_method()
            ),
        ));
    }

    if block.content_type() != EXPECTED_CONTENT_TYPE {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            format!(
                "invalid block content type: expected {:?}, got {:?}",
                EXPECTED_CONTENT_TYPE,
                block.content_type()
            ),
        ));
    }

    let mut data = block.decompressed_data()?;

    let len = usize::try_from(data.get_i32_le())
        .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;

    data.truncate(len);

    str::from_utf8(&data[..])
        .map(|s| s.into())
        .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))
}
