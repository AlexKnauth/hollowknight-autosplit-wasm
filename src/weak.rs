
extern "C" {
    #[linkage = "extern_weak"]
    /// Accesses the index of the split the attempt is currently on. If there's
    /// no attempt in progress, `-1` is returned instead. This returns an
    /// index that is equal to the amount of segments when the attempt is
    /// finished, but has not been reset. So you need to be careful when using
    /// this value for indexing.
    static timer_current_split_index: Option<unsafe extern "C" fn () -> i32>;
}

/// Accesses the index of the split the attempt is currently on. If there's
/// no attempt in progress, `-1` is returned instead. This returns an
/// index that is equal to the amount of segments when the attempt is
/// finished, but has not been reset. So you need to be careful when using
/// this value for indexing.
pub fn get_timer_current_split_index() -> Option<i32> {
    let i = unsafe { (timer_current_split_index?)() };
    Some(i)
}
