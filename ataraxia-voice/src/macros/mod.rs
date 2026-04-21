
#[macro_export]
/// Throws an error but without requiring Rust Nightly Toolchain.
macro_rules! yeet {
    ($e:expr) => {
        return Err($e);
    };
}