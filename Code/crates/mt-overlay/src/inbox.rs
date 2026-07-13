//! Padding-бакеты (Этап 2). Адресация ящика (epoch_tag) заменена Montana Unlinkable
//! Queues — см. muq.rs / queue_host.rs. Здесь остаётся только padding (переиспользуется MUQ).

// spec: padding-бакеты {256, 1024, 4096, 16384, 65536, 262144, 1048576} (степени двойки, ×4).
pub const PADDING_BUCKETS: [usize; 7] = [256, 1024, 4096, 16384, 65536, 262144, 1_048_576];

/// Наименьший бакет ≥ n. None если n превышает верхний бакет (1 MiB = MAX_PLAINTEXT).
pub fn bucket_len(n: usize) -> Option<usize> {
    PADDING_BUCKETS.iter().copied().find(|&b| b >= n)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bucket_len_rounds_up_power_of_four() {
        assert_eq!(bucket_len(0), Some(256));
        assert_eq!(bucket_len(256), Some(256));
        assert_eq!(bucket_len(257), Some(1024));
        assert_eq!(bucket_len(1_048_576), Some(1_048_576));
        assert_eq!(bucket_len(1_048_577), None);
    }
}
