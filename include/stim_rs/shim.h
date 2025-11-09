#pragma once

#include "rust/cxx.h"

#include <cstdint>
#include <memory>
#include <optional>
#include <random>
#include <string>

#ifdef __clang__
#pragma clang diagnostic push
#pragma clang diagnostic ignored "-Wdeprecated-copy-with-user-provided-copy"
#pragma clang diagnostic ignored "-Wunused-parameter"
#endif

#include "stim/circuit/circuit.h"
#include "stim/mem/simd_bits.h"
#include "stim/gen/gen_surface_code.h"
#include "stim/util_top/circuit_to_dem.h"

#ifdef __clang__
#pragma clang diagnostic pop
#endif

namespace stimrs {

struct CircuitHandle {
    explicit CircuitHandle(stim::Circuit circuit);
    stim::Circuit circuit;
};

struct SamplerHandle {
    SamplerHandle(
        stim::Circuit circuit,
        stim::simd_bits<stim::MAX_BITWORD_WIDTH> ref_sample,
        bool skip_reference_sample,
        std::mt19937_64 rng);

    stim::Circuit circuit;
    stim::simd_bits<stim::MAX_BITWORD_WIDTH> reference_sample;
    bool skip_reference_sample;
    std::mt19937_64 rng;
};

struct ReferenceSignsRaw;
struct DemError;
struct DetectorErrorModelFlat;
struct SampledDetectionEvents;

std::unique_ptr<CircuitHandle> circuit_from_text(rust::Str text);
std::unique_ptr<CircuitHandle> circuit_clone(const CircuitHandle &circuit);
uint64_t circuit_count_qubits(const CircuitHandle &circuit);
uint64_t circuit_count_measurements(const CircuitHandle &circuit);
uint64_t circuit_count_detectors(const CircuitHandle &circuit);
uint64_t circuit_count_observables(const CircuitHandle &circuit);
rust::String circuit_to_string(const CircuitHandle &circuit);
rust::Vec<uint8_t> circuit_reference_sample(const CircuitHandle &circuit, bool bit_packed);
ReferenceSignsRaw circuit_reference_signs(const CircuitHandle &circuit, bool bit_packed);
std::unique_ptr<CircuitHandle> circuit_surface_code(
    uint64_t rounds,
    uint32_t distance,
    rust::Str task,
    double after_clifford_depolarization,
    double before_round_data_depolarization,
    double before_measure_flip_probability,
    double after_reset_flip_probability);

std::unique_ptr<SamplerHandle> sampler_create(
    const CircuitHandle &circuit,
    bool skip_reference_sample,
    uint64_t seed,
    bool use_seed);
DetectorErrorModelFlat circuit_detector_error_model(const CircuitHandle &circuit, bool decompose_errors);
SampledDetectionEvents circuit_sample_detection_events(
    const CircuitHandle &circuit, uint64_t shots, uint64_t seed, bool use_seed);
uint64_t sampler_count_measurements(const SamplerHandle &sampler);
rust::Vec<uint8_t> sampler_sample(SamplerHandle &sampler, uint64_t shots);
rust::Vec<uint8_t> sampler_sample_bit_packed(SamplerHandle &sampler, uint64_t shots);
void sampler_reseed(SamplerHandle &sampler, uint64_t seed);

}  // namespace stimrs
