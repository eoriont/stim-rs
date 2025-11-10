use crate::circuit::Circuit;
use crate::ffi;
use crate::util::{bytes_to_string, to_usize};
use crate::StimError;
use cxx::UniquePtr;
use std::collections::BTreeMap;
use std::convert::TryFrom;
use std::fmt;
use std::ops::{Add, Mul};

/// The kind of DEM target referenced by an instruction.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DemTargetKind {
    RelativeDetectorId,
    ObservableId,
    Separator,
}

/// A target referenced by a detector error model instruction.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DemTarget {
    pub kind: DemTargetKind,
    pub value: u64,
}

impl DemTarget {
    pub fn relative_detector(id: u64) -> Self {
        ffi::dem_target_relative_detector(id).into()
    }

    pub fn observable(id: u64) -> Self {
        ffi::dem_target_observable(id).into()
    }

    pub fn separator() -> Self {
        ffi::dem_target_separator().into()
    }

    pub fn is_relative_detector(&self) -> bool {
        matches!(self.kind, DemTargetKind::RelativeDetectorId)
    }

    pub fn is_observable(&self) -> bool {
        matches!(self.kind, DemTargetKind::ObservableId)
    }

    pub fn is_separator(&self) -> bool {
        matches!(self.kind, DemTargetKind::Separator)
    }

    pub fn shift(&self, offset: i64) -> Self {
        DemTarget::from(ffi::dem_target_shift(
            &ffi::DemTargetOwned::from(self),
            offset,
        ))
    }
}

/// The different instruction kinds inside a detector error model.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DemInstructionKind {
    Error,
    ShiftDetectors,
    Detector,
    LogicalObservable,
    RepeatBlock,
}

/// Metadata describing a repeat block body.
#[derive(Debug, Clone)]
pub struct RepeatBlock {
    pub repeat_count: u64,
    pub body: Vec<DemInstruction>,
}

/// A detector error model instruction.
#[derive(Debug, Clone)]
pub struct DemInstruction {
    pub kind: DemInstructionKind,
    pub args: Vec<f64>,
    pub targets: Vec<DemTarget>,
    pub tag: String,
    pub repeat: Option<RepeatBlock>,
}

impl DemInstruction {
    pub fn separated_target_groups(&self) -> Vec<Vec<DemTarget>> {
        let mut groups = Vec::new();
        let mut current = Vec::new();
        for target in &self.targets {
            if target.is_separator() {
                if !current.is_empty() {
                    groups.push(std::mem::take(&mut current));
                }
                continue;
            }
            current.push(target.clone());
        }
        if !current.is_empty() {
            groups.push(current);
        }
        groups
    }

    pub fn detector_targets(&self) -> impl Iterator<Item = &DemTarget> {
        self.targets
            .iter()
            .filter(|t| matches!(t.kind, DemTargetKind::RelativeDetectorId))
    }
}

/// A single error mechanism from the detector error model.
#[derive(Debug, Clone)]
pub struct DemError {
    pub probability: f64,
    pub detectors: Vec<u64>,
    pub observables: Vec<u64>,
}

/// Flattened detector error model information.
#[derive(Debug, Clone)]
pub struct DetectorErrorModelFlat {
    pub errors: Vec<DemError>,
    pub num_detectors: u64,
    pub num_observables: u64,
}

/// Options controlling detector error model extraction.
#[derive(Debug, Clone)]
pub struct DemOptions {
    pub decompose_errors: bool,
}

impl Default for DemOptions {
    fn default() -> Self {
        Self {
            decompose_errors: true,
        }
    }
}

/// A detector error model that can be inspected and mutated.
pub struct DetectorErrorModel {
    inner: UniquePtr<ffi::DetectorErrorModelHandle>,
}

impl DetectorErrorModel {
    /// Parses a detector error model from text.
    pub fn from_text(src: &str) -> Result<Self, StimError> {
        let inner = ffi::detector_error_model_from_text(src)?;
        Ok(Self { inner })
    }

    /// Creates an empty detector error model that can be mutated.
    pub fn new() -> Self {
        Self {
            inner: ffi::detector_error_model_empty(),
        }
    }

    /// Generates the detector error model for the provided circuit.
    pub fn from_circuit(circuit: &Circuit, options: DemOptions) -> Result<Self, StimError> {
        let inner =
            ffi::detector_error_model_from_circuit(circuit.as_ref(), options.decompose_errors)?;
        Ok(Self { inner })
    }

    /// Returns a copy of the detector error model without tags.
    pub fn without_tags(&self) -> Self {
        Self {
            inner: ffi::detector_error_model_without_tags(self.as_ref()),
        }
    }

    /// Returns a flattened version with repeat blocks removed.
    pub fn flattened(&self) -> Self {
        Self {
            inner: ffi::detector_error_model_flattened(self.as_ref()),
        }
    }

