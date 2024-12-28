pub fn cycle_cw<T>(a: &mut T, b: &mut T, c: &mut T)
where
    T: Copy,
{
    let old_c = *c;
    *c = *b;
    *b = *a;
    *a = old_c;
}

#[cfg(test)]
mod tests {
    use crate::permutation_helpers::cycle_cw;

    #[test]
    fn cycle_cw_test() {
        let mut a = 1;
        let mut b = 2;
        let mut c = 3;

        cycle_cw(&mut a, &mut b, &mut c);

        assert_eq!(a, 3);
        assert_eq!(b, 1);
        assert_eq!(c, 2);

        cycle_cw(&mut a, &mut b, &mut c);

        assert_eq!(a, 2);
        assert_eq!(b, 3);
        assert_eq!(c, 1);

        cycle_cw(&mut a, &mut b, &mut c);

        assert_eq!(a, 1);
        assert_eq!(b, 2);
        assert_eq!(c, 3);
    }
}
