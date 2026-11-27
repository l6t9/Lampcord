use std::time::Duration;

#[derive(Clone, Debug)]
pub struct Backoff {
    min: u64,
    max: u64,
    current: u64,
    fails: u32,
}

impl Backoff {
    pub fn new() -> Self {
        Self::with_range(500, None)
    }

    pub fn with_range(min_ms: u64, max_ms: Option<u64>) -> Self {
        Self {
            min: min_ms,
            max: max_ms.unwrap_or(min_ms * 10),
            current: min_ms,
            fails: 0,
        }
    }

    pub fn fails(&self) -> u32 {
        self.fails
    }

    pub fn succeed(&mut self) {
        self.fails = 0;
        self.current = self.min;
    }

    pub fn fail(&mut self, minimum: Option<Duration>) -> Duration {
        self.fail_with(minimum, random_unit())
    }

    fn fail_with(&mut self, minimum: Option<Duration>, random: f64) -> Duration {
        self.fails += 1;
        let growth = 2.0 * self.current as f64 * random;
        self.current = ((self.current as f64 + growth) as u64).min(self.max);
        let current = Duration::from_millis(self.current);
        match minimum {
            Some(minimum) if minimum > current => minimum,
            _ => current,
        }
    }
}

impl Default for Backoff {
    fn default() -> Self {
        Self::new()
    }
}

fn random_unit() -> f64 {
    (uuid::Uuid::new_v4().as_u128() >> 64) as f64 / u64::MAX as f64
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_grows_by_twice_the_current_value_times_the_random_factor_up_to_the_cap() {
        let mut b = Backoff::new();
        assert_eq!(
            b.fail_with(None, 1.0).as_millis(),
            1_500,
            "500 + 2 x 500 x 1"
        );
        assert_eq!(b.fail_with(None, 1.0).as_millis(), 4_500);
        assert_eq!(
            b.fail_with(None, 1.0).as_millis(),
            5_000,
            "capped at ten times the minimum"
        );
        assert_eq!(b.fail_with(None, 0.0).as_millis(), 5_000);
        assert_eq!(b.fails(), 4);
        b.succeed();
        assert_eq!(b.fail_with(None, 0.0).as_millis(), 500);
    }

    #[test]
    fn a_retry_after_longer_than_the_backoff_wins() {
        let mut b = Backoff::with_range(1_000, Some(60_000));
        assert_eq!(
            b.fail_with(Some(Duration::from_secs(5)), 0.0),
            Duration::from_secs(5)
        );
        assert_eq!(
            b.fail_with(Some(Duration::from_millis(100)), 0.0),
            Duration::from_millis(1_000)
        );
    }

    #[test]
    fn the_random_factor_stays_in_range() {
        for _ in 0..100 {
            let r = random_unit();
            assert!((0.0..=1.0).contains(&r));
        }
    }
}
