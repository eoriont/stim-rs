#include "stim_rs/shim.h"
#include "stim-rs/src/ffi.rs.h"

#include <limits>
#include <random>
#include <stdexcept>
#include <utility>

#ifdef __clang__
#pragma clang diagnostic push
#pragma clang diagnostic ignored "-Wdeprecated-copy-with-user-provided-copy"
#pragma clang diagnostic ignored "-Wunused-parameter"
#endif

#include "stim/circuit/circuit.h"
#include "stim/mem/bit_ref.h"
#include "stim/mem/simd_bit_table.h"
#include "stim/mem/simd_bits.h"
#include "stim/mem/simd_bits_range_ref.h"
#include "stim/simulators/frame_simulator_util.h"
#include "stim/simulators/tableau_simulator.h"

#ifdef __clang__
#pragma clang diagnostic pop
#endif

using stim::Circuit;
using stim::CircuitInstruction;
using stim::GateType;
using stim::TableauSimulator;

namespace {

template <typename Fn>
auto handle_errors(Fn &&fn) {
    try {
        return fn();
    } catch (...) {
        throw;
    }
}

template <size_t W>
rust::Vec<uint8_t> copy_bits_to_vec(const stim::simd_bits<W> &bits, size_t count, bool bit_packed) {
    rust::Vec<uint8_t> out;
    if (bit_packed) {
        size_t byte_len = (count + 7) / 8;
        out.reserve(byte_len);
        for (size_t byte_index = 0; byte_index < byte_len; ++byte_index) {
            uint8_t byte = 0;
            for (size_t bit = 0; bit < 8; ++bit) {
                size_t idx = (byte_index * 8) + bit;
                if (idx >= count) {
                    break;
                }
                if (bits[idx]) {
                    byte |= static_cast<uint8_t>(1u << bit);
                }
            }
            out.push_back(byte);
        }
    } else {
        out.reserve(count);
        for (size_t k = 0; k < count; ++k) {
            out.push_back(bits[k] ? 1 : 0);
        }
    }
    return out;
}

template <size_t W>
rust::Vec<uint8_t> copy_table_to_vec(
    const stim::simd_bit_table<W> &table, size_t shots, size_t measurements, bool bit_packed) {
    rust::Vec<uint8_t> out;
    if (bit_packed) {
        size_t row_bytes = (measurements + 7) / 8;
        out.reserve(shots * row_bytes);
        for (size_t shot = 0; shot < shots; ++shot) {
            const auto row = table[shot];
            for (size_t byte_index = 0; byte_index < row_bytes; ++byte_index) {
                uint8_t byte = 0;
                for (size_t bit = 0; bit < 8; ++bit) {
                    size_t idx = byte_index * 8 + bit;
                    if (idx >= measurements) {
                        break;
                    }
                    if (row[idx]) {
                        byte |= static_cast<uint8_t>(1u << bit);
                    }
                }
                out.push_back(byte);
            }
        }
    } else {
        out.reserve(shots * measurements);
        for (size_t shot = 0; shot < shots; ++shot) {
            const auto row = table[shot];
            for (size_t m = 0; m < measurements; ++m) {
                out.push_back(row[m] ? 1 : 0);
            }
        }
    }
    return out;
}

template <size_t W>
rust::Vec<uint8_t> copy_table_shot_major(
    const stim::simd_bit_table<W> &table, size_t num_detectors, size_t shots) {
    rust::Vec<uint8_t> out;
    if (num_detectors == 0 || shots == 0) {
        return out;
    }
    size_t row_bytes = (num_detectors + 7) / 8;
    out.reserve(row_bytes * shots);
    for (size_t shot = 0; shot < shots; ++shot) {
        size_t det_index = 0;
        for (size_t byte_index = 0; byte_index < row_bytes; ++byte_index) {
            uint8_t byte = 0;
            for (size_t bit = 0; bit < 8; ++bit, ++det_index) {
                if (det_index >= num_detectors) {
                    break;
                }
                if (table[det_index][shot]) {
                    byte |= static_cast<uint8_t>(1u << bit);
                }
            }
            out.push_back(byte);
        }
    }
    return out;
}

uint64_t expect_u64_fits_size(size_t value) {
    if (value > std::numeric_limits<uint64_t>::max()) {
        throw std::overflow_error("Value does not fit into u64.");
    }
    return static_cast<uint64_t>(value);
}

size_t expect_size_fits_u64(uint64_t value) {
    if (value > std::numeric_limits<size_t>::max()) {
        throw std::overflow_error("Value exceeds platform limits.");
    }
    return static_cast<size_t>(value);
}

std::mt19937_64 make_rng(const std::optional<uint64_t> &seed) {
    if (seed.has_value()) {
        return std::mt19937_64(*seed);
    }
    std::random_device rd;
    return std::mt19937_64(rd());
}

}  // namespace

