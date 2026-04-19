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

/// Like `List<String>` but also accepts bare identifiers: `aliases(foo, bar)`.
#[derive(Debug, Default)]
pub struct AliasList(pub Vec<String>);

impl darling::FromMeta for AliasList {
    fn from_list(items: &[::syn::NestedMeta]) -> darling::Result<Self> {
        items
            .iter()
            .map(|item| match item {
                ::syn::NestedMeta::Lit(::syn::Lit::Str(s)) => Ok(s.value()),
                ::syn::NestedMeta::Meta(::syn::Meta::Path(p)) => p
                    .get_ident()
                    .map(|i| i.to_string())
                    .ok_or_else(|| darling::Error::custom("expected identifier or string")),
                _ => Err(darling::Error::custom("expected identifier or string literal")),
            })
            .collect::<darling::Result<Vec<String>>>()
            .map(AliasList)
    }
}
