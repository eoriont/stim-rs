use stim_rs::{Circuit, Sampler, SamplerOptions};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let circuit = Circuit::from_text(
        "R 0
         R 1
         CX 0 1
         M 0 1
         DETECTOR rec[-1] rec[-2]
         OBSERVABLE_INCLUDE(0) rec[-2]",
    )?;

    println!("Circuit:\n{}", circuit.to_stim_string());
    println!(
        "Qubits: {}, measurements: {}",
        circuit.count_qubits()?,
        circuit.count_measurements()?
    );

    let mut sampler = Sampler::new(&circuit, SamplerOptions::default())?;
    let shots = sampler.sample(4)?;
    for (i, shot) in shots.iter().enumerate() {
        println!("Shot {i}: {:?}", shot);
    }

    let dem = circuit.detector_error_model(Default::default())?;
    println!(
        "DEM has {} detectors and {} observables",
        dem.num_detectors, dem.num_observables
    );

    Ok(())
}
