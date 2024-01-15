/// A `RunEnumerateIter` wraps an `Iterator` and returns the same enumeration
/// as long as the inner value are the same.
///
/// A `RunEnumerateProjectIter` does the same but using a projection function
/// to check the equality.
///
/// For lack of other alternatives, the name is adapted from RunLength which accumulates the length instead.

#[derive(Clone, Default, PartialEq)]
enum Ix {
    #[default]
    None,
    At(usize),
}

impl From<usize> for Ix {
    fn from(value: usize) -> Self {
        Ix::At(value)
    }
}

impl Ix {
    fn inc(&mut self) {
        match self {
            Ix::None => *self = Ix::At(0),
            Ix::At(i) => *i += 1,
        }
    }
}

pub(crate) struct RunEnumerateIter<I, A> {
    inner_iter: I,
    last: Option<A>,
    ix: Ix,
}

impl<Item, I> Iterator for RunEnumerateIter<I, Item>
where
    Item: PartialEq,
    I: Iterator<Item = Item>,
{
    type Item = (usize, Item);
    fn next(&mut self) -> Option<Self::Item> {
        match self.ix {
            Ix::None => {
                // initialize
                if let Some(next) = self.inner_iter.next() {
                    let _ = self.last.insert(next);
                    self.ix.inc();
                    self.next()
                } else {
                    None
                }
            }
            Ix::At(ix) => {
                if self.last.is_some() {
                    let maybe_next = self.inner_iter.next();
                    let is_eq = self.last == maybe_next;
                    if !is_eq {
                        self.ix.inc();
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
    }
}

pub(crate) struct RunEnumerateProjectIter<I, A, F> {
    inner_iter: I,
    last: Option<A>,
    ix: Ix,
    proj: F,
}

impl<Proj, I, F> Iterator for RunEnumerateProjectIter<I, <I as Iterator>::Item, F>
where
    Proj: PartialEq,
    I: Iterator,
    F: Fn(&<I as Iterator>::Item) -> Proj,
{
    type Item = (usize, I::Item);
    fn next(&mut self) -> Option<Self::Item> {
        match self.ix {
            Ix::None => {
                // initialize
                if let Some(next) = self.inner_iter.next() {
                    let _ = self.last.insert(next);
                    self.ix.inc();
                    self.next()
                } else {
                    None
                }
            }
            Ix::At(ix) => {
                if self.last.is_some() {
                    let maybe_next = self.inner_iter.next();
                    let is_eq = self.last.as_ref().map(|x| (self.proj)(&x))
                        == maybe_next.as_ref().map(|x| (self.proj)(&x));
                    if !is_eq {
                        self.ix.inc();
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
    }
}

pub(crate) fn run_enumerate<I, Item>(iter: I) -> RunEnumerateIter<I, Item>
where
    Item: PartialEq,
    I: Iterator<Item = Item>,
{
    RunEnumerateIter {
        inner_iter: iter,
        last: None,
        ix: Ix::None,
    }
}

/// Given an iterator, the enumeration doesn't change when consecutive
/// items are the same according to the projection function `proj`.
///
/// # Example
/// `proj` computes the floor division by 3 which groups items by triplets.
///
/// ```
///   let ints: std::ops::Range<i32> = 1..6;
///   let mut iter = ints.enumerate();
///   let v: Vec<(usize, (usize, i32))> = run_enumerate_with(&mut iter, |x| x.0 / 3).collect();
///   assert_eq!(
///       v,
///       &[
///           (0, (0, 1)),
///           (0, (1, 2)),
///           (0, (2, 3)),
///           (1, (3, 4)),
///           (1, (4, 5))
///       ]
///   )
/// ```
pub(crate) fn run_enumerate_with<I, R, F>(
    iter: I,
    f: F,
) -> RunEnumerateProjectIter<I, <I as Iterator>::Item, F>
where
    I: Iterator,
    R: PartialEq,
    F: Fn(&<I as Iterator>::Item) -> R,
{
    RunEnumerateProjectIter {
        inner_iter: iter,
        last: None,
        ix: Ix::None,
        proj: f,
    }
}

#[cfg(test)]
mod test {
    use crate::uniq_enumerate::{run_enumerate, run_enumerate_with};
    #[test]
    pub fn it_increases_only_when_item_change() {
        let v = vec![1, 1, 2, 2, 3, 4];
        let mut iter = v.iter();
        let v: Vec<(usize, &u8)> = run_enumerate(&mut iter).collect();
        assert_eq!(v, &[(0, &1), (0, &1), (1, &2), (1, &2), (2, &3), (3, &4)])
    }
    #[test]
    pub fn it_increases_only_when_projection_changes() {
        let v: Vec<(&str, u8)> = vec![("a", 1), ("b", 1), ("c", 2), ("d", 2), ("e", 3), ("f", 4)];
        let mut iter = v.iter();
        let v: Vec<(usize, &(&str, u8))> = run_enumerate_with(&mut iter, |x| x.1).collect();
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
        let v: Vec<(usize, (usize, i32))> = run_enumerate_with(&mut iter, |x| x.0 / 3).collect();
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
