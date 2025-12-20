use std::env;
use std::io::{IsTerminal, Write, stdout};
use std::process::{Command, Stdio};

use crate::errors::FindItError;

const BOLD: &str = "\x1b[1m";
const RESET: &str = "\x1b[0m";
const BLUE: &str = "\x1b[34m";
fn get_syntax_help(term: bool) -> String {
    let (bold, blue, reset) = if term {
        (BOLD, BLUE, RESET)
    } else {
        ("", "", "")
    };
    format!(
        r##"
{bold}{blue}findit Expression Syntax - Quick Reference{reset}
===========================================

{bold}CASE SENSITIVITY{reset}:
  Keywords, properties, functions, methods are case-insensitive
  String literals ARE case-sensitive: "txt" ≠ "TXT"

{bold}LITERALS:{reset}
  Numbers:     123  0x1F  0o755  0b1010
  Strings:     "text"  "escape: \n \t \""
  Booleans:    true  false
  Dates:       @(2025-12-19)  @(19/Dec/2025 14:30)
  Paths:       @src  @"my file.txt"
  Lists:       [1, 2, 3]  ["a", "b"]
  Classes:     {{:name "value", :count 42}}

{bold}FILE PROPERTIES:{reset}
  name         File name with extension
  stem         File name without extension
  extension    File extension (without dot)
  path         Full file path as string
  absolute     Absolute path
  size         File size in bytes
  depth        Directory depth (root = 0)
  content      File content as string (empty if binary/unreadable)
  created      Creation date/time
  modified     Last modification date/time
  owner        File owner username
  group        File group name
  permission   File permissions (numeric)
  parent       Parent directory path
  files        List of files in directory
  
  IS FILE      True if regular file
  IS DIR       True if directory
  IS LINK      True if symbolic link
  exists       True if file exists

{bold}COMPARISON OPERATORS:{reset}
  =  ==        Equal
  != <>        Not equal
  <            Less than
  >            Greater than
  <=           Less than or equal
  >=           Greater than or equal
  BETWEEN      value BETWEEN min AND max

{bold}LOGICAL OPERATORS:{reset}
  AND          Both conditions true
  OR           At least one condition true
  NOT          Negates condition (suffix: "x NOT")
  XOR          Exactly one condition true

{bold}ARITHMETIC OPERATORS:{reset}
  +            Addition
  -            Subtraction
  *            Multiplication
  /            Division
  %            Modulo (remainder)
  &            Bitwise AND
  |            Bitwise OR
  ^            Bitwise XOR

{bold}STRING OPERATORS:{reset}
  +            Concatenation: "hello" + " " + "world"
  MATCHES      Regular expression: name MATCHES "^test.*\.rs$"

{bold}TYPE OPERATORS:{reset}
  IS SOME      Value is not empty
  IS NONE      Value is empty
  IS TRUE      Boolean is true
  IS FALSE     Boolean is false
  AS STRING    Convert to string
  AS NUMBER    Convert to number
  AS BOOLEAN   Convert to boolean
  AS DATE      Convert to date
  AS PATH      Convert to path

{bold}STRING METHODS:{reset}
  .length()            Number of characters
  .contains("text")    True if contains substring
  .toLower()           Convert to lowercase
  .toUpper()           Convert to uppercase
  .trim()              Remove leading/trailing whitespace
  .split(",")          Split into list
  .lines()             Split by newlines
  .words()             Split by whitespace
  .reverse()           Reverse string
  .hasPrefix("pre")    True if starts with prefix
  .hasSuffix("suf")    True if ends with suffix

{bold}LIST METHODS:{reset}
  .length()                Number of items
  .first()                 First item
  .last()                  Last item
  .contains(x)             True if contains item
  .filter($x <expr>)       Filter items
  .map($x <expr>)          Transform items
  .sort()                  Sort items
  .sortBy($x <expr>)       Sort by expression
  .distinct()              Remove duplicates
  .sum()                   Sum of numbers
  .max()                   Maximum value
  .min()                   Minimum value
  .avg()                   Average value
  .take(n)                 First n items
  .skip(n)                 Skip first n items
  .join(",")               Join into string

{bold}PATH METHODS:{reset}
  .lines()     File content as list of lines
  .words()     File content as list of words
  .walk()      All descendant files/directories
  .length()    Size in bytes

{bold}FUNCTIONS:{reset}
  now()                               Current timestamp
  env("VAR")                          Environment variable
  rand()                              Random number
  coalesce(a, b, c)                   First non-empty value
  replace(str FROM old TO new)        Replace in string
  replace(str PATTERN regex TO new)   Replace in string
  execute(cmd, args)                  Execute external command

{bold}CONTROL FLOW:{reset}
  IF condition THEN a ELSE b END
  CASE WHEN c1 THEN a WHEN c2 THEN b ELSE c END
  WITH $var AS value DO expression END

{bold}EXAMPLES:{reset}
  # Files by extension
  extension = "rs"
  
  # Large files
  size > 10485760
  
  # Recent files (last 24 hours)
  modified > now() - 86400
  
  # Find in content
  content.contains("TODO")
  
  # Complex filter
  extension = "txt" AND size BETWEEN 1024 AND 1048576
  
  # Executable files
  NOT IS DIR AND permission & 0o111 != 0
  
  # Files without tests
  IS FILE AND NOT content.contains("#[cfg(test)]")
  
  # Count files by extension
  IS DIR AND files.filter($f $f.IS FILE).groupBy($f $f.extension)

{bold}TIPS:{reset}
  - Use single quotes around expressions in shell: -w 'size > 1024'
  - Variables start with $: WITH $x AS size DO $x * 2 END
  - Methods can chain: content.toLower().contains("todo")
  - Omit parentheses for no-arg methods: name.length vs name.length()
  - Use @path for path literals: @src/main.rs

{bold}FULL DOCUMENTATION:{reset}
  https://github.com/yift/findit/blob/main/docs/syntax/index.md
  https://github.com/yift/findit/blob/main/docs/usage.md
"##
    )
}

