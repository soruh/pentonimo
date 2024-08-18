use std::fmt::Debug;

#[derive(Clone, Copy, Hash, PartialEq, Eq)]
pub struct Candidates(u64);

impl IntoIterator for Candidates {
    type Item = u8;

    type IntoIter = CandidateIterator;

    fn into_iter(self) -> Self::IntoIter {
        CandidateIterator {
            candidates: self,
            i: 0,
        }
    }
}

pub struct CandidateIterator {
    candidates: Candidates,
    i: u8,
}

impl Iterator for CandidateIterator {
    type Item = u8;

    fn next(&mut self) -> Option<Self::Item> {
        if self.i < 12 {
            self.i += 1;
            Some(self.candidates.get(self.i - 1))
        } else {
            None
        }
    }
}

impl Debug for Candidates {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("Candidates")
            .field(&self.into_iter().collect::<Vec<_>>())
            .finish()
    }
}

impl Candidates {
    pub fn mask(i: u8) -> u64 {
        0b11111 << (5 * i)
    }
    pub fn get(self, i: u8) -> u8 {
        ((self.0 & Self::mask(i)) >> (5 * i)) as u8
    }
    pub fn set(&mut self, i: u8, v: u8) {
        assert!(v < 0b100000);

        self.0 = (self.0 & !Self::mask(i)) | ((v as u64) << (5 * i));
    }

    pub fn decrement(&mut self, i: u8) {
        self.set(i, self.get(i).checked_sub(1).unwrap())
    }

    pub fn new(candidates: [u8; 12]) -> Self {
        let mut res: u64 = 0;

        for (i, &v) in candidates.iter().enumerate() {
            assert!(v < 0b100000);
            res |= ((v & 0b11111) as u64) << (5 * i);
        }

        Self(res)
    }
}
