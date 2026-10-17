use std::mem;

use dash_middle::indexvec::IndexVec;
use rustc_hash::FxHashMap;

use crate::frame::Ip;
use crate::jit::ir::{BasicBlockId, Body};

pub type CpuIp = u32;

#[derive(Debug, Clone)]
pub enum PatchData {
    Unresolved { references: Vec<CpuIp> },
    Resolved { ip: CpuIp },
}

impl Default for PatchData {
    fn default() -> Self {
        Self::Unresolved { references: Vec::new() }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SyntheticLabel {
    StubStatusHandler,
    Exit,
    Epilogue,
}

#[derive(Debug, Default)]
pub struct JumpResolver {
    /// Maps basic block to CPU IP
    user_labels: IndexVec<PatchData, BasicBlockId>,
    /// Maps synthetic labels to CPU IP
    internal_labels: FxHashMap<SyntheticLabel, PatchData>,
}

impl JumpResolver {
    pub fn new(body: &Body) -> Self {
        Self {
            user_labels: IndexVec::repeat_n(PatchData::default(), body.blocks.len()),
            internal_labels: FxHashMap::default(),
        }
    }

    pub fn assert_resolved(&self) {
        for label in self.user_labels.iter() {
            if let PatchData::Unresolved { references } = label {
                if !references.is_empty() {
                    panic!("unresolved user label with references: {:?}", references);
                }
            }
        }
        for (label, patch_data) in self.internal_labels.iter() {
            if let PatchData::Unresolved { references } = patch_data {
                if !references.is_empty() {
                    panic!(
                        "unresolved internal label {:?} with references: {:?}",
                        label, references
                    );
                }
            }
        }
    }

    pub fn user_label(&self, block: BasicBlockId) -> Option<CpuIp> {
        match &self.user_labels[block] {
            PatchData::Resolved { ip } => Some(*ip),
            PatchData::Unresolved { .. } => None,
        }
    }

    pub fn add_user_reference(&mut self, target: BasicBlockId, patch_site: CpuIp) -> Option<CpuIp> {
        let slot = &mut self.user_labels[target];

        match slot {
            PatchData::Resolved { ip: x86_ip } => Some(*x86_ip),
            PatchData::Unresolved { references } => {
                references.push(patch_site);
                None
            }
        }
    }

    /// Registers a user label at the given IP and returns the usages that need to be resolved (if it wasn't already resolved previously,
    /// in which case it returns `PatchData::Resolved`).
    #[must_use]
    pub fn register_user_label(&mut self, block: BasicBlockId, ip: CpuIp) -> PatchData {
        let slot = &mut self.user_labels[block];

        match *slot {
            PatchData::Unresolved { references: _ } => mem::replace(slot, PatchData::Resolved { ip }),
            PatchData::Resolved { ip } => PatchData::Resolved { ip },
        }
    }

    pub fn add_synthetic_reference(&mut self, label: SyntheticLabel, patch_site: CpuIp) -> Option<CpuIp> {
        let slot = self.internal_labels.entry(label).or_default();

        match slot {
            PatchData::Resolved { ip: x86_ip } => Some(*x86_ip),
            PatchData::Unresolved { references } => {
                references.push(patch_site);
                None
            }
        }
    }

    /// Registers a synthetic label at the given IP and returns the usages that need to be resolved (if it wasn't already resolved previously,
    /// in which case it returns `PatchData::Resolved`).
    #[must_use]
    pub fn register_synthetic_label(&mut self, label: SyntheticLabel, x86_ip: CpuIp) -> PatchData {
        let slot = self.internal_labels.entry(label).or_default();

        match *slot {
            PatchData::Unresolved { references: _ } => mem::replace(slot, PatchData::Resolved { ip: x86_ip }),
            PatchData::Resolved { ip } => PatchData::Resolved { ip },
        }
    }
}
