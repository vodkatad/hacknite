extern crate argparse;

mod vm;

use argparse::{ArgumentParser, Collect, Store};
use std::fs;
use std::io::{BufWriter};
use vm::VM;

fn main() {
    let mut out = "out.asm".to_string();
    let mut files: Vec<String> = Vec::new();
    {
        let mut ap = ArgumentParser::new();
        ap.set_description("Compile hack VM files to asm.");
        ap.refer(&mut out).add_option(&["-o", "--out"], Store, "output file");
        ap.refer(&mut files).add_argument("FILES", Collect, "input files");
        ap.parse_args_or_exit();
    }

    let file = fs::File::create(&out).expect(&format!("could not open {} for writing", &out));
    let mut writer = BufWriter::new(file);

    for file in files {
        let vm = VM::new(&file);
        vm.parse(&mut writer);
    }
}