    /// Returns a rounded version of all probabilities.
    pub fn rounded(&self, digits: u8) -> Self {
        Self {
            inner: ffi::detector_error_model_rounded(self.as_ref(), digits),
        }
    }

    /// Multiplies the detector error model by repeating it.
    pub fn repeated(&self, reps: u64) -> Self {
        Self {
            inner: ffi::detector_error_model_mul(self.as_ref(), reps),
        }
    }

    /// Adds two detector error models together.
    pub fn add(&self, other: &Self) -> Self {
        Self {
            inner: ffi::detector_error_model_add(self.as_ref(), other.as_ref()),
        }
    }

    /// Returns true if the models are approximately equal within atol.
    pub fn approx_equals(&self, other: &Self, atol: f64) -> bool {
        ffi::detector_error_model_approx_eq(self.as_ref(), other.as_ref(), atol)
    }

    pub fn count_detectors(&self) -> Result<usize, StimError> {
        to_usize(ffi::detector_error_model_num_detectors(self.as_ref()))
    }

    pub fn count_observables(&self) -> Result<usize, StimError> {
        to_usize(ffi::detector_error_model_num_observables(self.as_ref()))
    }

    pub fn total_detector_shift(&self) -> u64 {
        ffi::detector_error_model_total_detector_shift(self.as_ref())
    }

    pub fn instructions(&self) -> Result<Vec<DemInstruction>, StimError> {
        let info = ffi::detector_error_model_instructions(self.as_ref());
        info.instructions
            .into_iter()
            .map(DemInstruction::try_from)
            .collect()
    }

    pub fn flattened_error_instructions(&self) -> Result<Vec<DemInstruction>, StimError> {
        let info = ffi::detector_error_model_flattened_instructions(self.as_ref());
        info.instructions
            .into_iter()
            .map(DemInstruction::try_from)
            .collect()
    }

    pub fn detector_coordinates(&self, included: &[u64]) -> BTreeMap<u64, Vec<f64>> {
        let data =
            ffi::detector_error_model_get_detector_coordinates(self.as_ref(), included.to_vec());
        let mut map = BTreeMap::new();
        for entry in data {
            map.insert(entry.detector, entry.coords);
        }
        map
    }

    pub fn final_detector_and_coord_shift(&self) -> (u64, Vec<f64>) {
        let raw = ffi::detector_error_model_final_detector_and_coord_shift(self.as_ref());
        (raw.detector_shift, raw.coord_shift)
    }

    pub fn layout_str(&self) -> String {
        ffi::detector_error_model_layout_str(self.as_ref())
    }

    pub fn hint_str(&self) -> String {
        ffi::detector_error_model_hint_str(self.as_ref())
    }

    pub fn to_stim_string(&self) -> String {
        ffi::detector_error_model_str(self.as_ref())
    }

    pub fn append_error(
        &mut self,
        probability: f64,
        targets: &[DemTarget],
        tag: &str,
    ) -> Result<(), StimError> {
        let owned: Vec<_> = targets
            .iter()
            .cloned()
            .map(ffi::DemTargetOwned::from)
            .collect();
        ffi::detector_error_model_append_error(self.inner.pin_mut(), probability, &owned, tag);
        Ok(())
    }

    pub fn append_shift_detectors(
        &mut self,
        coord_shift: &[f64],
        detector_shift: u64,
        tag: &str,
    ) -> Result<(), StimError> {
        ffi::detector_error_model_append_shift_detectors(
            self.inner.pin_mut(),
            coord_shift,
            detector_shift,
            tag,
        );
        Ok(())
    }

    pub fn append_detector(
        &mut self,
        coords: &[f64],
        target: &DemTarget,
        tag: &str,
    ) -> Result<(), StimError> {
        ffi::detector_error_model_append_detector(
            self.inner.pin_mut(),
            coords,
            &ffi::DemTargetOwned::from(target.clone()),
            tag,
        );
        Ok(())
    }

    pub fn append_logical_observable(
        &mut self,
        target: &DemTarget,
        tag: &str,
    ) -> Result<(), StimError> {
        ffi::detector_error_model_append_logical_observable(
            self.inner.pin_mut(),
            &ffi::DemTargetOwned::from(target.clone()),
            tag,
        );
        Ok(())
    }

    pub fn append_repeat_block(
        &mut self,
        repeat_count: u64,
        body: &DetectorErrorModel,
        tag: &str,
    ) -> Result<(), StimError> {
        ffi::detector_error_model_append_repeat_block(
            self.inner.pin_mut(),
            repeat_count,
            body.as_ref(),
            tag,
        );
        Ok(())
    }

    pub fn append_from_text(&mut self, text: &str) -> Result<(), StimError> {
        ffi::detector_error_model_append_from_text(self.inner.pin_mut(), text);
        Ok(())
    }

    pub fn append_from_file(&mut self, path: &str) -> Result<(), StimError> {
        ffi::detector_error_model_append_from_file(self.inner.pin_mut(), path);
        Ok(())
    }

