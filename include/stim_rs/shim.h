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
#include "stim/dem/detector_error_model.h"
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

enum class DemTargetKind : uint8_t;
struct DemTargetOwned;
enum class DemInstructionKind : uint8_t;
struct DemInstructionOwned;
struct DetectorErrorModelInfo;
struct DemDetectorCoordinates;
struct DetectorFinalShift;

struct DetectorErrorModelHandle {
    explicit DetectorErrorModelHandle(stim::DetectorErrorModel dem);
    stim::DetectorErrorModel dem;
};

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

std::unique_ptr<DetectorErrorModelHandle> detector_error_model_from_text(rust::Str text);
std::unique_ptr<DetectorErrorModelHandle> detector_error_model_empty();
std::unique_ptr<DetectorErrorModelHandle> detector_error_model_from_circuit(
    const CircuitHandle &circuit, bool decompose_errors);
std::unique_ptr<DetectorErrorModelHandle> detector_error_model_clone(const DetectorErrorModelHandle &model);
std::unique_ptr<DetectorErrorModelHandle> detector_error_model_add(
    const DetectorErrorModelHandle &lhs, const DetectorErrorModelHandle &rhs);
std::unique_ptr<DetectorErrorModelHandle> detector_error_model_mul(const DetectorErrorModelHandle &model, uint64_t reps);
std::unique_ptr<DetectorErrorModelHandle> detector_error_model_without_tags(const DetectorErrorModelHandle &model);
std::unique_ptr<DetectorErrorModelHandle> detector_error_model_flattened(const DetectorErrorModelHandle &model);
std::unique_ptr<DetectorErrorModelHandle> detector_error_model_rounded(const DetectorErrorModelHandle &model, uint8_t digits);
uint64_t detector_error_model_num_detectors(const DetectorErrorModelHandle &model);
uint64_t detector_error_model_num_observables(const DetectorErrorModelHandle &model);
uint64_t detector_error_model_total_detector_shift(const DetectorErrorModelHandle &model);
rust::String detector_error_model_str(const DetectorErrorModelHandle &model);
DetectorErrorModelInfo detector_error_model_instructions(const DetectorErrorModelHandle &model);
DetectorErrorModelInfo detector_error_model_flattened_instructions(const DetectorErrorModelHandle &model);
bool detector_error_model_eq(const DetectorErrorModelHandle &lhs, const DetectorErrorModelHandle &rhs);
bool detector_error_model_approx_eq(
    const DetectorErrorModelHandle &lhs, const DetectorErrorModelHandle &rhs, double atol);
rust::Vec<DemDetectorCoordinates> detector_error_model_get_detector_coordinates(
    const DetectorErrorModelHandle &model, rust::Vec<uint64_t> included);
DetectorFinalShift detector_error_model_final_detector_and_coord_shift(
    const DetectorErrorModelHandle &model);
rust::String detector_error_model_layout_str(const DetectorErrorModelHandle &model);
rust::String detector_error_model_hint_str(const DetectorErrorModelHandle &model);

void detector_error_model_append_error(
    DetectorErrorModelHandle &model, double probability, rust::Slice<const DemTargetOwned> targets, rust::Str tag);
void detector_error_model_append_shift_detectors(
    DetectorErrorModelHandle &model, rust::Slice<const double> coord_shift, uint64_t detector_shift, rust::Str tag);
void detector_error_model_append_detector(
    DetectorErrorModelHandle &model, rust::Slice<const double> coords, const DemTargetOwned &target, rust::Str tag);
void detector_error_model_append_logical_observable(
    DetectorErrorModelHandle &model, const DemTargetOwned &target, rust::Str tag);
void detector_error_model_append_repeat_block(
    DetectorErrorModelHandle &model, uint64_t repeat_count, const DetectorErrorModelHandle &body, rust::Str tag);
void detector_error_model_append_from_text(DetectorErrorModelHandle &model, rust::Str text);
void detector_error_model_append_from_file(DetectorErrorModelHandle &model, rust::Str path);

DemTargetOwned dem_target_relative_detector(uint64_t id);
DemTargetOwned dem_target_observable(uint64_t id);
DemTargetOwned dem_target_separator();
DemTargetOwned dem_target_shift(const DemTargetOwned &target, int64_t offset);

uint64_t sampler_count_measurements(const SamplerHandle &sampler);
rust::Vec<uint8_t> sampler_sample(SamplerHandle &sampler, uint64_t shots);
rust::Vec<uint8_t> sampler_sample_bit_packed(SamplerHandle &sampler, uint64_t shots);
void sampler_reseed(SamplerHandle &sampler, uint64_t seed);

}  // namespace stimrs
