mod meta;
mod transpile_runner;

use std::path::{Path, PathBuf};

use self::meta::{CompilerSettings, TestCaseContent, TestUnitData};
pub use self::transpile_runner::{TranspileRunner, TypeScriptTranspileCase};
use crate::suite::{Case, Suite, TestResult};

const TESTS_ROOT: &str = "typescript/tests";

pub struct TypeScriptSuite<T: Case> {
    test_root: PathBuf,
    test_cases: Vec<T>,
}

impl<T: Case> TypeScriptSuite<T> {
    pub fn new() -> Self {
        Self { test_root: PathBuf::from(TESTS_ROOT).join("cases"), test_cases: vec![] }
    }
}

impl<T: Case> Suite<T> for TypeScriptSuite<T> {
    fn get_test_root(&self) -> &Path {
        &self.test_root
    }

    fn skip_test_path(&self, path: &Path) -> bool {
        // stack overflows in compiler tests
        #[cfg(any(coverage, coverage_nightly))]
        let supported_paths = ["conformance"].iter().any(|p| path.to_string_lossy().contains(p));
        #[cfg(not(any(coverage, coverage_nightly)))]
        let supported_paths =
            ["conformance", "compiler"].iter().any(|p| path.to_string_lossy().contains(p));
        let unsupported_tests = [
            // these 2 relies on the ts "target" option
            "functionWithUseStrictAndSimpleParameterList.ts",
            "parameterInitializerBeforeDestructuringEmit.ts",
            // these also relies on "target: es5" option w/ RegExp `u` flag
            "unicodeExtendedEscapesInRegularExpressions01.ts",
            "unicodeExtendedEscapesInRegularExpressions02.ts",
            "unicodeExtendedEscapesInRegularExpressions03.ts",
            "unicodeExtendedEscapesInRegularExpressions04.ts",
            "unicodeExtendedEscapesInRegularExpressions05.ts",
            "unicodeExtendedEscapesInRegularExpressions06.ts",
            "unicodeExtendedEscapesInRegularExpressions08.ts",
            "unicodeExtendedEscapesInRegularExpressions09.ts",
            "unicodeExtendedEscapesInRegularExpressions10.ts",
            "unicodeExtendedEscapesInRegularExpressions11.ts",
            "unicodeExtendedEscapesInRegularExpressions13.ts",
            "unicodeExtendedEscapesInRegularExpressions15.ts",
            "unicodeExtendedEscapesInRegularExpressions16.ts",
            "unicodeExtendedEscapesInRegularExpressions18.ts",
        ]
        .iter()
        .any(|p| path.to_string_lossy().contains(p));
        !supported_paths || unsupported_tests
    }

    fn save_test_cases(&mut self, tests: Vec<T>) {
        self.test_cases = tests;
    }

    fn get_test_cases(&self) -> &Vec<T> {
        &self.test_cases
    }

    fn get_test_cases_mut(&mut self) -> &mut Vec<T> {
        &mut self.test_cases
    }
}

pub struct TypeScriptCase {
    path: PathBuf,
    pub code: String,
    pub units: Vec<TestUnitData>,
    pub settings: CompilerSettings,
    error_codes: Vec<String>,
    pub result: TestResult,
}

impl Case for TypeScriptCase {
    fn new(path: PathBuf, code: String) -> Self {
        let TestCaseContent { tests, settings, error_codes } =
            TestCaseContent::make_units_from_test(&path, &code);
        Self { path, code, units: tests, settings, error_codes, result: TestResult::ToBeRun }
    }

    fn code(&self) -> &str {
        &self.code
    }

    fn path(&self) -> &Path {
        &self.path
    }

    fn test_result(&self) -> &TestResult {
        &self.result
    }

    fn should_fail(&self) -> bool {
        // If there are still error codes to be supported, it should fail
        self.error_codes.iter().any(|code| !NOT_SUPPORTED_ERROR_CODES.contains(code.as_str()))
    }

    fn always_strict(&self) -> bool {
        self.settings.always_strict
    }

    fn run(&mut self) {
        let result = self
            .units
            .iter()
            .map(|unit| self.parse(&unit.content, unit.source_type))
            .find(Result::is_err)
            .unwrap_or(Ok(()));
        self.result = self.evaluate_result(result);
    }
}

// TODO: Filter out more not-supported error codes here
static NOT_SUPPORTED_ERROR_CODES: phf::Set<&'static str> = phf::phf_set![
    "2315", // Type 'U' is not generic.
];
