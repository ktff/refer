use super::{drain::Drain, itemize::Itemize, own::Own};

/// * --> Data<T>
pub type Data<T> = Own<Drain<Itemize<T>>>;
