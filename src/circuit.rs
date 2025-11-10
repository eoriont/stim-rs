use crate::dem::{DemOptions, DetectorErrorModelFlat};
use crate::ffi;
use crate::util::{bool_vec_from_u8, to_u64, to_usize};
use crate::StimError;
use cxx::UniquePtr;
use std::fmt;

/// A parsed Stim circuit.
pub struct Circuit {
    pub(crate) inner: UniquePtr<ffi::CircuitHandle>,
}

impl Circuit {
    /// Parses a circuit from the provided text.
    pub fn from_text(src: &str) -> Result<Self, StimError> {
        let inner = ffi::circuit_from_text(src)?;
        Ok(Self { inner })
    }

    /// Generates a surface code memory circuit using Stim's built-in templates.
    pub fn surface_code(config: SurfaceCodeConfig) -> Result<Self, StimError> {
        let inner = ffi::circuit_surface_code(
            config.rounds,
            config.distance,
            &config.task,
            config.after_clifford_depolarization,
            config.before_round_data_depolarization,
            config.before_measure_flip_probability,
            config.after_reset_flip_probability,
        )?;
        Ok(Self { inner })
    }

    /// Returns the number of qubits referenced by the circuit.
    pub fn count_qubits(&self) -> Result<usize, StimError> {
        to_usize(ffi::circuit_count_qubits(self.as_ref()))
    }

    /// Returns the number of measurement results produced by the circuit.
    pub fn count_measurements(&self) -> Result<usize, StimError> {
        to_usize(ffi::circuit_count_measurements(self.as_ref()))
    }

    /// Returns the number of detectors declared by the circuit.
    pub fn count_detectors(&self) -> Result<usize, StimError> {
        to_usize(ffi::circuit_count_detectors(self.as_ref()))
    }

    /// Returns the number of observables declared by the circuit.
    pub fn count_observables(&self) -> Result<usize, StimError> {
        to_usize(ffi::circuit_count_observables(self.as_ref()))
    }

    /// Renders the circuit back to text using Stim's canonical formatting.
    pub fn to_stim_string(&self) -> String {
        ffi::circuit_to_string(self.as_ref())
    }

    /// Computes a deterministic noiseless measurement sample.
    pub fn reference_sample(&self) -> Result<Vec<bool>, StimError> {
        Ok(bool_vec_from_u8(ffi::circuit_reference_sample(
            self.as_ref(),
            false,
        )?))
    }

    /// Returns the packed noiseless measurement sample (little endian, 8 bits per byte).
    pub fn reference_sample_bit_packed(&self) -> Result<Vec<u8>, StimError> {
        Ok(ffi::circuit_reference_sample(self.as_ref(), true)?)
    }

    /// Returns the deterministic detector and observable signs for the circuit.
    pub fn reference_signs(&self) -> Result<ReferenceSigns, StimError> {
        let raw = ffi::circuit_reference_signs(self.as_ref(), false)?;
        Ok(ReferenceSigns {
            detectors: bool_vec_from_u8(raw.detectors),
            observables: bool_vec_from_u8(raw.observables),
        })
    }

    /// Returns bit-packed detector and observable signs (little endian).
    pub fn reference_signs_bit_packed(&self) -> Result<ReferenceSignsPacked, StimError> {
        let raw = ffi::circuit_reference_signs(self.as_ref(), true)?;
        Ok(ReferenceSignsPacked {
            detectors: raw.detectors,
            observables: raw.observables,
        })
    }

    /// Returns a flattened detector error model describing how errors create detection events.
    pub fn detector_error_model(
        &self,
        options: DemOptions,
    ) -> Result<DetectorErrorModelFlat, StimError> {
        Ok(ffi::circuit_detector_error_model(self.as_ref(), options.decompose_errors)?.into())
    }

    /// Samples detection events and observable flips produced by running the circuit.
    pub fn sample_detection_events(
        &self,
        shots: usize,
        seed: Option<u64>,
    ) -> Result<DetectionEventBatch, StimError> {
        let (seed_value, use_seed) = match seed {
            Some(value) => (value, true),
            None => (0, false),
        };
        let raw = ffi::circuit_sample_detection_events(
            self.as_ref(),
            to_u64(shots)?,
            seed_value,
            use_seed,
        )?;
        DetectionEventBatch::try_from(raw)
    }

    pub(crate) fn as_ref(&self) -> &ffi::CircuitHandle {
        self.inner
            .as_ref()
            .expect("Circuit handle unexpectedly null (internal bug)")
    }
}

impl Clone for Circuit {
    fn clone(&self) -> Self {
        Self {
            inner: ffi::circuit_clone(self.as_ref()),
        }
    }
}

impl fmt::Debug for Circuit {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple("Circuit")
            .field(&self.to_stim_string())
            .finish()
    }
}

