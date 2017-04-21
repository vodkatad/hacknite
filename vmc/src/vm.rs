use std::str::FromStr;
use std::fs;
use std::io::Write;
use std::io::{BufWriter,BufRead,BufReader};
use std::path::Path;
use std::cell::Cell;

pub struct VM {
    path: String,
    label_base: String,
    label_counter: Cell<usize>
}

impl VM {
    pub fn new(path: &str) -> VM {
        let stem = Path::new(path).file_stem().unwrap().to_str().unwrap().to_owned();

        VM { path: path.to_owned(), label_base: stem, label_counter: Cell::new(0) }
    }

    pub fn parse(&self, writer: &mut BufWriter<fs::File>) {
        let file = fs::File::open(&self.path).expect(&format!("could not open {} for reading", self.path));
        let mut reader = BufReader::new(file);
        let mut buffer = String::new();

        while reader.read_line(&mut buffer).unwrap() > 0 {
            let command = parse_line(&buffer);
            match command {
                VmCommand::None => (),
                VmCommand::Arithmetic(command) => self.emit_arithmetic(writer, command),
                VmCommand::Push(segment, value) => self.emit_push(writer, segment, value),
                VmCommand::Pop(segment, value) => self.emit_pop(writer, segment, value),
            }
            buffer.clear();
        }
    }

    fn emit_arithmetic(&self, writer: &mut BufWriter<fs::File>, command: VmArithmeticCommand) {
        let text = match command {
            VmArithmeticCommand::Add => format!(include_str!("templates/add.asm")),
            VmArithmeticCommand::Sub => format!(include_str!("templates/sub.asm")),
            VmArithmeticCommand::Neg => format!(include_str!("templates/neg.asm")),
            VmArithmeticCommand::And => format!(include_str!("templates/and.asm")),
            VmArithmeticCommand::Or  => format!(include_str!("templates/or.asm")),
            VmArithmeticCommand::Not => format!(include_str!("templates/not.asm")),
            VmArithmeticCommand::Eq  => format!(include_str!("templates/eq.asm"),
                                                label_false = self.get_label()),
            VmArithmeticCommand::Gt  => format!(include_str!("templates/gt.asm"),
                                                label_true = self.get_label(),
                                                label_end = self.get_label()),
            VmArithmeticCommand::Lt  => format!(include_str!("templates/lt.asm"),
                                                label_true = self.get_label(),
                                                label_end = self.get_label()),
        };
        writer.write(text.as_bytes()).expect("could not write to output buffer");
    }

    fn emit_push(&self, writer: &mut BufWriter<fs::File>, segment: VmSegment, value: i32) {
        let text = match segment {
            VmSegment::Constant => format!(include_str!("templates/push_constant.asm"), value = value),

            VmSegment::Argument => format!(include_str!("templates/push_segment.asm"), segment = "ARG", index = value),
            VmSegment::Local => format!(include_str!("templates/push_segment.asm"), segment = "LCL", index = value),
            VmSegment::That => format!(include_str!("templates/push_segment.asm"), segment = "THAT", index = value),
            VmSegment::This => format!(include_str!("templates/push_segment.asm"), segment = "THIS", index = value),

            VmSegment::Pointer => format!(include_str!("templates/push_static.asm"), index = 3 + value),
            VmSegment::Temp => format!(include_str!("templates/push_static.asm"), index = 5 + value),
            VmSegment::Static => format!(include_str!("templates/push_static.asm"), index = 16 + value),
        };
        writer.write(text.as_bytes()).expect("could not write to output buffer");
    }

    fn emit_pop(&self, writer: &mut BufWriter<fs::File>, segment: VmSegment, value: i32) {
        let text = match segment {
            VmSegment::Constant => panic!("thou shall not use 'pop constant'"),

            VmSegment::Argument => format!(include_str!("templates/pop_segment.asm"), segment = "ARG", index = value),
            VmSegment::Local => format!(include_str!("templates/pop_segment.asm"), segment = "LCL", index = value),
            VmSegment::That => format!(include_str!("templates/pop_segment.asm"), segment = "THAT", index = value),
            VmSegment::This => format!(include_str!("templates/pop_segment.asm"), segment = "THIS", index = value),

            VmSegment::Pointer => format!(include_str!("templates/pop_static.asm"), index = 3 + value),
            VmSegment::Temp => format!(include_str!("templates/pop_static.asm"), index = 5 + value),
            VmSegment::Static => format!(include_str!("templates/pop_static.asm"), index = 16 + value),
        };

        writer.write(text.as_bytes()).expect("could not write to output buffer");
    }

    fn get_label(&self) -> String {
        let v = self.label_counter.get();
        let label = format!("{}.{}", self.label_base, v);
        self.label_counter.set(v+1);
        label
    }
}

fn parse_line(line: &str) -> VmCommand {
    let parts = line.split_whitespace().collect::<Vec<_>>();

    if parts.len() > 0 && parts[0] != "//" {
        match parts[0] {
            "neg" => VmCommand::Arithmetic(VmArithmeticCommand::Neg),
            "add" => VmCommand::Arithmetic(VmArithmeticCommand::Add),
            "sub" => VmCommand::Arithmetic(VmArithmeticCommand::Sub),
            "eq"  => VmCommand::Arithmetic(VmArithmeticCommand::Eq),
            "gt"  => VmCommand::Arithmetic(VmArithmeticCommand::Gt),
            "lt"  => VmCommand::Arithmetic(VmArithmeticCommand::Lt),
            "not" => VmCommand::Arithmetic(VmArithmeticCommand::Not),
            "and" => VmCommand::Arithmetic(VmArithmeticCommand::And),
            "or"  => VmCommand::Arithmetic(VmArithmeticCommand::Or),

            "push" => VmCommand::Push(parse_vmsegment(parts[1]), i32::from_str(parts[2]).unwrap()),
            "pop"  => VmCommand::Pop(parse_vmsegment(parts[1]), i32::from_str(parts[2]).unwrap()),

            _ => panic!("unknown command: {}", parts[0])
        }
    }
    else {
        VmCommand::None
    }
}

fn parse_vmsegment(segment: &str) -> VmSegment {
    match segment {
        "argument" => VmSegment::Argument,
        "local" => VmSegment::Local,
        "static" => VmSegment::Static,
        "constant" => VmSegment::Constant,
        "this" => VmSegment::This,
        "that" => VmSegment::That,
        "pointer" => VmSegment::Pointer,
        "temp" => VmSegment::Temp,
        _ => panic!("unknown segment: {}", segment)
    }
}

#[derive(Debug)]
enum VmCommand {
    None,
    Arithmetic(VmArithmeticCommand),
    Push(VmSegment, i32),
    Pop(VmSegment, i32)
}

#[derive(Debug)]
enum VmArithmeticCommand {
    Neg,
    Add,
    Sub,
    Eq,
    Gt,
    Lt,
    Not,
    And,
    Or
}

#[derive(Debug)]
enum VmSegment {
    Argument,
    Local,
    Static,
    Constant,
    This,
    That,
    Pointer,
    Temp
}
