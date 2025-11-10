//! Safe Rust bindings for the [Stim](https://github.com/quantumlib/Stim) stabilizer circuit simulator.

mod ffi;

mod circuit;
mod dem;
mod error;
mod sampler;
mod util;

pub use circuit::{
    Circuit, DetectionEventBatch, ReferenceSigns, ReferenceSignsPacked, SurfaceCodeConfig,
};
pub use dem::{
    DemError, DemInstruction, DemInstructionKind, DemOptions, DemTarget, DemTargetKind,
    DetectorErrorModel, DetectorErrorModelFlat, RepeatBlock,
};
pub use error::StimError;
pub use sampler::{Sampler, SamplerOptions};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_and_stats() -> Result<(), StimError> {
        let circuit = Circuit::from_text("H 0\nCX 0 1\nM 0 1")?;
        assert_eq!(circuit.count_qubits()?, 2);
        assert_eq!(circuit.count_measurements()?, 2);
        Ok(())
    }

    #[test]
    fn reference_sample_basic() -> Result<(), StimError> {
        let circuit = Circuit::from_text("X 1\nM 0 1")?;
        let sample = circuit.reference_sample()?;
        assert_eq!(sample, vec![false, true]);
        Ok(())
    }

    #[test]
    fn sampler_deterministic() -> Result<(), StimError> {
        let circuit = Circuit::from_text("X 0\nM 0")?;
        let mut sampler = Sampler::new(&circuit, SamplerOptions::default())?;
        let shots = sampler.sample(4)?;
        assert_eq!(shots.len(), 4);
        assert!(shots.iter().all(|shot| shot == &vec![true]));
        Ok(())
    }

    #[test]
    fn surface_code_generation() -> Result<(), StimError> {
        let circuit = Circuit::surface_code(SurfaceCodeConfig {
            rounds: 2,
            distance: 3,
            ..Default::default()
        })?;
        assert!(circuit.count_measurements()? > 0);
        Ok(())
    }

    #[test]
    fn detector_error_model_basic() -> Result<(), StimError> {
        let circuit = Circuit::from_text("R 0\nM 0\nDETECTOR rec[-1]")?;
        let dem = DetectorErrorModel::from_circuit(&circuit, DemOptions::default())?;
        let instructions = dem.instructions()?;
        assert!(!instructions.is_empty());
        Ok(())
    }

    #[test]
    fn detector_error_model_mutation() -> Result<(), StimError> {
        let mut dem = DetectorErrorModel::new();
        dem.append_error(0.25, &[DemTarget::relative_detector(0)], "")?;
        dem.append_shift_detectors(&[1.0, 2.0], 1, "")?;
        dem.append_detector(&[1.0, 2.0, 3.0], &DemTarget::relative_detector(0), "")?;
        let _det = DemTarget::relative_detector(0);
        let obs = DemTarget::observable(0);
        dem.append_logical_observable(&obs, "")?;
        let instructions = dem.instructions()?;
        assert_eq!(instructions.len(), 4);
        let groups = instructions[0].separated_target_groups();
        assert_eq!(groups.len(), 1);
        assert_eq!(groups[0][0], DemTarget::relative_detector(0));
        Ok(())
    }

    #[test]
    fn detector_error_model_arithmetic() -> Result<(), StimError> {
        let dem_a = DetectorErrorModel::from_text("error(0.1) D0\n")?;
        let dem_b = DetectorErrorModel::from_text("error(0.2) L0\n")?;
        let added = (&dem_a + &dem_b).flattened();
        assert!(added.to_stim_string().contains("error"));
        let repeated = (&dem_a * 3).flattened();
        assert!(repeated.count_detectors()? >= dem_a.count_detectors()?);
        let rounded = dem_a.rounded(1);
        assert!(rounded.approx_equals(&dem_a, 1e-3));
        assert!(dem_a == dem_a.without_tags());
        assert!((&dem_a + &dem_b) != dem_a);
        Ok(())
    }

    #[test]
    fn detector_error_model_flatten_iter() -> Result<(), StimError> {
        let dem =
            DetectorErrorModel::from_text("error(0.1) D0\nrepeat 2 {\n error(0.2) D1 D2\n}\n")?;
        let flattened = dem.flattened_error_instructions()?;
        assert_eq!(flattened.len(), 3);
        Ok(())
    }

    #[test]
    fn coordinate_queries() -> Result<(), StimError> {
        let dem = DetectorErrorModel::from_text(
            "shift_detectors(0, 0) 1\nerror(0.1) D0\nerror(0.2) D1\n",
        )?;
        let coords = dem.detector_coordinates(&[0, 1]);
        assert!(coords.contains_key(&0));
        let (_shift, coord_shift) = dem.final_detector_and_coord_shift();
        assert_eq!(coord_shift.len(), 2);
        assert!(!dem.layout_str().is_empty());
        Ok(())
    }
}
