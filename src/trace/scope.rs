// Copyright 2020 TiKV Project Authors. Licensed under Apache-2.0.

use crossbeam_channel::Sender;
use std::sync::atomic::AtomicBool;
use std::sync::Arc;

use crate::local::acquirer_group::merge_registered_local_acquirers;
use crate::local::scope_guard::LocalScopeGuard;
use crate::span::cycle::DefaultClock;
use crate::span::span_id::{DefaultIdGenerator, SpanId};
use crate::span::ScopeSpan;
use crate::trace::acquirer::{Acquirer, AcquirerGroup, SpanCollection};

#[derive(Clone, Default)]
pub struct Scope {
    pub(crate) acquirer_group: Option<Arc<AcquirerGroup>>,
}

impl Scope {
    pub fn start_scope(&self) -> LocalScopeGuard {
        LocalScopeGuard::new(self.acquirer_group.clone())
    }
}

impl Scope {
    pub(crate) fn new_root_scope(
        event: &'static str,
        sender: Sender<SpanCollection>,
        closed: Arc<AtomicBool>,
    ) -> Self {
        let root_span = ScopeSpan::new(
            DefaultIdGenerator::next_id(),
            SpanId::new(0),
            DefaultClock::now(),
            event,
        );
        let acq = Acquirer::new(Arc::new(sender), closed);
        let acq_group = AcquirerGroup::new(root_span, vec![acq]);

        Self {
            acquirer_group: Some(Arc::new(acq_group)),
        }
    }

    pub(crate) fn merge_local_scopes(event: &'static str) -> Self {
        Self {
            acquirer_group: merge_registered_local_acquirers(event).map(Arc::new),
        }
    }

    pub(crate) fn release(&mut self) {
        self.acquirer_group.take();
    }
}
