use std::{
    env,
    fmt::{Debug, Display},
    fs::{self, File},
    io::{Result as IoResult, Write},
    iter,
    path::{Path, PathBuf},
    process::Command,
    sync::{Arc, Mutex},
};

use clap::Parser;
use findit_cli::{cli_args::CliArgs, errors::FindItError, run_func::run};
use itertools::Itertools;
use toml::{Table, Value, map::Map};

struct TestWriter {
    written: Vec<u8>,
    data: Arc<Mutex<Vec<u8>>>,
}
impl Write for TestWriter {
    fn write(&mut self, buf: &[u8]) -> IoResult<usize> {
        self.written.extend_from_slice(buf);
        Ok(buf.len())
    }
    fn flush(&mut self) -> IoResult<()> {
        let mut data = self.data.lock().unwrap();
        data.extend_from_slice(&self.written);
        self.written.clear();
        Ok(())
    }
}
impl Drop for TestWriter {
    fn drop(&mut self) {
        self.flush().ok();
    }
}

#[derive(Default)]
struct TestWriterFactory {
    data: Arc<Mutex<Vec<u8>>>,
}

impl TestWriterFactory {
    fn make_writer(&self) -> TestWriter {
        TestWriter {
            written: vec![],
            data: self.data.clone(),
        }
    }
}
impl Display for TestWriterFactory {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let data = self.data.lock().unwrap();
        Display::fmt(str::from_utf8(&data).unwrap(), f)
    }
}

#[test]
fn integration_tests() -> Result<(), FindItError> {
    let tests_files = find_integration_tests_files("tests/test_cases")?;
    let dir = env::current_dir()?;
    for test_file in tests_files {
        println!(
            "Looking at tests defined in: {}",
            test_file.as_os_str().to_str().unwrap_or_default()
        );
        let content = std::fs::read_to_string(&test_file)?;
        let cfg = content.parse::<Table>().unwrap();
        for (key, val) in &cfg {
            let defs = val.as_table().unwrap();

            integration_test(key, &test_file, defs)?;
            env::set_current_dir(&dir)?;
        }
    }

    Ok(())
}

fn integration_test(name: &str, root: &Path, defs: &Map<String, Value>) -> Result<(), FindItError> {
    println!("Testing: {name}");

    let root = fs::canonicalize(root)?;

    if let Some(run_before) = defs.get("run_before").and_then(|p| p.as_array()) {
        run_bash(run_before)?;
    }
    if let Some(dir) = defs.get("root").and_then(|f| f.as_str()) {
        env::set_current_dir(dir)?
    }
    let writer = TestWriterFactory::default();
    let args = iter::once("-").chain(
        defs.get("arguments")
            .unwrap()
            .as_array()
            .unwrap()
            .iter()
            .map(|a| a.as_str().unwrap()),
    );
    let args = CliArgs::parse_from(args);
    run(&args, writer.make_writer())?;

    let dir = root
        .parent()
        .unwrap()
        .join(root.with_extension("").file_name().unwrap())
        .join("expected_output");
    fs::create_dir_all(&dir).ok();
    let expected_output_file = dir.join(format!("{}.txt", name));
    if env::var("CREATE_RESULTS").is_ok() {
        let mut file = File::create(&expected_output_file)?;
        file.write_all(writer.to_string().as_bytes())?;
    }

    let order = defs.get("order").and_then(|o| o.as_bool()).unwrap_or(true);
    let expected_content = fs::read_to_string(expected_output_file)?;

    if order {
        assert_eq!(expected_content, writer.to_string());
    } else {
        let output: Vec<String> = writer
            .to_string()
            .lines()
            .sorted()
            .map(|f| f.to_string())
            .collect();
        let expected_content: Vec<String> = expected_content
            .lines()
            .sorted()
            .map(|f| f.to_string())
            .collect();
        assert_eq!(expected_content, output);
    }

    if let Some(run_after) = defs.get("run_after").and_then(|p| p.as_array()) {
        run_bash(run_after)?;
    }

    Ok(())
}

fn find_integration_tests_files<P: AsRef<Path> + Debug>(
    path: P,
) -> Result<Vec<PathBuf>, FindItError> {
    let mut tests = vec![];
    let paths = fs::read_dir(path)?;
    for path in paths {
        let path = path?.path();
        if path.is_dir() {
            let more = find_integration_tests_files(&path)?;
            tests.extend_from_slice(&more);
        } else if path
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or_default()
            == "toml"
        {
            tests.push(path);
        }
    }
    Ok(tests)
}

fn run_bash(args: &[Value]) -> Result<(), FindItError> {
    let arguments: String = args.iter().map(|f| f.as_str().unwrap()).join(" && ");
    let mut proc = Command::new("bash").arg("-c").arg(&arguments).spawn()?;
    let exit_code = proc.wait()?;
    if !exit_code.success() {
        Err(FindItError::BadExpression(arguments.to_string()))
    } else {
        Ok(())
    }
}
