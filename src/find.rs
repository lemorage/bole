/// Generic discovery trait used by lightweight "detectors" to enumerate
/// instances of a given tool or resource.
///
/// In this codebase it is implemented by package‑manager detectors (npm, pip,
/// brew, etc.). Implementations should return all discoverable instances with
/// the following conventions:
/// - The first element is the active instance resolved from PATH.
/// - Additional elements are instances found in well‑known install locations.
/// - Results should be de‑duplicated by canonical path.
///
/// The `Output` type is the per‑instance info struct (e.g. `PmInfo`).
pub trait Find {
    /// Per‑instance information yielded by this finder.
    type Output;
    /// Stable identifier for the thing being discovered.
    fn name(&self) -> &'static str;
    /// Run discovery and return all found instances in the order described
    /// above (active first, then other locations), with duplicates removed.
    fn find(&self) -> Vec<Self::Output>;
}

/// Handy blanket impls so `&T` and `Box<T>` can be used anywhere a `Find`
/// implementor is expected, without forcing clones or moves.
impl<T: Find + ?Sized> Find for &T {
    type Output = T::Output;
    fn name(&self) -> &'static str {
        (**self).name()
    }
    fn find(&self) -> Vec<Self::Output> {
        (**self).find()
    }
}

impl<T: Find + ?Sized> Find for Box<T> {
    type Output = T::Output;
    fn name(&self) -> &'static str {
        (**self).name()
    }
    fn find(&self) -> Vec<Self::Output> {
        (**self).find()
    }
}
