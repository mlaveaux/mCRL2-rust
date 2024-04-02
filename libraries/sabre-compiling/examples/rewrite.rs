use sabre_ffi::ATerm;

#[no_mangle]
pub unsafe extern "C" fn rewrite_term(term: ATerm) -> ATerm {
    term
}

fn main() {

}