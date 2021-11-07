use std::path::{Path, PathBuf};

use lazy_static::lazy_static;
use regex::Regex;
use walkdir::WalkDir;

use rust_crafting_interpreters_lib::interpreter::{
    tree_walk::LoxLinePrinter, LoxInterpreter, LoxTreeWalkInterpreter,
};

pub fn discover_tests<P: AsRef<Path>>(root: P) -> Vec<PathBuf> {
    let mut paths = vec![];
    for entry in WalkDir::new(root).into_iter().filter_map(|e| e.ok()) {
        let entry_path = entry.path();
        if let Some(entry_extension) = entry_path.extension() {
            if entry_extension == "lox" {
                paths.push(entry_path.to_path_buf());
            }
        }
    }
    paths
}

pub enum LoxAutoTestAssertion {
    ExpectOutput(String),
}

impl LoxAutoTestAssertion {
    pub fn is_output(&self) -> bool {
        matches!(self, Self::ExpectOutput(_))
    }

    pub fn as_output(&self) -> Option<&String> {
        match self {
            Self::ExpectOutput(output) => Some(output),
        }
    }
}

pub struct LoxAutoTestSuite {
    code: String,
    path: PathBuf,
    asserts: Vec<LoxAutoTestAssertion>,
}

impl LoxAutoTestSuite {
    pub fn from_code(path: PathBuf, code: String) -> Result<Self, String> {
        lazy_static! {
            static ref ASSERT_OUTPUT_REGEX: Regex = Regex::new("// expect: ?(.*)").unwrap();
        }

        let mut asserts = vec![];
        for captures in ASSERT_OUTPUT_REGEX.captures_iter(&code) {
            let expected = captures.get(1).unwrap().as_str();
            asserts.push(LoxAutoTestAssertion::ExpectOutput(expected.into()));
        }
        Ok(Self {
            path,
            code,
            asserts,
        })
    }
}

#[derive(Default)]
pub struct HistoryPrinter {
    outputs: Vec<String>,
}

impl LoxLinePrinter for HistoryPrinter {
    fn print(&mut self, output: String) {
        self.outputs.push(output);
    }

    fn history(&self) -> Option<&[String]> {
        Some(&self.outputs)
    }
}

pub struct LoxAutoTestHarness {
    outputs: Vec<String>,
    interpreter: LoxTreeWalkInterpreter,
}

impl Default for LoxAutoTestHarness {
    fn default() -> Self {
        let mut outputs = vec![];
        Self {
            outputs,
            interpreter: LoxTreeWalkInterpreter::new(Some(Box::new(HistoryPrinter::default()))),
        }
    }
}

impl LoxAutoTestHarness {
    pub fn run_test_suite(&mut self, suite: &LoxAutoTestSuite) {
        let parsed = self
            .interpreter
            .parse(suite.code.clone())
            .expect("can parse the test suite's code");
        self.interpreter
            .interpret(&parsed)
            .expect("can interpret the test suite's code");

        self.run_assertions(suite);
    }

    fn run_assertions(&self, suite: &LoxAutoTestSuite) {
        assert_eq!(
            self.outputs.len(),
            suite
                .asserts
                .iter()
                .filter(|assertion| assertion.is_output())
                .count()
        );
        let mut output_assertions_count = 0;
        for output in &suite.asserts {
            if let Some(expected_output) = output.as_output() {
                assert_eq!(&self.outputs[output_assertions_count], expected_output);
                output_assertions_count += 1;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use std::fs::File;
    use std::io::Read;
    use std::path::{Path, PathBuf};

    use super::{discover_tests, LoxAutoTestHarness, LoxAutoTestSuite};

    /// For each tests group entry, detect all files and run their tests.
    ///
    /// We manually define each group entry instead of detecting them in order to
    /// allow for separate errors for each language domain.
    macro_rules! test_lox_suites_groups {
    ($ ( $name: ident : ($relative_root: literal), )* ) => {
        $(
            #[test]
            fn $name() {
                // discovery
                let root_path = Path::new("./tests/loxtests/").join($relative_root);
                let tests_paths = discover_tests(&root_path);
                let tests_tuples = tests_paths.iter().map(|test_path| {
                    let mut test_file = File::open(test_path).unwrap();
                    let mut test_source = String::new();
                    test_file.read_to_string(&mut test_source).unwrap();
                    (test_path.clone(), test_source)
                });
                // parsing
                let tests_suites = tests_tuples.map(|(test_path, test_source)| LoxAutoTestSuite::from_code(test_path.clone(), test_source).unwrap());
                // validation
                for test_suite in tests_suites {
                    let mut harness = LoxAutoTestHarness::default();
                    harness.run_test_suite(&test_suite);
                }
            }
        )*
        }
    }

    test_lox_suites_groups! {
        test_assignment: ("assignment"),
        test_block: ("block"),
        test_bool: ("bool"),
        test_call: ("call"),
        test_class: ("class"),
        test_closure: ("closure"),
        test_comments: ("comments"),
        test_constructor: ("constructor"),
        test_expressions: ("expressions"),
        test_field: ("field"),
        test_for_loops: ("for"),
        test_function: ("function"),
        test_if: ("if"),
        test_inheritance: ("inheritance"),
        // test_limit: ("limit"), // TODO: expect errors for this group
        test_logical_operator: ("logical_operator"),
        test_method: ("method"),
        test_nil: ("nil"),
        test_number: ("number"),
        test_operator: ("operator"),
        test_print: ("print"),
        test_regression: ("regression"),
        test_return: ("return"),
        test_scanning: ("scanning"),
        test_string: ("string"),
        test_super_class: ("super"),
        test_this: ("this"),
        test_variable: ("variable"),
        test_while: ("while"),
    }
}
