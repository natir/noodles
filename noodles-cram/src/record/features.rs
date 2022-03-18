mod with_positions;

pub use self::with_positions::WithPositions;

use std::{
    ops::{Deref, DerefMut},
    slice,
};

use noodles_core::Position;
use noodles_sam as sam;

use super::Feature;

/// CRAM record features.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct Features(Vec<Feature>);

impl Features {
    /// Converts SAM record CIGAR operations to CRAM record features.
    pub fn from_cigar(
        cigar: &sam::record::Cigar,
        sequence: &sam::record::Sequence,
        quality_scores: &sam::record::QualityScores,
    ) -> Self {
        cigar_to_features(cigar, sequence, quality_scores)
    }

    pub(crate) fn with_positions(
        &self,
        alignment_start: Position,
    ) -> WithPositions<'_, slice::Iter<'_, Feature>> {
        WithPositions::new(self.iter(), alignment_start)
    }
}

impl Deref for Features {
    type Target = Vec<Feature>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for Features {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl From<Vec<Feature>> for Features {
    fn from(features: Vec<Feature>) -> Self {
        Self(features)
    }
}

fn cigar_to_features(
    cigar: &sam::record::Cigar,
    sequence: &sam::record::Sequence,
    quality_scores: &sam::record::QualityScores,
) -> Features {
    use sam::record::cigar::op::Kind;

    let mut features = Features::default();
    let mut read_position = Position::MIN;

    for op in cigar.iter() {
        match op.kind() {
            Kind::Match | Kind::SequenceMatch | Kind::SequenceMismatch => {
                if op.len() == 1 {
                    let base = sequence[read_position];
                    let score = quality_scores[read_position];
                    features.push(Feature::ReadBase(read_position, base, score));
                } else {
                    let end = read_position
                        .checked_add(op.len())
                        .expect("attempt to add with overflow");

                    let bases = &sequence[read_position..end];
                    features.push(Feature::Bases(read_position, bases.to_vec()));

                    let scores = &quality_scores[read_position..end];
                    features.push(Feature::Scores(read_position, scores.to_vec()));
                }
            }
            Kind::Insertion => {
                if op.len() == 1 {
                    let base = sequence[read_position];
                    features.push(Feature::InsertBase(read_position, base));

                    let score = quality_scores[read_position];
                    features.push(Feature::QualityScore(read_position, score));
                } else {
                    let end = read_position
                        .checked_add(op.len())
                        .expect("attempt to add with overflow");

                    let bases = &sequence[read_position..end];
                    features.push(Feature::Insertion(read_position, bases.to_vec()));

                    let scores = &quality_scores[read_position..end];
                    features.push(Feature::Scores(read_position, scores.to_vec()));
                }
            }
            Kind::Deletion => features.push(Feature::Deletion(read_position, op.len())),
            Kind::Skip => features.push(Feature::ReferenceSkip(read_position, op.len())),
            Kind::SoftClip => {
                let end = read_position
                    .checked_add(op.len())
                    .expect("attempt to add with overflow");

                let bases = &sequence[read_position..end];
                features.push(Feature::SoftClip(read_position, bases.to_vec()));

                if bases.len() == 1 {
                    let score = quality_scores[read_position];
                    features.push(Feature::QualityScore(read_position, score));
                } else {
                    let scores = &quality_scores[read_position..end];
                    features.push(Feature::Scores(read_position, scores.to_vec()));
                }
            }
            Kind::HardClip => features.push(Feature::HardClip(read_position, op.len())),
            Kind::Pad => features.push(Feature::Padding(read_position, op.len())),
        };

        if matches!(
            op.kind(),
            Kind::Match
                | Kind::Insertion
                | Kind::SoftClip
                | Kind::SequenceMatch
                | Kind::SequenceMismatch
        ) {
            read_position = read_position
                .checked_add(op.len())
                .expect("attempt to add with overflow");
        }
    }

    features
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cigar_to_features() -> Result<(), Box<dyn std::error::Error>> {
        use sam::record::{quality_scores::Score, sequence::Base};

        let cigar = "1M".parse()?;
        let sequence = "A".parse()?;
        let quality_scores = "N".parse()?;
        let actual = cigar_to_features(&cigar, &sequence, &quality_scores);
        let expected = Features::from(vec![Feature::ReadBase(
            Position::try_from(1)?,
            Base::A,
            Score::try_from('N')?,
        )]);
        assert_eq!(actual, expected);

        let cigar = "2M".parse()?;
        let sequence = "AC".parse()?;
        let quality_scores = "ND".parse()?;
        let actual = cigar_to_features(&cigar, &sequence, &quality_scores);
        let expected = Features::from(vec![
            Feature::Bases(Position::try_from(1)?, vec![Base::A, Base::C]),
            Feature::Scores(
                Position::try_from(1)?,
                vec![Score::try_from('N')?, Score::try_from('D')?],
            ),
        ]);
        assert_eq!(actual, expected);

        let cigar = "1I1M".parse()?;
        let sequence = "AC".parse()?;
        let quality_scores = "ND".parse()?;
        let actual = cigar_to_features(&cigar, &sequence, &quality_scores);
        let expected = Features::from(vec![
            Feature::InsertBase(Position::try_from(1)?, Base::A),
            Feature::QualityScore(Position::try_from(1)?, Score::try_from('N')?),
            Feature::ReadBase(Position::try_from(2)?, Base::C, Score::try_from('D')?),
        ]);
        assert_eq!(actual, expected);

        let cigar = "2I1M".parse()?;
        let sequence = "ACG".parse()?;
        let quality_scores = "NDL".parse()?;
        let actual = cigar_to_features(&cigar, &sequence, &quality_scores);
        let expected = Features::from(vec![
            Feature::Insertion(Position::try_from(1)?, vec![Base::A, Base::C]),
            Feature::Scores(
                Position::try_from(1)?,
                vec![Score::try_from('N')?, Score::try_from('D')?],
            ),
            Feature::ReadBase(Position::try_from(3)?, Base::G, Score::try_from('L')?),
        ]);
        assert_eq!(actual, expected);

        let cigar = "1D2M".parse()?;
        let sequence = "AC".parse()?;
        let quality_scores = "ND".parse()?;
        let actual = cigar_to_features(&cigar, &sequence, &quality_scores);
        let expected = Features::from(vec![
            Feature::Deletion(Position::try_from(1)?, 1),
            Feature::Bases(Position::try_from(1)?, vec![Base::A, Base::C]),
            Feature::Scores(
                Position::try_from(1)?,
                vec![Score::try_from('N')?, Score::try_from('D')?],
            ),
        ]);
        assert_eq!(actual, expected);

        let cigar = "1N1M".parse()?;
        let sequence = "A".parse()?;
        let quality_scores = "N".parse()?;
        let actual = cigar_to_features(&cigar, &sequence, &quality_scores);
        let expected = Features::from(vec![
            Feature::ReferenceSkip(Position::try_from(1)?, 1),
            Feature::ReadBase(Position::try_from(1)?, Base::A, Score::try_from('N')?),
        ]);
        assert_eq!(actual, expected);

        let cigar = "1S1M".parse()?;
        let sequence = "AC".parse()?;
        let quality_scores = "ND".parse()?;
        let actual = cigar_to_features(&cigar, &sequence, &quality_scores);
        let expected = Features::from(vec![
            Feature::SoftClip(Position::try_from(1)?, vec![Base::A]),
            Feature::QualityScore(Position::try_from(1)?, Score::try_from('N')?),
            Feature::ReadBase(Position::try_from(2)?, Base::C, Score::try_from('D')?),
        ]);
        assert_eq!(actual, expected);

        let cigar = "2S1M".parse()?;
        let sequence = "ACG".parse()?;
        let quality_scores = "NDL".parse()?;
        let actual = cigar_to_features(&cigar, &sequence, &quality_scores);
        let expected = Features::from(vec![
            Feature::SoftClip(Position::try_from(1)?, vec![Base::A, Base::C]),
            Feature::Scores(
                Position::try_from(1)?,
                vec![Score::try_from('N')?, Score::try_from('D')?],
            ),
            Feature::ReadBase(Position::try_from(3)?, Base::G, Score::try_from('L')?),
        ]);
        assert_eq!(actual, expected);

        let cigar = "1H1M".parse()?;
        let sequence = "A".parse()?;
        let quality_scores = "N".parse()?;
        let actual = cigar_to_features(&cigar, &sequence, &quality_scores);
        let expected = Features::from(vec![
            Feature::HardClip(Position::try_from(1)?, 1),
            Feature::ReadBase(Position::try_from(1)?, Base::A, Score::try_from('N')?),
        ]);
        assert_eq!(actual, expected);

        let cigar = "1P1M".parse()?;
        let sequence = "A".parse()?;
        let quality_scores = "N".parse()?;
        let actual = cigar_to_features(&cigar, &sequence, &quality_scores);
        let expected = Features::from(vec![
            Feature::Padding(Position::try_from(1)?, 1),
            Feature::ReadBase(Position::try_from(1)?, Base::A, Score::try_from('N')?),
        ]);
        assert_eq!(actual, expected);

        Ok(())
    }
}
