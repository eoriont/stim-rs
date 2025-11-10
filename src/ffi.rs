#[cxx::bridge(namespace = "stimrs")]
mod ffi {
    unsafe extern "C++" {
        include!("stim_rs/shim.h");

        type CircuitHandle;
        type SamplerHandle;
        type DetectorErrorModelHandle;
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
        fn detector_error_model_from_text(
            text: &str,
        ) -> Result<UniquePtr<DetectorErrorModelHandle>>;
        fn detector_error_model_empty() -> UniquePtr<DetectorErrorModelHandle>;
        fn detector_error_model_from_circuit(
            circuit: &CircuitHandle,
            decompose_errors: bool,
        ) -> Result<UniquePtr<DetectorErrorModelHandle>>;
        fn detector_error_model_clone(
            model: &DetectorErrorModelHandle,
        ) -> UniquePtr<DetectorErrorModelHandle>;
        fn detector_error_model_add(
            lhs: &DetectorErrorModelHandle,
            rhs: &DetectorErrorModelHandle,
        ) -> UniquePtr<DetectorErrorModelHandle>;
        fn detector_error_model_mul(
            model: &DetectorErrorModelHandle,
            reps: u64,
        ) -> UniquePtr<DetectorErrorModelHandle>;
        fn detector_error_model_without_tags(
            model: &DetectorErrorModelHandle,
        ) -> UniquePtr<DetectorErrorModelHandle>;
        fn detector_error_model_flattened(
            model: &DetectorErrorModelHandle,
        ) -> UniquePtr<DetectorErrorModelHandle>;
        fn detector_error_model_rounded(
            model: &DetectorErrorModelHandle,
            digits: u8,
        ) -> UniquePtr<DetectorErrorModelHandle>;
        fn detector_error_model_num_detectors(model: &DetectorErrorModelHandle) -> u64;
        fn detector_error_model_num_observables(model: &DetectorErrorModelHandle) -> u64;
        fn detector_error_model_total_detector_shift(model: &DetectorErrorModelHandle) -> u64;
        fn detector_error_model_str(model: &DetectorErrorModelHandle) -> String;
        fn detector_error_model_instructions(
            model: &DetectorErrorModelHandle,
        ) -> DetectorErrorModelInfo;
        fn detector_error_model_flattened_instructions(
            model: &DetectorErrorModelHandle,
        ) -> DetectorErrorModelInfo;
        fn detector_error_model_eq(
            lhs: &DetectorErrorModelHandle,
            rhs: &DetectorErrorModelHandle,
        ) -> bool;
        fn detector_error_model_approx_eq(
            lhs: &DetectorErrorModelHandle,
            rhs: &DetectorErrorModelHandle,
            atol: f64,
        ) -> bool;
        fn detector_error_model_append_error(
            model: Pin<&mut DetectorErrorModelHandle>,
            probability: f64,
            targets: &[DemTargetOwned],
            tag: &str,
        );
        fn detector_error_model_append_shift_detectors(
            model: Pin<&mut DetectorErrorModelHandle>,
            coord_shift: &[f64],
            detector_shift: u64,
            tag: &str,
        );
        fn detector_error_model_append_detector(
            model: Pin<&mut DetectorErrorModelHandle>,
            coords: &[f64],
            target: &DemTargetOwned,
            tag: &str,
        );
        fn detector_error_model_append_logical_observable(
            model: Pin<&mut DetectorErrorModelHandle>,
            target: &DemTargetOwned,
            tag: &str,
        );
        fn detector_error_model_append_repeat_block(
            model: Pin<&mut DetectorErrorModelHandle>,
            repeat_count: u64,
            body: &DetectorErrorModelHandle,
            tag: &str,
        );
        fn detector_error_model_append_from_text(
            model: Pin<&mut DetectorErrorModelHandle>,
            text: &str,
        );
        fn detector_error_model_append_from_file(
            model: Pin<&mut DetectorErrorModelHandle>,
            path: &str,
        );
        fn dem_target_relative_detector(id: u64) -> DemTargetOwned;
        fn dem_target_observable(id: u64) -> DemTargetOwned;
        fn dem_target_separator() -> DemTargetOwned;
        fn dem_target_shift(target: &DemTargetOwned, offset: i64) -> DemTargetOwned;
        fn detector_error_model_get_detector_coordinates(
            model: &DetectorErrorModelHandle,
            included: Vec<u64>,
        ) -> Vec<DemDetectorCoordinates>;
        fn detector_error_model_final_detector_and_coord_shift(
            model: &DetectorErrorModelHandle,
        ) -> DetectorFinalShift;
        fn detector_error_model_layout_str(model: &DetectorErrorModelHandle) -> String;
        fn detector_error_model_hint_str(model: &DetectorErrorModelHandle) -> String;
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

    enum DemTargetKind {
        RELATIVE_DETECTOR_ID = 0,
        OBSERVABLE_ID = 1,
        SEPARATOR = 2,
    }

    struct DemTargetOwned {
        kind: DemTargetKind,
        value: u64,
    }

    enum DemInstructionKind {
        ERROR = 0,
        SHIFT_DETECTORS = 1,
        DETECTOR = 2,
        LOGICAL_OBSERVABLE = 3,
        REPEAT_BLOCK = 4,
    }

    struct DemInstructionOwned {
        kind: DemInstructionKind,
        args: Vec<f64>,
        targets: Vec<DemTargetOwned>,
        tag: Vec<u8>,
        repeat_count: u64,
        body: Vec<DemInstructionOwned>,
    }

    struct DetectorErrorModelInfo {
        instructions: Vec<DemInstructionOwned>,
    }

    struct DemDetectorCoordinates {
        detector: u64,
        coords: Vec<f64>,
    }

    struct DetectorFinalShift {
        detector_shift: u64,
        coord_shift: Vec<f64>,
    }
}

pub(crate) use ffi::*;
