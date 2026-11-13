use std::io;
use std::sync::{
    Arc,
    atomic::{AtomicUsize, Ordering},
};

#[derive(Clone, Copy)]
pub(crate) struct Seed(pub(crate) u64);

pub(crate) struct Calls(AtomicUsize);

impl Calls {
    pub(crate) fn record(&self) {
        self.0.fetch_add(1, Ordering::Relaxed);
    }
}

pub(crate) struct Context {
    pub(crate) seed: Seed,
    calls: Arc<Calls>,
}

impl Context {
    pub(crate) fn calls(&self) -> Arc<Calls> {
        Arc::clone(&self.calls)
    }
}

#[derive(Debug, Eq, PartialEq)]
pub(crate) struct Observation {
    pub(crate) tool: &'static str,
    pub(crate) value: u64,
}

pub(crate) trait Operation: Send + Sync {
    fn invoke(&self) -> Observation;
}

pub(crate) struct Definition {
    pub(crate) initialize: fn(&Context) -> Box<dyn Operation>,
}

kodogu_runtime::collect!(Definition);

impl kodogu_runtime::Initializer for Definition {
    type Context = Context;
    type Instance = Box<dyn Operation>;
    type Error = io::Error;
    type Future<'a> = kodogu_runtime::Initialization<'a, Self::Instance, Self::Error>;

    fn initialize<'a>(
        &'a self,
        context: &'a Context,
    ) -> kodogu_runtime::Initialization<'a, Self::Instance, Self::Error> {
        Box::pin(async move { Ok((self.initialize)(context)) })
    }
}

pub(crate) struct Host {
    context: Context,
    operations: Vec<Box<dyn Operation>>,
}

impl Host {
    pub(crate) async fn start(seed: io::Result<Seed>) -> io::Result<Self> {
        let context = Context { seed: seed?, calls: Arc::new(Calls(AtomicUsize::new(0))) };
        let operations =
            kodogu_runtime::initialize_all(kodogu_runtime::registered::<Definition>(), &context)
                .await?;
        Ok(Self { context, operations })
    }

    pub(crate) fn invoke(&self) -> Vec<Observation> {
        self.operations.iter().map(|operation| operation.invoke()).collect()
    }

    pub(crate) fn call_count(&self) -> usize {
        self.context.calls.0.load(Ordering::Relaxed)
    }
}
