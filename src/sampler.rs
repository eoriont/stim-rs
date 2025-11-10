use crate::circuit::Circuit;
use crate::ffi;
use crate::util::{matrix_from_flat, to_u64};
use crate::StimError;
use cxx::UniquePtr;
use std::fmt;

/// Options used when constructing a measurement sampler.
#[derive(Debug, Clone)]
pub struct SamplerOptions {
    /// When true, reference samples are skipped and returned data is measurement flips.
    pub skip_reference_sample: bool,
    /// Optional random seed overriding Stim's default RNG seeding.
    pub seed: Option<u64>,
}

impl Default for SamplerOptions {
    fn default() -> Self {
        Self {
            skip_reference_sample: false,
            seed: None,
        }
    }
}

/// A compiled circuit sampler that can generate measurement shots efficiently.
pub struct Sampler {
    inner: UniquePtr<ffi::SamplerHandle>,
    num_measurements: usize,
}

impl Sampler {
    /// Compiles a sampler for the provided circuit using the given options.
    pub fn new(circuit: &Circuit, options: SamplerOptions) -> Result<Self, StimError> {
        let num_measurements = circuit.count_measurements()?;
        let (seed, use_seed) = match options.seed {
            Some(value) => (value, true),
            None => (0, false),
        };
        let inner = ffi::sampler_create(
            circuit.as_ref(),
            options.skip_reference_sample,
            seed,
            use_seed,
        )?;
        Ok(Self {
            inner,
            num_measurements,
        })
    }

    /// Returns the number of measurements produced per shot.
    pub fn num_measurements(&self) -> usize {
        self.num_measurements
    }

    /// Re-seeds the sampler's RNG.
    pub fn reseed(&mut self, seed: u64) {
        ffi::sampler_reseed(self.inner.pin_mut(), seed);
    }

    /// Samples the circuit `shots` times and returns a `shots x num_measurements` boolean matrix.
    pub fn sample(&mut self, shots: usize) -> Result<Vec<Vec<bool>>, StimError> {
        let raw = ffi::sampler_sample(self.inner.pin_mut(), to_u64(shots)?)?;
        Ok(matrix_from_flat(raw, shots, self.num_measurements))
    }

    /// Samples the circuit `shots` times and returns the bit-packed results.
    ///
    /// The output is arranged as `shots` consecutive rows, each containing
    /// `ceil(num_measurements / 8)` bytes in little-endian bit order.
    pub fn sample_bit_packed(&mut self, shots: usize) -> Result<Vec<u8>, StimError> {
        Ok(ffi::sampler_sample_bit_packed(
            self.inner.pin_mut(),
            to_u64(shots)?,
        )?)
    }
}

impl fmt::Debug for Sampler {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Sampler")
            .field("num_measurements", &self.num_measurements)
            .finish()
    }
}
