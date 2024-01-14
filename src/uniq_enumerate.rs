/// A `UniqEnumerate` wraps an `Iterator` and returns the same enumeration
/// for a contiguous sequnce of equal vlaues.
/// TODO example, here
///
struct UniqEnumerate<'a, T: Iterator> {
    inner_iter: &'a mut T,
    last: Option<T::Item>,
    ix: usize,
}

impl<'a, Item: PartialEq + 'a, T: Iterator<Item = &'a Item>> Iterator for UniqEnumerate<'a, T> {
    type Item = (usize, &'a Item);
    fn next(&mut self) -> Option<Self::Item> {
        if let some_next @ Some(next) = self.inner_iter.next() {
            if let Some(last) = self.last {
                if last != next {
                    self.ix += 1;
                    self.last = some_next;
                }
                Some((self.ix, next))
            } else {
                self.last = some_next;
                Some((self.ix, next))
            }
        } else {
            None
        }
    }
}

impl<'a, Item: PartialEq + 'a, T: Iterator<Item = &'a Item>> UniqEnumerate<'a, T> {
    fn from(inner_iter: &'a mut T) -> Self {
        Self {
            inner_iter,
            last: None,
            ix: 0,
        }
    }
}

#[cfg(test)]
mod test {
    use crate::uniq_enumerate::UniqEnumerate;
    #[test]
    pub fn it_increases_only_when_item_change() {
        let v = vec![1, 1, 2, 2, 3, 4];
        let mut iter = v.iter();
        let v: Vec<(usize, &u8)> = UniqEnumerate::from(&mut iter).collect();
        assert_eq!(v, &[(0, &1), (0, &1), (1, &2), (1, &2), (2, &3), (3, &4)])
    }
}
