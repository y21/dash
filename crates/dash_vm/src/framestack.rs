use dash_middle::compiler::constant::ConstantPool;
use dash_middle::index_type;
use dash_middle::indexvec::IndexVec;
use dash_middle::interner::Symbol;
use dash_proc_macro::Trace;

use crate::frame::{BaseFrame, ExtendedFrame, Frame, FrameState};
use crate::gc::ObjectId;
use crate::value::object::This;
use crate::value::{ExternalValue, Unrooted};

const MAX_FRAME_COUNT: u32 = 1024;

index_type! {
    #[derive(Debug, Clone, Copy, Trace, PartialEq, Eq, PartialOrd, Ord)]
    pub struct FrameId(pub u32);
}

#[derive(Trace)]
pub struct FrameStack {
    // Implementation detail: we split the frame data simply because certain fields are so frequently accessed
    // that we would like to inline that directly into the struct so as to be able to read it without indirection.
    current_base: Option<BaseFrame>,
    base: IndexVec<BaseFrame, FrameId>,
    extended: IndexVec<ExtendedFrame, FrameId>,
}

impl FrameStack {
    pub fn new() -> Self {
        Self {
            current_base: None,
            base: IndexVec::new(),
            extended: IndexVec::new(),
        }
    }

    fn pop_base(&mut self) -> BaseFrame {
        let current = self.current_base.take().expect("no active frame");
        if let Some(next) = self.base.pop() {
            self.current_base = Some(next);
        }
        current
    }

    fn push_base(&mut self, base: BaseFrame) {
        if let Some(current) = self.current_base.take() {
            self.base.push(current);
        }
        self.current_base = Some(base);
    }

    fn current_base(&self) -> BaseFrame {
        self.current_base.clone().expect("no active frame")
    }

    fn current_base_ref(&self) -> &BaseFrame {
        self.current_base.as_ref().expect("no active frame")
    }

    fn current_base_mut(&mut self) -> &mut BaseFrame {
        self.current_base.as_mut().expect("no active frame")
    }

    fn current_extended(&self) -> &ExtendedFrame {
        self.extended.last().expect("no active frame")
    }

    fn current_extended_mut(&mut self) -> &mut ExtendedFrame {
        self.extended.last_mut().expect("no active frame")
    }

    pub fn take_delayed_ret(&mut self) -> Option<Result<Unrooted, Unrooted>> {
        self.current_extended_mut().delayed_ret.take()
    }

    pub fn current_sp(&self) -> usize {
        self.current_extended().sp
    }

    pub fn current_ip(&self) -> usize {
        self.current_base().ip
    }

    pub fn current_this(&self) -> This {
        self.current_extended().this
    }

    pub fn current_external(&self, id: usize) -> ExternalValue {
        self.current_extended().externals[id].clone()
    }

    pub fn current_constants(&self) -> &ConstantPool {
        &self.current_base_ref().function.constants
    }

    pub fn current_state(&self) -> &FrameState {
        &self.current_extended().state
    }

    pub fn current_state_mut(&mut self) -> &mut FrameState {
        &mut self.current_extended_mut().state
    }

    pub fn current_arguments(&self) -> Option<ObjectId> {
        self.current_extended().arguments
    }

    pub fn set_ip(&mut self, ip: usize) {
        let base = self.current_base_mut();
        base.ip = ip;
    }

    pub fn set_delayed_ret(&mut self, delayed_ret: Option<Result<Unrooted, Unrooted>>) {
        let extended = self.current_extended_mut();
        extended.delayed_ret = delayed_ret;
    }

    pub fn set_this(&mut self, this: This) {
        let extended = self.current_extended_mut();
        extended.this = this;
    }

    pub fn resolve_ip_debuginfo(&self, ip: u16) -> &str {
        let base = self.current_base_ref();
        base.function.debug_symbols.get(ip).res(&base.function.source)
    }

    pub fn fetch_and_inc_ip(&mut self) -> u8 {
        let base = self.current_base_mut();
        let ip = base.ip;
        base.ip += 1;
        base.function.buffer.with(|buf| buf[ip as usize])
    }

    fn fetch_n_and_inc_ip<const N: usize>(&mut self) -> [u8; N] {
        let base = self.current_base_mut();
        let value: [u8; N] = base.function.buffer.with(|buf| {
            // Intermediate `as u32` cast is needed to make overflow on the addition impossible and help LLVM
            // collapse the two bounds checks into one.
            // FIXME: store `ip` as `u32` directly to avoid this
            buf[base.ip as u32 as usize..base.ip as u32 as usize + N]
                .try_into()
                .expect("Failed to get wide instruction")
        });
        base.ip += N;
        value
    }

    pub fn fetchw_and_inc_ip(&mut self) -> u16 {
        u16::from_ne_bytes(self.fetch_n_and_inc_ip::<2>())
    }

    pub fn fetch32_and_inc_ip(&mut self) -> u32 {
        u32::from_ne_bytes(self.fetch_n_and_inc_ip::<4>())
    }

    pub fn pop(&mut self) -> Frame {
        let extended = self.extended.pop().expect("no active frame");
        let base = self.pop_base();
        Frame {
            function: base.function,
            ip: base.ip,
            extra_stack_space: extended.extra_stack_space,
            externals: extended.externals,
            this: extended.this,
            sp: extended.sp,
            state: extended.state,
            delayed_ret: extended.delayed_ret,
            arguments: extended.arguments,
            loop_counter: extended.loop_counter,
        }
    }

    pub fn pop_discard(&mut self) {
        self.extended.pop();
        self.pop_base();
    }

    pub fn current_id(&self) -> FrameId {
        FrameId(self.extended.len() - 1)
    }

    /// "Unwinds" to a frame, i.e. removing all frames above the specified frame.
    pub fn unwind_to(&mut self, frame_id: FrameId) {
        if frame_id.0 + 1 < self.len() {
            // We need to pop _at least_ one frame.
            let target = self.base[frame_id].clone();
            self.base.truncate(frame_id.0);
            self.current_base = Some(target);
        }

        self.extended.truncate(frame_id.0 + 1);
    }

    pub fn len(&self) -> u32 {
        self.extended.len() as u32
    }

    pub fn push(&mut self, frame: Frame) -> Result<(), ()> {
        if self.len() < MAX_FRAME_COUNT {
            self.push_base(BaseFrame {
                ip: frame.ip,
                function: frame.function,
            });
            self.extended.push(ExtendedFrame {
                extra_stack_space: frame.extra_stack_space,
                externals: frame.externals,
                this: frame.this,
                sp: frame.sp,
                state: frame.state,
                delayed_ret: frame.delayed_ret,
                arguments: frame.arguments,
                loop_counter: frame.loop_counter,
            });
            Ok(())
        } else {
            Err(())
        }
    }

    pub fn function_name_iter(&self) -> impl DoubleEndedIterator<Item = Option<Symbol>> {
        self.base
            .iter()
            .chain(self.current_base.iter())
            .map(|frame| frame.function.name)
    }
}
