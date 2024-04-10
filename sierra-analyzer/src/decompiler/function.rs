use cairo_lang_sierra::program::GenFunction;
use cairo_lang_sierra::program::StatementIdx;

#[allow(dead_code)]
pub struct ControlFlowGraph {}
impl ControlFlowGraph {}

#[allow(dead_code)]
pub struct Function<'a> {
    function: GenFunction<&'a StatementIdx>,
    // TODO : Add statements
    cfg: ControlFlowGraph,
}
