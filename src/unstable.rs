/// Accesses the index of the split the attempt is currently on. If there's
/// no attempt in progress, `-1` is returned instead. This returns an
/// index that is equal to the amount of segments when the attempt is
/// finished, but has not been reset. So you need to be careful when using
/// this value for indexing.
pub fn maybe_timer_current_split_index() -> Option<i32> {
    #[cfg(feature = "unstable")]
    return Some(asr::timer::current_split_index().map_or(-1, |i| i as i32));
    #[allow(unreachable_code)]
    None
}
