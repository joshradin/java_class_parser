//! utility stuff

/// Matches a thing to a thing
macro_rules! match_as {
    (
        $($ids:ident),+;
        $pat:pat_param = $expr:expr
    ) => {
        match $expr {
            $pat => {
                Some(($($ids),*))
            }
            _ => None
        }
    };
}
pub(crate) use match_as;
