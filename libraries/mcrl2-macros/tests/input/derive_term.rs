use mcrl2_macros::Term;

#[derive(Term)]
struct BoolSortRef {}

impl BoolSortRef {

    pub fn test(&self) -> bool {
        true
    }
}

fn main() {

}