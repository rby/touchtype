/// A `UniqEnumerate` wraps an `Iterator` and returns the same enumeration
/// for a contiguous sequnce of equal vlaues.
/// TODO example, here
///
struct UniqEnumerateIter<'a, T: Iterator> {
    inner_iter: &'a mut T,
    last: Option<T::Item>,
    ix: usize,
}

struct UniqEnumerateProjIter<'a, T, R, F> {
    inner_iter: &'a mut T,
    last: Option<&'a R>,
    ix: usize,
    proj: F,
}

impl<'a, Item: PartialEq + 'a, T: Iterator<Item = &'a Item>> Iterator for UniqEnumerateIter<'a, T> {
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

impl<'a, F, Item: 'a, R: PartialEq + 'a, T: Iterator<Item = &'a Item>> Iterator
    for UniqEnumerateProjIter<'a, T, R, F>
where
    F: Fn(T::Item) -> &'a R,
{
    type Item = (usize, &'a Item);
    fn next(&mut self) -> Option<Self::Item> {
        if let Some(next) = self.inner_iter.next() {
            let r = (self.proj)(next);
            if let Some(last) = self.last {
                if last != r {
                    self.ix += 1;
                    self.last = Some(r);
                }
                Some((self.ix, next))
            } else {
                self.last = Some(r);
                Some((self.ix, next))
            }
        } else {
            None
        }
    }
}

pub(crate) fn enumerate<'a, I, Item>(iter: &'a mut I) -> impl Iterator<Item = (usize, &'a Item)>
where
    Item: PartialEq + 'a,
    I: Iterator<Item = &'a Item>,
{
    UniqEnumerateIter {
        inner_iter: iter,
        last: None,
        ix: 0,
    }
}

pub(crate) fn enumerate_proj<'a, Item, I, R, F>(
    iter: &'a mut I,
    f: F,
) -> impl Iterator<Item = (usize, &'a Item)>
where
    Item: 'a,
    I: Iterator<Item = &'a Item>,
    <I as Iterator>::Item: 'a,
    R: PartialEq + 'a,
    F: Fn(<I as Iterator>::Item) -> &'a R,
{
    UniqEnumerateProjIter {
        inner_iter: iter,
        last: None,
        ix: 0,
        proj: f,
    }
}

#[cfg(test)]
mod test {
    use crate::uniq_enumerate::{enumerate, enumerate_proj};
    #[test]
    pub fn it_increases_only_when_item_change() {
        let v = vec![1, 1, 2, 2, 3, 4];
        let mut iter = v.iter();
        let v: Vec<(usize, &u8)> = enumerate(&mut iter).collect();
        assert_eq!(v, &[(0, &1), (0, &1), (1, &2), (1, &2), (2, &3), (3, &4)])
    }
    #[test]
    pub fn it_projects_and_enumerate_only_uniq_values() {
        let v = vec![("a", 1), ("b", 1), ("c", 2), ("d", 2), ("e", 3), ("f", 4)];
        let mut iter = v.iter();
        let v: Vec<(usize, &(&str, u8))> = enumerate_proj(&mut iter, |x| &x.1).collect();
        assert_eq!(
            v,
            &[
                (0, &("a", 1)),
                (0, &("b", 1)),
                (1, &("c", 2)),
                (1, &("d", 2)),
                (2, &("e", 3)),
                (3, &("f", 4))
            ]
        )
    }
}
