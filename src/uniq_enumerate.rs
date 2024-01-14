/// A `UniqEnumerate` wraps an `Iterator` and returns the same enumeration
/// for a contiguous sequnce of equal vlaues.
/// TODO example, here
///
pub(crate) struct UniqEnumerateIter<'a, I, A> {
    inner_iter: &'a mut I,
    last: Option<A>,
    ix: usize,
}

pub(crate) struct UniqEnumerateProjIter<'a, I, A, F> {
    inner_iter: &'a mut I,
    last: Option<A>,
    ix: usize,
    proj: F,
}

impl<'a, Item, I> Iterator for UniqEnumerateIter<'a, I, Item>
where
    Item: PartialEq,
    I: Iterator<Item = Item>,
{
    type Item = (usize, Item);
    fn next(&mut self) -> Option<Self::Item> {
        if self.last.is_some() {
            let ix = self.ix;
            let maybe_next = self.inner_iter.next();
            let is_eq = self.last == maybe_next;
            if !is_eq {
                self.ix += 1;
            }
            let res = if let Some(next) = maybe_next {
                self.last.replace(next)
            } else {
                self.last.take()
            };
            res.map(|x| (ix, x))
        } else {
            None
        }
    }
}

impl<'a, Proj, I, F> Iterator for UniqEnumerateProjIter<'a, I, <I as Iterator>::Item, F>
where
    Proj: PartialEq,
    I: Iterator,
    F: Fn(&<I as Iterator>::Item) -> Proj,
{
    type Item = (usize, I::Item);
    fn next(&mut self) -> Option<Self::Item> {
        if self.last.is_some() {
            let ix = self.ix;
            let maybe_next = self.inner_iter.next();
            let is_eq = self.last.as_ref().map(|x| (self.proj)(&x))
                == maybe_next.as_ref().map(|x| (self.proj)(&x));
            if !is_eq {
                self.ix += 1;
            }
            let res = if let Some(next) = maybe_next {
                self.last.replace(next)
            } else {
                self.last.take()
            };
            res.map(|x| (ix, x))
        } else {
            None
        }
    }
}

pub(crate) fn enumerate<'a, I, Item>(iter: &'a mut I) -> UniqEnumerateIter<'a, I, Item>
where
    Item: PartialEq,
    I: Iterator<Item = Item>,
{
    let next = iter.next();
    UniqEnumerateIter {
        inner_iter: iter,
        last: next,
        ix: 0,
    }
}

pub(crate) fn enumerate_proj<'a, I, R, F>(
    iter: &'a mut I,
    f: F,
) -> UniqEnumerateProjIter<'a, I, <I as Iterator>::Item, F>
where
    I: Iterator,
    R: PartialEq,
    F: Fn(&<I as Iterator>::Item) -> R,
{
    let next = iter.next();
    UniqEnumerateProjIter {
        inner_iter: iter,
        last: next,
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
        let v: Vec<(&str, u8)> = vec![("a", 1), ("b", 1), ("c", 2), ("d", 2), ("e", 3), ("f", 4)];
        let mut iter = v.iter();
        let v: Vec<(usize, &(&str, u8))> = enumerate_proj(&mut iter, |x| x.1).collect();
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
    #[test]
    pub fn it_can_detect_line_changes() {
        let ints: std::ops::Range<i32> = 1..6;
        let mut iter = ints.enumerate();
        let v: Vec<(usize, (usize, i32))> = enumerate_proj(&mut iter, |x| x.0 / 3).collect();
        assert_eq!(
            v,
            &[
                (0, (0, 1)),
                (0, (1, 2)),
                (0, (2, 3)),
                (1, (3, 4)),
                (1, (4, 5))
            ]
        )
    }
}
