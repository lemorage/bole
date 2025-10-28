//! Data transformation pipeline.

/// Pipeline for chaining operations on data.
pub(crate) struct Pipeline<T> {
    data: T,
}

impl<T> Pipeline<T> {
    /// Start a new pipeline with the given data.
    pub(crate) fn new(data: T) -> Self {
        Self { data }
    }

    /// Transform the data through a function.
    pub(crate) fn then<U, F>(self, f: F) -> Pipeline<U>
    where
        F: FnOnce(T) -> U,
    {
        Pipeline::new(f(self.data))
    }

    /// Apply a transformation only if the condition is true.
    #[allow(dead_code)]
    pub(crate) fn apply_if<F>(self, condition: bool, f: F) -> Pipeline<T>
    where
        F: FnOnce(T) -> T,
    {
        if condition {
            Pipeline::new(f(self.data))
        } else {
            self
        }
    }

    /// Perform a side effect without changing the data.
    #[allow(dead_code)]
    pub(crate) fn map<F>(self, f: F) -> Pipeline<T>
    where
        F: FnOnce(&T),
    {
        f(&self.data);
        self
    }

    /// Extract the final value from the pipeline.
    pub(crate) fn finish(self) -> T {
        self.data
    }
}

/// Create a new pipeline with the given data.
pub(crate) fn pipe<T>(data: T) -> Pipeline<T> {
    Pipeline::new(data)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_pipeline() {
        let result = pipe(5).then(|x| x * 2).then(|x| x + 1).finish();
        assert_eq!(result, 11);
    }

    #[test]
    fn test_conditional_pipeline() {
        let result = pipe(10)
            .apply_if(true, |x| x * 2)
            .apply_if(false, |x| x + 100)
            .finish();
        assert_eq!(result, 20);
    }

    #[test]
    fn test_map_side_effect() {
        let mut side_effect = 0;
        let result = pipe(42).map(|x| side_effect = *x).then(|x| x * 2).finish();
        assert_eq!(side_effect, 42);
        assert_eq!(result, 84);
    }
}
