use rustc_hash::FxHashMap;

use crate::frame::Ip;

pub type PatchSite = u32;

pub type X86Ip = u32;

#[derive(Debug, Clone)]
pub enum PatchData {
    Unresolved { references: Vec<PatchSite> },
    Resolved { x86_ip: X86Ip },
}

impl Default for PatchData {
    fn default() -> Self {
        Self::Unresolved { references: Vec::new() }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum InternalLabel {
    StubStatusHandler,
    Epilogue,
}

/// Resolves branches from bytecode IPs to emitted x86 offsets.
///
/// - `user_labels` tracks bytecode targets and whether they are resolved.
/// - `internal_unresolved` tracks unresolved branches to internal labels
///   such as the stub status slow-path handler.
#[derive(Debug, Default)]
pub struct JumpResolver {
    user_labels: Vec<PatchData>,
    internal_labels: FxHashMap<InternalLabel, PatchData>,
}

impl JumpResolver {
    pub fn new(bytecode_len: usize) -> Self {
        Self {
            user_labels: vec![PatchData::default(); bytecode_len + 1], // +1 for the end-of-bytecode label (exit branch)
            internal_labels: FxHashMap::default(),
        }
    }

    /// Returns the resolved x86 target if already known; otherwise records
    /// this patch site as unresolved and returns `None`.
    pub fn add_user_reference(&mut self, target_bc_ip: Ip, patch_site: PatchSite) -> Option<X86Ip> {
        let idx = target_bc_ip.0 as usize;
        let slot = self
            .user_labels
            .get_mut(idx)
            .unwrap_or_else(|| panic!("bytecode ip out of bounds: {target_bc_ip:?}"));

        match slot {
            PatchData::Resolved { x86_ip } => Some(*x86_ip),
            PatchData::Unresolved { references } => {
                references.push(patch_site);
                None
            }
        }
    }

    /// Marks a bytecode label as resolved and returns any patch sites that
    /// previously referenced it.
    pub fn resolve_user_label(&mut self, bc_ip: Ip, x86_ip: X86Ip) -> Vec<PatchSite> {
        let idx = bc_ip.0 as usize;
        let slot = self
            .user_labels
            .get_mut(idx)
            .unwrap_or_else(|| panic!("bytecode ip out of bounds: {bc_ip:?}"));

        match std::mem::replace(slot, PatchData::Resolved { x86_ip }) {
            PatchData::Unresolved { references } => references,
            PatchData::Resolved { .. } => {
                panic!("duplicate label resolution for bytecode ip: {bc_ip:?}")
            }
        }
    }

    pub fn add_internal_reference(&mut self, label: InternalLabel, patch_site: PatchSite) -> Option<X86Ip> {
        let slot = self.internal_labels.entry(label).or_default();

        match slot {
            PatchData::Resolved { x86_ip } => Some(*x86_ip),
            PatchData::Unresolved { references } => {
                references.push(patch_site);
                None
            }
        }
    }

    pub fn resolve_internal_label(&mut self, label: InternalLabel, x86_ip: X86Ip) -> Vec<PatchSite> {
        let slot = self.internal_labels.entry(label).or_default();

        match std::mem::replace(slot, PatchData::Resolved { x86_ip }) {
            PatchData::Unresolved { references } => references,
            PatchData::Resolved { .. } => {
                panic!("duplicate internal label resolution for {label:?}")
            }
        }
    }
}
