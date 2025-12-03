#[derive(Debug, Clone, Copy, PartialOrd, PartialEq)]
pub struct DeltaTime {
    second: f64,
}

impl DeltaTime {
    pub fn new(delta: f64) -> DeltaTime {
        DeltaTime { second: delta }
    }

    pub fn as_seconds(&self) -> f64 {
        self.second.max(0.0)
    }

    pub fn to_milliseconds(&self) -> f64 {
        self.second * 1000.0
    }

    pub fn to_microseconds(&self) -> f64 {
        self.second * 1_000_000.0
    }
}