namespace stimrs {

CircuitHandle::CircuitHandle(Circuit circuit) : circuit(std::move(circuit)) {
}

SamplerHandle::SamplerHandle(
    Circuit circuit, stim::simd_bits<stim::MAX_BITWORD_WIDTH> ref_sample, bool skip_ref, std::mt19937_64 rng)
    : circuit(std::move(circuit)),
      reference_sample(std::move(ref_sample)),
      skip_reference_sample(skip_ref),
      rng(std::move(rng)) {
}

std::unique_ptr<CircuitHandle> circuit_from_text(rust::Str text) {
    return handle_errors([&]() {
        Circuit parsed{std::string(text.data(), text.size())};
        return std::make_unique<CircuitHandle>(std::move(parsed));
    });
}

std::unique_ptr<CircuitHandle> circuit_clone(const CircuitHandle &circuit) {
    return std::make_unique<CircuitHandle>(circuit.circuit);
}

uint64_t circuit_count_qubits(const CircuitHandle &circuit) {
    return expect_u64_fits_size(circuit.circuit.count_qubits());
}

uint64_t circuit_count_measurements(const CircuitHandle &circuit) {
    return circuit.circuit.count_measurements();
}

uint64_t circuit_count_detectors(const CircuitHandle &circuit) {
    return circuit.circuit.count_detectors();
}

uint64_t circuit_count_observables(const CircuitHandle &circuit) {
    return circuit.circuit.count_observables();
}

rust::String circuit_to_string(const CircuitHandle &circuit) {
    return circuit.circuit.str();
}

rust::Vec<uint8_t> circuit_reference_sample(const CircuitHandle &circuit, bool bit_packed) {
    return handle_errors([&]() {
        auto ref = TableauSimulator<stim::MAX_BITWORD_WIDTH>::reference_sample_circuit(circuit.circuit);
        return copy_bits_to_vec(ref, circuit.circuit.count_measurements(), bit_packed);
    });
}

ReferenceSignsRaw circuit_reference_signs(const CircuitHandle &circuit, bool bit_packed) {
    return handle_errors([&]() {
        ReferenceSignsRaw out;
        auto ref = TableauSimulator<stim::MAX_BITWORD_WIDTH>::reference_sample_circuit(circuit.circuit);
        size_t num_detectors = circuit.circuit.count_detectors();
        size_t num_observables = circuit.circuit.count_observables();
        stim::simd_bits<stim::MAX_BITWORD_WIDTH> detector_signs(num_detectors);
        stim::simd_bits<stim::MAX_BITWORD_WIDTH> observable_signs(num_observables);
        size_t measurement_cursor = 0;
        size_t detector_index = 0;
        circuit.circuit.for_each_operation([&](const CircuitInstruction &inst) {
            if (inst.gate_type == GateType::DETECTOR || inst.gate_type == GateType::OBSERVABLE_INCLUDE) {
                stim::bit_ref dest = inst.gate_type == GateType::DETECTOR
                                         ? detector_signs[detector_index++]
                                         : observable_signs[static_cast<size_t>(inst.args[0])];
                for (const auto &target : inst.targets) {
                    if (target.is_measurement_record_target()) {
                        dest ^= ref[measurement_cursor + target.value()];
                    }
                }
            } else {
                measurement_cursor += inst.count_measurement_results();
            }
        });
        out.detectors = copy_bits_to_vec(detector_signs, num_detectors, bit_packed);
        out.observables = copy_bits_to_vec(observable_signs, num_observables, bit_packed);
        return out;
    });
}

DetectorErrorModelFlat circuit_detector_error_model(const CircuitHandle &circuit, bool decompose_errors) {
    return handle_errors([&]() {
        stim::DemOptions options;
        options.decompose_errors = decompose_errors;
        options.flatten_loops = true;
        auto dem = stim::circuit_to_dem(circuit.circuit, options);
        DetectorErrorModelFlat out;
        out.num_detectors = dem.count_detectors();
        out.num_observables = dem.count_observables();
        dem.iter_flatten_error_instructions([&](const stim::DemInstruction &inst) {
            if (inst.type != stim::DemInstructionType::DEM_ERROR || inst.arg_data.empty()) {
                return;
            }
            DemError error;
            error.probability = inst.arg_data[0];
            for (auto target : inst.target_data) {
                if (target.is_separator()) {
                    continue;
                }
                if (target.is_relative_detector_id()) {
                    error.detectors.push_back(target.val());
                } else if (target.is_observable_id()) {
                    error.observables.push_back(target.val());
                }
            }
            out.errors.push_back(std::move(error));
        });
        return out;
    });
}

SampledDetectionEvents circuit_sample_detection_events(
    const CircuitHandle &circuit, uint64_t shots, uint64_t seed, bool use_seed) {
    return handle_errors([&]() {
        size_t shot_count = expect_size_fits_u64(shots);
        std::mt19937_64 rng = make_rng(use_seed ? std::optional<uint64_t>(seed) : std::nullopt);
        auto tables = stim::sample_batch_detection_events<stim::MAX_BITWORD_WIDTH>(circuit.circuit, shot_count, rng);
        SampledDetectionEvents out;
        out.num_detectors = circuit.circuit.count_detectors();
        out.num_observables = circuit.circuit.count_observables();
        out.shots = shots;
        out.detector_data = copy_table_shot_major(tables.first, out.num_detectors, shot_count);
        out.observable_data = copy_table_shot_major(tables.second, out.num_observables, shot_count);
        return out;
    });
}

std::unique_ptr<CircuitHandle> circuit_surface_code(
    uint64_t rounds,
    uint32_t distance,
    rust::Str task,
    double after_clifford_depolarization,
    double before_round_data_depolarization,
    double before_measure_flip_probability,
    double after_reset_flip_probability) {
    return handle_errors([&]() {
        stim::CircuitGenParameters params(rounds, distance, std::string(task.data(), task.size()));
        params.after_clifford_depolarization = after_clifford_depolarization;
        params.before_round_data_depolarization = before_round_data_depolarization;
        params.before_measure_flip_probability = before_measure_flip_probability;
        params.after_reset_flip_probability = after_reset_flip_probability;
        auto generated = stim::generate_surface_code_circuit(params);
        return std::make_unique<CircuitHandle>(std::move(generated.circuit));
    });
}

std::unique_ptr<SamplerHandle> sampler_create(
    const CircuitHandle &circuit, bool skip_reference_sample, uint64_t seed, bool use_seed) {
    return handle_errors([&]() {
        std::optional<uint64_t> maybe_seed = use_seed ? std::make_optional(seed) : std::nullopt;
        stim::simd_bits<stim::MAX_BITWORD_WIDTH> reference =
            skip_reference_sample ? stim::simd_bits<stim::MAX_BITWORD_WIDTH>(circuit.circuit.count_measurements())
                                  : TableauSimulator<stim::MAX_BITWORD_WIDTH>::reference_sample_circuit(circuit.circuit);
        return std::make_unique<SamplerHandle>(
            circuit.circuit, std::move(reference), skip_reference_sample, make_rng(maybe_seed));
    });
}

uint64_t sampler_count_measurements(const SamplerHandle &sampler) {
    return sampler.circuit.count_measurements();
}

rust::Vec<uint8_t> sampler_sample(SamplerHandle &sampler, uint64_t shots) {
    return handle_errors([&]() {
        size_t shot_count = expect_size_fits_u64(shots);
        auto table = stim::sample_batch_measurements<stim::MAX_BITWORD_WIDTH>(
            sampler.circuit, sampler.reference_sample, shot_count, sampler.rng, true);
        return copy_table_to_vec(table, shot_count, sampler.circuit.count_measurements(), false);
    });
}

rust::Vec<uint8_t> sampler_sample_bit_packed(SamplerHandle &sampler, uint64_t shots) {
    return handle_errors([&]() {
        size_t shot_count = expect_size_fits_u64(shots);
        auto table = stim::sample_batch_measurements<stim::MAX_BITWORD_WIDTH>(
            sampler.circuit, sampler.reference_sample, shot_count, sampler.rng, true);
        return copy_table_to_vec(table, shot_count, sampler.circuit.count_measurements(), true);
    });
}

void sampler_reseed(SamplerHandle &sampler, uint64_t seed) {
    sampler.rng.seed(seed);
}

}  // namespace stimrs
