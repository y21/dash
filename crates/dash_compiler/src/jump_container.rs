use std::collections::BTreeMap;

use crate::builder::Label;

/// A container that is responsible for storing and resolving jumps to labels
pub struct JumpContainer {
    jumps: BTreeMap<Label, Vec<usize>>,
    labels: BTreeMap<Label, usize>,
}

impl JumpContainer {
    pub fn new() -> Self {
        Self {
            jumps: BTreeMap::new(),
            labels: BTreeMap::new(),
        }
    }
}
/// Adds a label at the current instruction pointer, which can be jumped to using add_local_jump
pub fn add_label(jc: &mut JumpContainer, label: Label, buf: &mut [u8]) {
    let ip = buf.len();

    // get vector of existing jumps to this label
    if let Some(assoc_jumps) = jc.jumps.remove(&label) {
        for jump in assoc_jumps {
            let offset = (ip - jump - 2) as u16; // TODO: don't hardcast..? and use i16

            // write jump offset
            let pt = &mut buf[jump..jump + 2];
            pt.copy_from_slice(&u16::to_ne_bytes(offset));
        }
    }

    jc.labels.insert(label, ip);
}

/// Emits a jump instruction to a local label
///
/// Requirement for calling this function: there must be two bytes in the buffer, reserved for this jump
pub fn add_jump(jc: &mut JumpContainer, label: Label, buf: &mut [u8]) {
    if let Some(&ip) = jc.labels.get(&label) {
        let ip = ip as isize;
        let len = buf.len() as isize;
        let offset = (ip - len) as i16; // TODO: don't hardcast..?

        let pt = &mut buf[len as usize - 2..];
        pt.copy_from_slice(&i16::to_ne_bytes(offset));
    } else {
        jc.jumps.entry(label).or_insert_with(Vec::new).push(buf.len() - 2);
    }
}
