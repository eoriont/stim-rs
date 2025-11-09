use fusion_blossom::{
    mwpm_solver::LegacySolverSerial,
    util::{SolverInitializer, SyndromePattern, VertexIndex, Weight},
};
use std::collections::HashMap;
use std::convert::TryFrom;
use stim_rs::{Circuit, DemError, DemOptions, DetectorErrorModelFlat, SurfaceCodeConfig};

#[derive(Clone, Copy)]
struct EdgeAccum {
    probability: f64,
    mask: u64,
}

fn combine_prob(existing: f64, new_p: f64) -> f64 {
    existing * (1.0 - new_p) + (1.0 - existing) * new_p
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let circuit = Circuit::surface_code(SurfaceCodeConfig {
        rounds: 4,
        distance: 3,
        after_clifford_depolarization: 1e-3,
        before_round_data_depolarization: 1e-3,
        before_measure_flip_probability: 1e-3,
        after_reset_flip_probability: 1e-3,
        ..Default::default()
    })?;

    let dem = circuit.detector_error_model(DemOptions::default())?;
    println!(
        "Surface code: {} detectors, {} observables, {} error mechanisms",
        dem.num_detectors,
        dem.num_observables,
        dem.errors.len()
    );

    let shots: usize = std::env::args()
        .nth(1)
        .and_then(|s| s.parse().ok())
        .unwrap_or(10_000);

    let detection_batch = circuit.sample_detection_events(shots, Some(0xC0FFEE))?;
    let (initializer, edge_masks) = build_decoding_graph(&dem)?;
    let mut solver = LegacySolverSerial::new(&initializer);
    let mut defect_buffer = Vec::<usize>::new();
    let mut logical_failures = 0usize;
    for shot in 0..detection_batch.shots() {
        detection_batch.collect_defects(shot, &mut defect_buffer);
        let actual_mask = detection_batch.observable_mask_u64(shot)?;
        let syndrome_vertices: Vec<VertexIndex> = defect_buffer
            .iter()
            .map(|&idx| idx as VertexIndex)
            .collect();
        let syndrome = SyndromePattern::new_vertices(syndrome_vertices);
        let used_edges = solver.solve_subgraph(&syndrome);
        let mut predicted_mask = 0u64;
        for edge_idx in used_edges.iter().copied() {
            predicted_mask ^= edge_masks[edge_idx as usize];
        }
        if predicted_mask != actual_mask {
            logical_failures += 1;
        }
    }

    let ler = logical_failures as f64 / shots as f64;
    println!("Simulated {shots} shots; logical error rate ≈ {:.4e}", ler);

    Ok(())
}

fn build_decoding_graph(
    dem: &DetectorErrorModelFlat,
) -> Result<(SolverInitializer, Vec<u64>), Box<dyn std::error::Error>> {
    let num_detectors = usize::try_from(dem.num_detectors)?;
    let num_observables = usize::try_from(dem.num_observables)?;
    let default_boundary = num_detectors + num_observables;
    let total_vertices = default_boundary + 1;

    let virtual_vertices: Vec<VertexIndex> = (num_detectors..total_vertices).collect();

    let mut accum: HashMap<(usize, usize), EdgeAccum> = HashMap::new();

    for error in dem.errors.iter() {
        if !(0.0 < error.probability && error.probability < 1.0) {
            continue;
        }
        let mask = observables_mask(error)?;
        match error.detectors.len() {
            2 => {
                let a = usize::try_from(error.detectors[0])?;
                let b = usize::try_from(error.detectors[1])?;
                accumulate_edge(&mut accum, a, b, error.probability, mask);
            }
            1 => {
                let detector = usize::try_from(error.detectors[0])?;
                let boundary = if let Some(&obs) = error.observables.first() {
                    num_detectors + usize::try_from(obs)?
                } else {
                    default_boundary
                };
                accumulate_edge(&mut accum, detector, boundary, error.probability, mask);
            }
            _ => continue,
        }
    }

    let mut weighted_edges = Vec::with_capacity(accum.len());
    let mut edge_masks = Vec::with_capacity(accum.len());
    for ((a, b), data) in accum.into_iter() {
        let weight = probability_to_weight(data.probability);
        weighted_edges.push((a, b, weight));
        edge_masks.push(data.mask);
    }

    let initializer = SolverInitializer {
        vertex_num: total_vertices,
        weighted_edges,
        virtual_vertices,
    };
    Ok((initializer, edge_masks))
}

fn accumulate_edge(
    accum: &mut HashMap<(usize, usize), EdgeAccum>,
    a: usize,
    b: usize,
    probability: f64,
    mask: u64,
) {
    let key = if a < b { (a, b) } else { (b, a) };
    accum
        .entry(key)
        .and_modify(|entry| {
            if entry.mask != mask {
                panic!(
                    "Logical mask mismatch for edge ({}, {}): {:b} vs {:b}",
                    key.0, key.1, entry.mask, mask
                );
            }
            entry.probability = combine_prob(entry.probability, probability);
        })
        .or_insert(EdgeAccum { probability, mask });
}

fn observables_mask(error: &DemError) -> Result<u64, Box<dyn std::error::Error>> {
    let mut mask = 0u64;
    for &obs in error.observables.iter() {
        if obs >= 64 {
            return Err(format!("observable {obs} exceeds 64-bit mask capacity").into());
        }
        mask ^= 1u64 << obs;
    }
    Ok(mask)
}

fn probability_to_weight(p: f64) -> Weight {
    const SCALE: f64 = 1000.0;
    let w = ((1.0 - p) / p).ln() * SCALE;
    let clamped = w.max(1.0).min((Weight::MAX / 2) as f64);
    let value = clamped.round() as Weight;
    (value * 2).max(2)
}
