use rand::seq::SliceRandom;
use rand::Rng;

use super::TwoParity;

/// Structure representing a permutation of indices
// Internals kept private to ensure the data is well-formed
#[derive(Clone, Eq, PartialEq, Debug)]
pub struct Permutation(Vec<usize>);

impl Permutation {
    pub fn from_len(len: usize) -> Self {
        let owned: Vec<usize> = (0..len).collect();
        Permutation(owned)
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn apply(&self, i: usize) -> usize {
        assert!(i < self.0.len(), "Index should be in range");

        self.0[i]
    }

    /// Determine the even/odd nature of the permutation.
    pub fn parity(&self) -> TwoParity {
        let mut seen = vec![false; self.len()];

        let mut running_odd = false;

        for i in 0..self.len() {
            if seen[i] {
                continue;
            }

            let mut cycle_length = 1;
            let mut j = self.0[i];

            while !seen[j] {
                seen[j] = true;
                cycle_length += 1;
                j = self.0[j];
            }

            if cycle_length % 2 == 1 {
                running_odd = !running_odd;
            }
        }

        if running_odd {
            TwoParity::Odd
        } else {
            TwoParity::Even
        }
    }
}

pub fn any_permutation<R: Rng>(rng: &mut R, len: usize) -> Permutation {
    let mut out: Vec<usize> = (0..len).collect();
    out.shuffle(rng);
    Permutation(out)
}

pub fn with_parity<R: Rng>(rng: &mut R, len: usize, desired: TwoParity) -> Permutation {
    let mut out: Vec<usize> = (0..len).collect();
    out.shuffle(rng);

    let out = Permutation(out);

    if desired != out.parity() {
        let mut swap = Permutation::from_len(len);
        swap.0.swap(0, 1);

        compose(&swap, &out)
    } else {
        out
    }
}

/// Gives the result of perm_a \circ perm_b, that is, |i| perm_a(perm_b(i))
pub fn compose(a: &Permutation, b: &Permutation) -> Permutation {
    assert_eq!(a.0.len(), b.0.len(), "Permutation lengths should match");

    Permutation((0..a.len()).map(|i| a.0[b.0[i]]).collect())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn even_odd_tests() {
        let fixtures: Vec<(Permutation, TwoParity)> = vec![
            // constant permutations do nothing
            (Permutation(vec![0, 1]), TwoParity::Even),
            (Permutation(vec![0, 1, 2]), TwoParity::Even),
            (Permutation(vec![0, 1, 2, 3]), TwoParity::Even),
            // one swap -- 0 fixed, (1 2) cycle
            (Permutation(vec![0, 2, 1]), TwoParity::Odd),
            // two swaps -- (0 3) cycle, (1 2) cycle
            (Permutation(vec![3, 2, 1, 0]), TwoParity::Even),
            // three cycle -- 0 fixed, (1 2 5) cycle, 3 fixed, 4 fixed, 6 fixed
            (Permutation(vec![0, 2, 5, 3, 4, 1, 6]), TwoParity::Even),
            // three cycle + swap -- 0 fixed, (1 2 5) cycle, (3 6) cycle, 4 fixed
            (Permutation(vec![0, 2, 5, 6, 4, 1, 3]), TwoParity::Odd),
        ];

        for (p, exp) in fixtures {
            let act = p.parity();

            assert_eq!(act, exp, "Parity should match for {p:?}");
        }
    }
}