    pub(crate) fn as_ref(&self) -> &ffi::DetectorErrorModelHandle {
        self.inner
            .as_ref()
            .expect("DetectorErrorModel handle unexpectedly null (internal bug)")
    }
}

impl Clone for DetectorErrorModel {
    fn clone(&self) -> Self {
        Self {
            inner: ffi::detector_error_model_clone(self.as_ref()),
        }
    }
}

impl fmt::Debug for DetectorErrorModel {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("DetectorErrorModel")
            .field("num_detectors", &self.count_detectors().ok())
            .field("num_observables", &self.count_observables().ok())
            .finish()
    }
}

impl PartialEq for DetectorErrorModel {
    fn eq(&self, other: &Self) -> bool {
        ffi::detector_error_model_eq(self.as_ref(), other.as_ref())
    }
}

impl Eq for DetectorErrorModel {}

impl Add for &DetectorErrorModel {
    type Output = DetectorErrorModel;

    fn add(self, rhs: Self) -> Self::Output {
        self.add(rhs)
    }
}

impl Mul<u64> for &DetectorErrorModel {
    type Output = DetectorErrorModel;

    fn mul(self, rhs: u64) -> Self::Output {
        self.repeated(rhs)
    }
}

impl From<ffi::DemTargetKind> for DemTargetKind {
    fn from(kind: ffi::DemTargetKind) -> Self {
        match kind {
            ffi::DemTargetKind::RELATIVE_DETECTOR_ID => DemTargetKind::RelativeDetectorId,
            ffi::DemTargetKind::OBSERVABLE_ID => DemTargetKind::ObservableId,
            ffi::DemTargetKind::SEPARATOR => DemTargetKind::Separator,
            _ => panic!("unknown DemTargetKind value"),
        }
    }
}

impl From<ffi::DemTargetOwned> for DemTarget {
    fn from(raw: ffi::DemTargetOwned) -> Self {
        Self {
            kind: raw.kind.into(),
            value: raw.value,
        }
    }
}

impl From<DemTarget> for ffi::DemTargetOwned {
    fn from(target: DemTarget) -> Self {
        (&target).into()
    }
}

impl From<&DemTarget> for ffi::DemTargetOwned {
    fn from(target: &DemTarget) -> Self {
        Self {
            kind: match target.kind {
                DemTargetKind::RelativeDetectorId => ffi::DemTargetKind::RELATIVE_DETECTOR_ID,
                DemTargetKind::ObservableId => ffi::DemTargetKind::OBSERVABLE_ID,
                DemTargetKind::Separator => ffi::DemTargetKind::SEPARATOR,
            },
            value: target.value,
        }
    }
}

impl From<ffi::DemInstructionKind> for DemInstructionKind {
    fn from(kind: ffi::DemInstructionKind) -> Self {
        match kind {
            ffi::DemInstructionKind::ERROR => DemInstructionKind::Error,
            ffi::DemInstructionKind::SHIFT_DETECTORS => DemInstructionKind::ShiftDetectors,
            ffi::DemInstructionKind::DETECTOR => DemInstructionKind::Detector,
            ffi::DemInstructionKind::LOGICAL_OBSERVABLE => DemInstructionKind::LogicalObservable,
            ffi::DemInstructionKind::REPEAT_BLOCK => DemInstructionKind::RepeatBlock,
            _ => panic!("unknown DemInstructionKind value"),
        }
    }
}

impl TryFrom<ffi::DemInstructionOwned> for DemInstruction {
    type Error = StimError;

    fn try_from(raw: ffi::DemInstructionOwned) -> Result<Self, Self::Error> {
        let targets = raw.targets.into_iter().map(DemTarget::from).collect();
        let repeat = if raw.kind == ffi::DemInstructionKind::REPEAT_BLOCK {
            let body = raw
                .body
                .into_iter()
                .map(DemInstruction::try_from)
                .collect::<Result<Vec<_>, _>>()?;
            Some(RepeatBlock {
                repeat_count: raw.repeat_count,
                body,
            })
        } else {
            None
        };
        Ok(Self {
            kind: raw.kind.into(),
            args: raw.args,
            targets,
            tag: bytes_to_string(raw.tag),
            repeat,
        })
    }
}

impl From<ffi::DemError> for DemError {
    fn from(raw: ffi::DemError) -> Self {
        Self {
            probability: raw.probability,
            detectors: raw.detectors,
            observables: raw.observables,
        }
    }
}

impl From<ffi::DetectorErrorModelFlat> for DetectorErrorModelFlat {
    fn from(raw: ffi::DetectorErrorModelFlat) -> Self {
        Self {
            errors: raw.errors.into_iter().map(DemError::from).collect(),
            num_detectors: raw.num_detectors,
            num_observables: raw.num_observables,
        }
    }
}
