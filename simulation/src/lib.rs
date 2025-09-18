/// Abstraction for getting timestamps even when `std::time` isn't supported.
pub enum PortableInstant {
    #[cfg(not(target_arch = "wasm32"))]
    Native(std::time::Instant),

    #[cfg(target_arch = "wasm32")]
    Web {
        performance_timestamp_ms: f64,

        // HACK(eddyb) cached `window().performance()` to speed up/simplify `elapsed`.
        cached_window_performance: web_sys::Performance,
    },
}

impl PortableInstant {
    pub fn now() -> Self {
        #[cfg(not(target_arch = "wasm32"))]
        {
            Self::Native(std::time::Instant::now())
        }
        #[cfg(target_arch = "wasm32")]
        {
            let performance = web_sys::window()
                .expect("missing window")
                .performance()
                .expect("missing window.performance");
            Self::Web {
                performance_timestamp_ms: performance.now(),
                cached_window_performance: performance,
            }
        }
    }

    pub fn elapsed_secs_f32(&self) -> f32 {
        match self {
            #[cfg(not(target_arch = "wasm32"))]
            Self::Native(instant) => instant.elapsed().as_secs_f32(),

            #[cfg(target_arch = "wasm32")]
            Self::Web {
                performance_timestamp_ms,
                cached_window_performance,
            } => ((cached_window_performance.now() - performance_timestamp_ms) / 1000.0) as f32,
        }
    }
}
