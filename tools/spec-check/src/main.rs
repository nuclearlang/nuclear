// Reads the ```nuclear blocks of a markdown file and prints, for each, its output in a dialect
// (even when it uses names declared elsewhere: N0106 is ignored). Used by check.py.

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let (md, dialect) = (&args[1], &args[2]);
    let loader = nuclear_dialect::Loader::new(nuclear_dialect::dialect_dirs(std::path::Path::new(concat!(env!("CARGO_MANIFEST_DIR"), "/../.."))));
    let d = loader.load(dialect).0.expect("dialect");
    let text = std::fs::read_to_string(md).unwrap();
    let mut blocks = Vec::new();
    let mut cur: Option<String> = None;
    for line in text.lines() {
        match (&mut cur, line.trim_end()) {
            (None, "```nuclear") => cur = Some(String::new()),
            (Some(_), "```") => blocks.push(cur.take().unwrap()),
            (Some(b), l) => {
                b.push_str(l);
                b.push('\n');
            }
            _ => {}
        }
    }
    for b in blocks {
        let out = nuclear_transpile::transpile(&b, &d);
        let real: Vec<_> = out.diagnostics.iter().filter(|x| x.is_error() && x.code.id != "N0106").map(|x| x.message.clone()).collect();
        println!("<<<NUCLEAR\n{b}>>>OUT\n{}", out.code);
        if !real.is_empty() {
            println!("!!! {real:?}");
        }
        println!("<<<END");
    }
}
