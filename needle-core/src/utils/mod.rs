pub(crate) fn crop<T: PartialOrd>(val: T, max: T) -> T {
    if val < max {
        val
    } else {
        max
    }
}