/// Reference detector and observable parities.
pub struct ReferenceSigns {
    pub detectors: Vec<bool>,
    pub observables: Vec<bool>,
}

/// Bit-packed reference detector and observable parities.
pub struct ReferenceSignsPacked {
    pub detectors: Vec<u8>,
    pub observables: Vec<u8>,
}

/// A batch of sampled detection events from running a circuit.
#[derive(Debug)]
pub struct DetectionEventBatch {
    detector_data: Vec<u8>,
    observable_data: Vec<u8>,
    num_detectors: usize,
    num_observables: usize,
    shots: usize,
    det_row_bytes: usize,
    obs_row_bytes: usize,
}

impl TryFrom<ffi::SampledDetectionEvents> for DetectionEventBatch {
    type Error = StimError;

    fn try_from(raw: ffi::SampledDetectionEvents) -> Result<Self, Self::Error> {
        let num_detectors = to_usize(raw.num_detectors)?;
        let num_observables = to_usize(raw.num_observables)?;
        let shots = to_usize(raw.shots)?;
        let det_row_bytes = if num_detectors == 0 {
            0
        } else {
            (num_detectors + 7) / 8
        };
        let obs_row_bytes = if num_observables == 0 {
            0
        } else {
            (num_observables + 7) / 8
        };
        if det_row_bytes.saturating_mul(shots) != raw.detector_data.len() {
            return Err(StimError::Conversion("detector data size mismatch".into()));
        }
        if obs_row_bytes.saturating_mul(shots) != raw.observable_data.len() {
            return Err(StimError::Conversion(
                "observable data size mismatch".into(),
            ));
        }
        Ok(Self {
            detector_data: raw.detector_data,
            observable_data: raw.observable_data,
            num_detectors,
            num_observables,
            shots,
            det_row_bytes,
            obs_row_bytes,
        })
    }
}

impl DetectionEventBatch {
    pub fn shots(&self) -> usize {
        self.shots
    }

    fn assert_shot(&self, shot: usize) {
        assert!(
            shot < self.shots,
            "shot index {shot} out of range {}",
            self.shots
        );
    }

    fn detector_row(&self, shot: usize) -> &[u8] {
        if self.det_row_bytes == 0 {
            return &[];
        }
        let start = shot * self.det_row_bytes;
        &self.detector_data[start..start + self.det_row_bytes]
    }

    fn observable_row(&self, shot: usize) -> &[u8] {
        if self.obs_row_bytes == 0 {
            return &[];
        }
        let start = shot * self.obs_row_bytes;
        &self.observable_data[start..start + self.obs_row_bytes]
    }

    /// Collects detector indices that fired for the given shot.
    pub fn collect_defects(&self, shot: usize, out: &mut Vec<usize>) {
        self.assert_shot(shot);
        out.clear();
        for (byte_index, &byte) in self.detector_row(shot).iter().enumerate() {
            if byte == 0 {
                continue;
            }
            for bit in 0..8 {
                let detector = byte_index * 8 + bit;
                if detector >= self.num_detectors {
                    break;
                }
                if (byte >> bit) & 1 == 1 {
                    out.push(detector);
                }
            }
        }
    }

    /// Returns the observable flip mask for a shot (supports up to 64 observables).
    pub fn observable_mask_u64(&self, shot: usize) -> Result<u64, StimError> {
        self.assert_shot(shot);
        if self.num_observables > 64 {
            return Err(StimError::Conversion(
                "observable mask exceeds 64 bits; use observable_row() instead".into(),
            ));
        }
        let mut mask = 0u64;
        for (byte_index, &byte) in self.observable_row(shot).iter().enumerate() {
            if byte == 0 {
                continue;
            }
            for bit in 0..8 {
                let obs = byte_index * 8 + bit;
                if obs >= self.num_observables {
                    break;
                }
                if (byte >> bit) & 1 == 1 {
                    mask |= 1u64 << obs;
                }
            }
        }
        Ok(mask)
    }
}

/// Parameters used to generate a surface code circuit.
#[derive(Debug, Clone)]
pub struct SurfaceCodeConfig {
    pub rounds: u64,
    pub distance: u32,
    pub task: String,
    pub after_clifford_depolarization: f64,
    pub before_round_data_depolarization: f64,
    pub before_measure_flip_probability: f64,
    pub after_reset_flip_probability: f64,
}

impl Default for SurfaceCodeConfig {
    fn default() -> Self {
        Self {
            rounds: 3,
            distance: 3,
            task: "rotated_memory_x".to_string(),
            after_clifford_depolarization: 0.0,
            before_round_data_depolarization: 0.0,
            before_measure_flip_probability: 0.0,
            after_reset_flip_probability: 0.0,
        }
    }
}
