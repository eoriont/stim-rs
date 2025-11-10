use stim_rs::{DemOptions, DemTarget, DetectorErrorModel};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut dem = DetectorErrorModel::new();
    dem.append_error(
        0.001,
        &[
            DemTarget::relative_detector(0),
            DemTarget::relative_detector(1),
        ],
        "bulk",
    )?;
    dem.append_shift_detectors(&[1.0, 0.0], 1, "move")?;
    dem.append_detector(&[1.0, 0.5], &DemTarget::relative_detector(0), "mark")?;
    dem.append_logical_observable(&DemTarget::observable(0), "L0")?;

    println!("Custom DEM:\n{}", dem.to_stim_string());

    let flattened = dem.flattened();
    println!(
        "Flattened copy has {} detectors and {} observables",
        flattened.count_detectors()?,
        flattened.count_observables()?
    );

    let coords = flattened.detector_coordinates(&[0, 1]);
    println!("Detector coordinates: {coords:?}");
    println!("Layout:\n{}", flattened.layout_str());
    println!("Hint string:\n{}", flattened.hint_str());

    // Combine with a DEM extracted from a small deterministic circuit.
    let circuit = stim_rs::Circuit::from_text("R 0\nM 0\nDETECTOR rec[-1]")?;
    let from_circuit = DetectorErrorModel::from_circuit(
        &circuit,
        DemOptions {
            decompose_errors: true,
        },
    )?;
    let merged = (&dem + &from_circuit).flattened();
    println!(
        "Merged DEM has {} instructions after flattening",
        merged.flattened_error_instructions()?.len()
    );

    Ok(())
}
