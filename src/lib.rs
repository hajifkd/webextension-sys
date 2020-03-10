extern crate js_sys;
extern crate wasm_bindgen;

pub mod ext;

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
