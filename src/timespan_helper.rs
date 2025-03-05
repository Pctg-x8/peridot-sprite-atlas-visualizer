use windows::Foundation::TimeSpan;

pub const fn timespan_ms(ms: u32) -> TimeSpan {
    TimeSpan {
        Duration: (10_000 * ms) as _,
    }
}
