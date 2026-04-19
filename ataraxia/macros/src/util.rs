/// Darling utility type that accepts a list of things, e.g. `#[attr(thing1, thing2...)]`
#[derive(Debug)]
pub struct List<T>(pub Vec<T>);

impl<T: darling::FromMeta> darling::FromMeta for List<T> {
    fn from_list(items: &[::syn::NestedMeta]) -> darling::Result<Self> {
        items
            .iter()
            .map(|item| T::from_nested_meta(item))
            .collect::<darling::Result<Vec<T>>>()
            .map(Self)
    }
}

impl<T> Default for List<T> {
    fn default() -> Self {
        Self(Vec::default())
    }
}
