extern crate lalrpop;

fn main() {
    build_grammar()
}

fn build_grammar() {
    lalrpop::process_root().unwrap();
}
