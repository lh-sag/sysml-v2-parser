use std::fs;
fn main() {
    let p = "tests/fixtures/SurveillanceDrone-error.sysml";
    let input = fs::read_to_string(p).unwrap().replace("\r\n","\n");
    match sysml_v2_parser::parse(&input) {
        Ok(r) => println!("OK elements={}", r.elements.len()),
        Err(e) => println!("ERR line={:?} code={:?} msg={} found={:?}", e.line, e.code, e.message, e.found),
    }
}
