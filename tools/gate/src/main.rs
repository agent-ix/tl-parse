//! Shim: the mutation gate and runner are `tl-gate`, built from tl-syntax.

fn main() -> std::process::ExitCode {
    tl_gate::run(std::env::args().skip(1))
}
