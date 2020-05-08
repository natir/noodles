#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Base {
    Eq,
    A,
    C,
    M,
    G,
    R,
    S,
    V,
    T,
    W,
    Y,
    H,
    K,
    D,
    B,
    N,
}

impl Base {
    // https://en.wikipedia.org/wiki/Nucleic_acid_notation#IUPAC_notation
    pub fn complement(self) -> Self {
        match self {
            Self::Eq => Self::Eq,
            Self::A => Self::T,
            Self::C => Self::G,
            Self::G => Self::C,
            Self::T => Self::A,
            Self::W => Self::W,
            Self::S => Self::S,
            Self::M => Self::K,
            Self::K => Self::M,
            Self::R => Self::Y,
            Self::Y => Self::R,
            Self::B => Self::V,
            Self::D => Self::H,
            Self::H => Self::D,
            Self::V => Self::B,
            Self::N => Self::N,
        }
    }
}

impl From<Base> for char {
    fn from(base: Base) -> Self {
        match base {
            Base::Eq => '=',
            Base::A => 'A',
            Base::C => 'C',
            Base::G => 'G',
            Base::T => 'T',
            Base::W => 'W',
            Base::S => 'S',
            Base::M => 'M',
            Base::K => 'K',
            Base::R => 'R',
            Base::Y => 'Y',
            Base::B => 'B',
            Base::D => 'D',
            Base::H => 'H',
            Base::V => 'V',
            Base::N => 'N',
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_complement() {
        assert_eq!(Base::Eq.complement(), Base::Eq);
        assert_eq!(Base::A.complement(), Base::T);
        assert_eq!(Base::C.complement(), Base::G);
        assert_eq!(Base::M.complement(), Base::K);
        assert_eq!(Base::G.complement(), Base::C);
        assert_eq!(Base::R.complement(), Base::Y);
        assert_eq!(Base::S.complement(), Base::S);
        assert_eq!(Base::V.complement(), Base::B);
        assert_eq!(Base::T.complement(), Base::A);
        assert_eq!(Base::W.complement(), Base::W);
        assert_eq!(Base::Y.complement(), Base::R);
        assert_eq!(Base::H.complement(), Base::D);
        assert_eq!(Base::K.complement(), Base::M);
        assert_eq!(Base::D.complement(), Base::H);
        assert_eq!(Base::B.complement(), Base::V);
        assert_eq!(Base::N.complement(), Base::N);
    }

    #[test]
    fn test_from_base_for_char() {
        assert_eq!(char::from(Base::Eq), '=');
        assert_eq!(char::from(Base::A), 'A');
        assert_eq!(char::from(Base::C), 'C');
        assert_eq!(char::from(Base::M), 'M');
        assert_eq!(char::from(Base::G), 'G');
        assert_eq!(char::from(Base::R), 'R');
        assert_eq!(char::from(Base::S), 'S');
        assert_eq!(char::from(Base::V), 'V');
        assert_eq!(char::from(Base::T), 'T');
        assert_eq!(char::from(Base::W), 'W');
        assert_eq!(char::from(Base::Y), 'Y');
        assert_eq!(char::from(Base::H), 'H');
        assert_eq!(char::from(Base::K), 'K');
        assert_eq!(char::from(Base::D), 'D');
        assert_eq!(char::from(Base::B), 'B');
        assert_eq!(char::from(Base::N), 'N');
    }
}
