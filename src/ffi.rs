#[cxx::bridge(namespace = "stimrs")]
mod ffi {
    unsafe extern "C++" {
        include!("stim_rs/shim.h");

        type CircuitHandle;
        type SamplerHandle;
        fn circuit_from_text(text: &str) -> Result<UniquePtr<CircuitHandle>>;
        fn circuit_clone(circuit: &CircuitHandle) -> UniquePtr<CircuitHandle>;
        fn circuit_count_qubits(circuit: &CircuitHandle) -> u64;
        fn circuit_count_measurements(circuit: &CircuitHandle) -> u64;
        fn circuit_count_detectors(circuit: &CircuitHandle) -> u64;
        fn circuit_count_observables(circuit: &CircuitHandle) -> u64;
        fn circuit_to_string(circuit: &CircuitHandle) -> String;
        fn circuit_reference_sample(circuit: &CircuitHandle, bit_packed: bool) -> Result<Vec<u8>>;
        fn circuit_reference_signs(
            circuit: &CircuitHandle,
            bit_packed: bool,
        ) -> Result<ReferenceSignsRaw>;
        fn circuit_surface_code(
            rounds: u64,
            distance: u32,
            task: &str,
            after_clifford_depolarization: f64,
            before_round_data_depolarization: f64,
            before_measure_flip_probability: f64,
            after_reset_flip_probability: f64,
        ) -> Result<UniquePtr<CircuitHandle>>;

        fn sampler_create(
            circuit: &CircuitHandle,
            skip_reference_sample: bool,
            seed: u64,
            use_seed: bool,
        ) -> Result<UniquePtr<SamplerHandle>>;
        fn sampler_sample(sampler: Pin<&mut SamplerHandle>, shots: u64) -> Result<Vec<u8>>;
        fn sampler_sample_bit_packed(
            sampler: Pin<&mut SamplerHandle>,
            shots: u64,
        ) -> Result<Vec<u8>>;
        fn sampler_reseed(sampler: Pin<&mut SamplerHandle>, seed: u64);
        fn circuit_detector_error_model(
            circuit: &CircuitHandle,
            decompose_errors: bool,
        ) -> Result<DetectorErrorModelFlat>;
        fn circuit_sample_detection_events(
            circuit: &CircuitHandle,
            shots: u64,
            seed: u64,
            use_seed: bool,
        ) -> Result<SampledDetectionEvents>;
    }

    #[derive(Debug)]
    struct ReferenceSignsRaw {
        detectors: Vec<u8>,
        observables: Vec<u8>,
    }

    #[derive(Debug)]
    struct DemError {
        probability: f64,
        detectors: Vec<u64>,
        observables: Vec<u64>,
    }

    #[derive(Debug)]
    struct DetectorErrorModelFlat {
        errors: Vec<DemError>,
        num_detectors: u64,
        num_observables: u64,
    }

    #[derive(Debug)]
    struct SampledDetectionEvents {
        detector_data: Vec<u8>,
        observable_data: Vec<u8>,
        num_detectors: u64,
        num_observables: u64,
        shots: u64,
    }
}

pub(crate) use ffi::*;
