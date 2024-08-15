#[path = "ctx_winit.rs"]
mod wctx;
use wctx::run;

fn main() {
    pollster::block_on(run());
}
