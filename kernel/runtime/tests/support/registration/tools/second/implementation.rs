use crate::foundation::{Calls, Context, Definition, Observation, Operation, Seed};
use std::sync::Arc;

struct Instance {
    seed: Seed,
    calls: Arc<Calls>,
}

impl Operation for Instance {
    fn invoke(&self) -> Observation {
        self.calls.record();
        Observation { tool: "second", value: domain::evaluate(self.seed.0) }
    }
}

fn initialize(context: &Context) -> Box<dyn Operation> {
    Box::new(Instance { seed: context.seed, calls: context.calls() })
}

kodogu_runtime::submit!(Definition { initialize });

mod domain {
    pub(super) fn evaluate(value: u64) -> u64 {
        value * 3
    }
}