pub(crate) trait Executor {
    fn spawn(&self, program: &str, args: &[&str], input: &[u8]) -> Result<(), FindItError>;
}
pub(crate) trait Pager {
    fn pager(&self) -> String;
}

struct DefaultExecutor;
impl Executor for DefaultExecutor {
    fn spawn(&self, program: &str, args: &[&str], input: &[u8]) -> Result<(), FindItError> {
        let mut child = Command::new(program)
            .args(args)
            .stdin(Stdio::piped())
            .spawn()?;

        if let Some(mut stdin) = child.stdin.take() {
            stdin.write_all(input)?;
        }

        let status = child.wait()?;

        if status.success() {
            Ok(())
        } else {
            Err(FindItError::PagerFailed(status))
        }
    }
}
struct DefaultPager;
impl Pager for DefaultPager {
    fn pager(&self) -> String {
        env::var("PAGER").unwrap_or_else(|_| "less".to_string())
    }
}

pub(crate) fn default_executor() -> impl Executor {
    DefaultExecutor
}
pub(crate) fn default_pager() -> impl Pager {
    DefaultPager
}

pub(crate) fn show_syntax_help(pager: impl Pager, executor: impl Executor) {
    if !stdout().is_terminal() || show_with_pager(pager, executor).is_err() {
        print!("{}", get_syntax_help(false));
    }
}

fn show_with_pager(pager: impl Pager, executor: impl Executor) -> Result<(), FindItError> {
    let pager_cmd = pager.pager();

    executor.spawn(&pager_cmd, &["-R", "-F"], get_syntax_help(true).as_bytes())
}

#[cfg(test)]
mod tests {
    use std::{
        cell::RefCell,
        fs::{self, File},
        os::unix::fs::PermissionsExt,
        rc::Rc,
    };

    use tempfile::tempdir;

    use crate::errors::FindItError;

    use super::*;

    #[test]
    fn get_syntax_help_no_format() -> Result<(), FindItError> {
        let help = get_syntax_help(false);
        assert!(help.contains("findit Expression Syntax - Quick Reference"));
        assert!(!help.contains(RESET));
        Ok(())
    }

    #[test]
    fn get_syntax_help_with_format() -> Result<(), FindItError> {
        let help = get_syntax_help(true);
        assert!(help.contains("findit Expression Syntax - Quick Reference"));
        assert!(help.contains(RESET));
        Ok(())
    }

    #[test]
    fn test_default_pager() -> Result<(), FindItError> {
        let cmd = default_pager().pager();

        assert_eq!(cmd, "less".to_string());
        Ok(())
    }

    #[test]
    fn test_default_executor() -> Result<(), FindItError> {
        let dir = tempdir()?;
        let script_path = dir.path().join("capture.sh");
        let output = dir.path().join("output.txt");

        let mut file = File::create(&script_path)?;
        writeln!(
            file,
            r#"#!/bin/bash
cat - > "$1"
"#
        )?;
        drop(file);
        let mut perms = fs::metadata(&script_path)?.permissions();
        perms.set_mode(0o755);
        fs::set_permissions(&script_path, perms)?;

        default_executor().spawn(
            script_path.display().to_string().as_str(),
            &[output.display().to_string().as_str()],
            b"hello world",
        )?;

        let out = fs::read_to_string(&output)?;
        assert_eq!(out, "hello world");
        Ok(())
    }

    #[test]
    fn test_default_executor_with_error() -> Result<(), FindItError> {
        let dir = tempdir()?;
        let script_path = dir.path().join("capture.sh");
        let output = dir.path().join("output.txt");

        let mut file = File::create(&script_path)?;
        writeln!(
            file,
            r#"#!/bin/bash
cat - > "$1"
exit 1
"#
        )?;
        drop(file);
        let mut perms = fs::metadata(&script_path)?.permissions();
        perms.set_mode(0o755);
        fs::set_permissions(&script_path, perms)?;

        let err = default_executor()
            .spawn(
                script_path.display().to_string().as_str(),
                &[output.display().to_string().as_str()],
                b"hello world",
            )
            .err();

        let out = fs::read_to_string(&output)?;
        assert_eq!(out, "hello world");
        assert!(err.is_some());
        Ok(())
    }

    #[test]
    fn test_show_with_pager() -> Result<(), FindItError> {
        struct TestPager;
        impl Pager for TestPager {
            fn pager(&self) -> String {
                "cat".to_string()
            }
        }
        struct TestExecutor {
            called: Rc<RefCell<usize>>,
        }
        let called = Rc::new(RefCell::new(0));
        impl Executor for TestExecutor {
            fn spawn(&self, program: &str, args: &[&str], input: &[u8]) -> Result<(), FindItError> {
                assert_eq!(program, "cat");
                assert_eq!(args, &["-R", "-F"]);
                assert!(input.starts_with(b"\n\x1b[1m\x1b[34mfindit Expression Syntax"));
                *self.called.borrow_mut() += 1;
                Ok(())
            }
        }

        show_with_pager(
            TestPager,
            TestExecutor {
                called: called.clone(),
            },
        )?;

        assert_eq!(*called.borrow(), 1);
        Ok(())
    }
}
