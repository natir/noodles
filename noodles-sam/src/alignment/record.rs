//! Alignment record.

pub mod cigar;
pub mod data;
mod flags;
pub mod mapping_quality;
pub mod quality_scores;
pub mod read_name;
pub mod sequence;

pub use self::{
    cigar::Cigar, data::Data, flags::Flags, mapping_quality::MappingQuality,
    quality_scores::QualityScores, read_name::ReadName, sequence::Sequence,
};

use self::{quality_scores::Score, sequence::Base};

/// An alignment record sequence.
pub trait AlignmentSequence {
    /// Returns the number of bases in the sequence.
    fn len(&self) -> usize;

    /// Returns whether the sequence is empty.
    fn is_empty(&self) -> bool;

    /// Removes all bases from the sequence.
    fn clear(&mut self);

    /// Returns an iterator over the bases in the sequence.
    fn bases(&self) -> Box<dyn Iterator<Item = Base> + '_>;
}

/// Alignment record quality scores.
pub trait AlignmentQualityScores {
    /// Returns the number of scores.
    fn len(&self) -> usize;

    /// Returns whether there are any scores.
    fn is_empty(&self) -> bool;

    /// Removes all scores.
    fn clear(&mut self);

    /// Returns an iterator over the scores.
    fn scores(&self) -> Box<dyn Iterator<Item = Score> + '_>;
}