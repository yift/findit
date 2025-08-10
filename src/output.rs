use crate::errors::FindItError;
use crate::expr::Evaluator;
use crate::expr::read_expr;
use crate::{cli_args::CliArgs, file_wrapper::FileWrapper, limit::make_limit, walker::Walk};

pub(crate) fn build_output(args: &CliArgs) -> Result<Box<dyn Walk>, FindItError> {
    let next = make_limit(args);
    match &args.display {
        None => Ok(Box::new(SimpleOutput { next })),
        Some(display) => {
            let fields = parse_display(
                "display",
                display,
                &args.interpolation_start,
                &args.interpolation_end,
            )?;
            Ok(Box::new(ComplexOutput { fields, next }))
        }
    }
}

struct SimpleOutput {
    next: Option<Box<dyn Walk>>,
}

impl Walk for SimpleOutput {
    fn step(&mut self, file: &FileWrapper) {
        println!("{}", file);
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

trait OutputField {
    fn print(&self, file: &FileWrapper);
}

struct StaticField {
    str: String,
}
impl OutputField for StaticField {
    fn print(&self, _: &FileWrapper) {
        print!("{}", self.str)
    }
}

struct DynamicField {
    extractor: Box<dyn Evaluator>,
}
impl OutputField for DynamicField {
    fn print(&self, file: &FileWrapper) {
        print!("{}", self.extractor.eval(file))
    }
}
struct ComplexOutput {
    next: Option<Box<dyn Walk>>,
    fields: Vec<Box<dyn OutputField>>,
}
impl Walk for ComplexOutput {
    fn enough(&self) -> bool {
        if let Some(next) = self.next.as_deref() {
            next.enough()
        } else {
            false
        }
    }
    fn step(&mut self, file: &FileWrapper) {
        for f in &self.fields {
            f.print(file);
        }
        println!();
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
) -> Result<Vec<Box<dyn OutputField>>, FindItError> {
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
    let mut fields: Vec<Box<dyn OutputField>> = vec![];
    let mut str = display_string;
    while !str.is_empty() {
        let next_int_start = str.find(interpolation_start).unwrap_or(str.len());
        if next_int_start > 0 {
            let str = str[0..next_int_start].to_string();
            fields.push(Box::new(StaticField { str }));
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
            fields.push(Box::new(DynamicField { extractor }));
            str = &str[next_int_start + interpolation_start.len() + end + interpolation_end.len()..]
        } else {
            str = ""
        }
    }

    Ok(fields)
}
