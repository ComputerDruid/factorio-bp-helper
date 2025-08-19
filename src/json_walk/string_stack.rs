use std::marker::PhantomData;

pub struct StringStack<'a, 'parent> {
    // unsafe field: must only access the first len elements, must not mutate the strings, strings only valid for 'a.
    // other strings can be pushed but their validity can only be proved locally.
    storage: &'parent mut Vec<*const str>,
    // unsafe field; indicates which elements are safe to read from for 'a
    len: usize,
    phantom: PhantomData<&'a str>,
}

impl<'parent> StringStack<'static, 'parent> {
    pub fn new(storage: &'parent mut Vec<*const str>) -> Self {
        Self {
            // SAFETY: we don't need any of the elements to be valid because we start with len=0
            storage,
            // SAFETY: len=0 trivially valid because it is always safe to read the first 0 elements
            len: 0,
            phantom: Default::default(),
        }
    }
}

impl<'a, 'parent> StringStack<'a, 'parent> {
    pub fn as_slice<'s>(&'s self) -> &'s [&'a str] {
        let slice: &'s [*const str] = &self.storage[..self.len];
        // SAFETY: it is safe to read the first len elements for 'a.
        // Using a lifetime of 's for the slice ensures we will not mutate the storage while this slice is alive.
        unsafe {
            std::slice::from_raw_parts::<'s, &'a str>(slice.as_ptr() as *const &'a str, slice.len())
        }
    }

    #[must_use]
    pub fn push<'s, 'b>(&'s mut self, s: &'b str) -> StringStack<'b, 's>
    where
        'a: 'b,
    {
        assert!(self.len <= self.storage.len());
        // SAFETY: assert proves we're only shrinking here, so new len will be within capacity and all the elements will already be initialized.
        unsafe {
            self.storage.set_len(self.len);
        }
        // SAFETY: we're allowed to push to storage; this new string will be valid for lifetime 'b
        self.storage.push(s as *const str);

        // demonstration: the existing strings from this stack are valid `&'b str`s:
        let _: &[&'b str] = self.as_slice();

        // SAFETY: The new StringStack will allow access to:
        // - the existing strings from this stack for lifetime `'b` (see demonstration, above)
        // - plus the new one we just pushed, which is also valid for `'b`.
        // Because the new StringStack is reborrowing the storage from this one, we won't
        // accidentally overwrite the new element in storage with one that isn't valid for
        // `'b` until it's no longer relevant.
        StringStack {
            storage: self.storage,
            len: self.len + 1,
            phantom: Default::default(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_string_stack() {
        let mut s = vec![];
        let mut stack = StringStack::new(&mut s);
        let mut stack = stack.push("a");
        {
            let b = "b".to_owned();
            let stack = stack.push(&b);
            assert_eq!(stack.as_slice(), ["a", "b"]);
        };
        assert_eq!(stack.as_slice(), ["a"]);
        let _: &'static str = stack.as_slice()[0];
    }
}
