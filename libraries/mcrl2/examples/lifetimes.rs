use std::marker::PhantomData;


struct A<'a> {
    mark: PhantomData<&'a ()>,
}

fn main() {


    let _ = {
        let a = A { mark: Default::default() };
        a
    };
}