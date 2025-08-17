use std::io::Error as IoError;
use std::io::Write;

use crate::errors::FindItError;
use crate::expr::Evaluator;
use crate::expr::read_expr;
use crate::{cli_args::CliArgs, file_wrapper::FileWrapper, limit::make_limit, walker::Walk};

pub(crate) fn build_output<W: Write + 'static>(
    args: &CliArgs,
    writer: W,
) -> Result<Box<dyn Walk>, FindItError> {
    let next = make_limit(args);
    match &args.display {
        None => Ok(Box::new(SimpleOutput { next, writer })),
        Some(display) => {
            let fields = parse_display(
                "display",
                display,
                &args.interpolation_start,
                &args.interpolation_end,
            )?;
            Ok(Box::new(ComplexOutput {
                next,
                fields,
                writer,
            }))
        }
    }
}

struct SimpleOutput<W: Write> {
    next: Option<Box<dyn Walk>>,
    writer: W,
}

impl<W: Write> Walk for SimpleOutput<W> {
    fn step(&mut self, file: &FileWrapper) {
        writeln!(&mut self.writer, "{file}").ok();
        if let Some(next) = self.next.as_deref_mut() {
            next.step(file);
        }
    }
    fn enough(&self) -> bool {
        if let Some(next) = self.next.as_deref() {
            next.enough()
        } else {
            false
        }
    }
}

enum OutputField {
    Static(String),
    Dynamic(Box<dyn Evaluator>),
}
impl OutputField {
    fn write<W: Write>(&self, w: &mut W, file: &FileWrapper) -> Result<(), IoError> {
        match self {
            OutputField::Static(str) => write!(w, "{str}"),
            OutputField::Dynamic(e) => {
                let v = e.eval(file);
                write!(w, "{v}")
            }
        }
    }
}
struct ComplexOutput<W: Write> {
    next: Option<Box<dyn Walk>>,
    fields: Vec<OutputField>,
    writer: W,
}
impl<W: Write> Walk for ComplexOutput<W> {
    fn enough(&self) -> bool {
        if let Some(next) = self.next.as_deref() {
            next.enough()
        } else {
            false
        }
    }
    fn step(&mut self, file: &FileWrapper) {
        for f in &self.fields {
            f.write(&mut self.writer, file).ok();
        }
        writeln!(&mut self.writer).ok();
        if let Some(next) = self.next.as_deref_mut() {
            next.step(file);
        }
    }
}
fn parse_display(
    parse_type: &str,
    display_string: &str,
    interpolation_start: &str,
    interpolation_end: &str,
) -> Result<Vec<OutputField>, FindItError> {
    if display_string.is_empty() {
        return Err(FindItError::DisplayParserError(
            parse_type.into(),
            "Empty String".into(),
        ));
    }
    if interpolation_start.is_empty() {
        return Err(FindItError::DisplayParserError(
            parse_type.into(),
            "Empty interpolation start".into(),
        ));
    }
    if interpolation_end.is_empty() {
        return Err(FindItError::DisplayParserError(
            parse_type.into(),
            "Empty interpolation end".into(),
        ));
    }
    let mut fields: Vec<OutputField> = vec![];
    let mut str = display_string;
    while !str.is_empty() {
        let next_int_start = str.find(interpolation_start).unwrap_or(str.len());
        if next_int_start > 0 {
            let str = str[0..next_int_start].to_string();
            fields.push(OutputField::Static(str));
        }
        if next_int_start < str.len() {
            let Some(end) =
                str[next_int_start + interpolation_start.len()..].find(interpolation_end)
            else {
                return Err(FindItError::DisplayParserError(
                    parse_type.into(),
                    "never ending interpolation".into(),
                ));
            };
            let extractor = read_expr(
                &str[next_int_start + interpolation_start.len()
                    ..next_int_start + interpolation_start.len() + end],
            )?;
            fields.push(OutputField::Dynamic(extractor));
            str =
                &str[next_int_start + interpolation_start.len() + end + interpolation_end.len()..];
        } else {
            str = "";
        }
    }

    Ok(fields)
}
