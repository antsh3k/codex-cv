use crate::error::SubagentResult;
use crate::spec::SubagentSpec;
use crate::task_context::TaskContext;

pub trait Subagent {
    fn spec(&self) -> &SubagentSpec;
    fn spec_mut(&mut self) -> &mut SubagentSpec;

    fn name(&self) -> &str {
        self.spec().name()
    }

    fn model_override(&self) -> Option<&str> {
        self.spec().model()
    }

    fn instructions(&self) -> &str {
        self.spec().instructions()
    }
}

pub trait TypedSubagent: Subagent {
    type Request;
    type Response;

    fn run(
        &mut self,
        ctx: &mut TaskContext,
        request: Self::Request,
    ) -> SubagentResult<Self::Response>;
}

pub trait ContextualSubagent<C>: TypedSubagent {
    fn prepare(&mut self, ctx: &mut TaskContext, context: &C) -> SubagentResult<()> {
        let _ = ctx;
        let _ = context;
        Ok(())
    }
}

impl<T, C> ContextualSubagent<C> for T where T: TypedSubagent {}
