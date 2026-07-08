pub(crate) struct Builder;

/// Promotes the container to a 64-bit variant when the declared size exceeds [`u32::MAX`].
/// For RIFF this means promoting to RF64 and inserting a [`ds64`] chunk.
fn promote_container() -> () {
    todo!()
}
