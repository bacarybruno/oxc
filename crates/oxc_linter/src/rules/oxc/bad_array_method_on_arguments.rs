use oxc_ast::{AstKind, ast::MemberExpression};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::Span;

use crate::{AstNode, context::LintContext, rule::Rule};

fn bad_array_method_on_arguments_diagnostic(method_name: &str, span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Bad array method on arguments")
        .with_help(format!(
            "The 'arguments' object does not have a '{method_name}()' method. If you intended to use an array method, consider converting the 'arguments' object to an array or using an ES6 rest parameter instead."
        ))
        .with_label(span)
}

#[derive(Debug, Default, Clone)]
pub struct BadArrayMethodOnArguments;

declare_oxc_lint!(
    /// ### What it does
    ///
    /// This rule applies when an array method is called on the arguments object itself.
    ///
    /// ### Why is this bad?
    ///
    /// The arguments object is not an array, but an array-like object. It should be converted to a real array before calling an array method.
    /// Otherwise, a TypeError exception will be thrown because of the non-existent method.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```javascript
    /// function add(x, y) {
    ///   return x + y;
    /// }
    /// function sum() {
    ///   return arguments.reduce(add, 0);
    /// }
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```javascript
    /// function add(x, y) {
    ///   return x + y;
    /// }
    /// function sum(...args) {
    ///   return args.reduce(add, 0);
    /// }
    /// ```
    BadArrayMethodOnArguments,
    oxc,
    correctness,
);

impl Rule for BadArrayMethodOnArguments {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        if !node.kind().is_specific_id_reference("arguments") {
            return;
        }
        let Some(parent) = ctx.nodes().parent_node(node.id()) else {
            return;
        };
        if !matches!(
            parent.kind(),
            AstKind::MemberExpression(_) | AstKind::ComputedMemberExpression(_)
        ) {
            return;
        }
        let member_expr = parent.kind();
        let Some(grandparent) = ctx.nodes().parent_node(parent.id()) else {
            return;
        };
        if matches!(member_expr, AstKind::ComputedMemberExpression(_)) {
            let great_grandparent = ctx.nodes().parent_kind(grandparent.id());
            let Some(AstKind::CallExpression(_)) = great_grandparent else {
                return;
            };
        } else if !matches!(member_expr, AstKind::ComputedMemberExpression(_)) {
            let AstKind::CallExpression(_) = grandparent.kind() else {
                return;
            };
        }
        match member_expr {
            AstKind::MemberExpression(MemberExpression::StaticMemberExpression(expr)) => {
                if ARRAY_METHODS.binary_search(&expr.property.name.as_str()).is_ok() {
                    ctx.diagnostic(bad_array_method_on_arguments_diagnostic(
                        expr.property.name.as_str(),
                        expr.span,
                    ));
                }
            }
            AstKind::ComputedMemberExpression(expr) => {
                let Some(name) = expr.static_property_name() else {
                    return;
                };
                if ARRAY_METHODS.binary_search(&name.as_str()).is_ok() {
                    ctx.diagnostic(bad_array_method_on_arguments_diagnostic(
                        name.as_str(),
                        expr.span,
                    ));
                }
            }
            _ => {}
        }
    }
}

/// `https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Array#instance_methods`
#[rustfmt::skip]
const ARRAY_METHODS: [&str; 32] = [
    "@@iterator",
    "at",
    "concat", "copyWithin",
    "entries", "every",
    "fill", "filter", "find", "findIndex", "flat", "flatMap", "forEach",
    "includes", "indexOf",
    "join",
    "keys",
    "lastIndexOf",
    "map",
    "pop", "push", "push",
    "reduce", "reduceRight", "reverse",
    "shift", "slice", "some", "sort", "splice",
    "unshift",
    "values",
];

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        ("function fn() {}", None),
        ("function fn(...args) {return args.reduce((prev, cur) => prev + cur, 0)}", None),
        ("function fn() {arguments.foo}", None),
        ("function fn() {arguments.map}", None),
        ("function fn() {arguments[method] }", None),
        ("function fn() {let method='map'; arguments[method](() => {}) }", None),
        ("function fn() {arguments['map']}", None),
        ("function fn() {arguments[`map`]}", None),
        ("function fn() {arg['map'](() => {})}", None),
        ("function fn() {foo.arguments.map}", None),
        ("function fn() {arguments[`map${''}`]((prev, cur) => prev + cur, 0)}", None),
        ("function fn() {arguments[`${''}map`]((prev, cur) => prev + cur, 0)}", None),
        ("function fn() {arguments[`${'map'}`]((prev, cur) => prev + cur, 0)}", None),
        ("function fn() {arguments.toLocaleString(() => {})}", None),
        ("function fn() {arguments.toString(() => {})}", None),
        ("function fn() {arguments.findLast(() => {})}", None),
        ("function fn() {arguments.group(() => {})}", None),
        ("function fn() {arguments.groupToMap(() => {})}", None),
        ("function fn() {arguments.toReversed(() => {})}", None),
        ("function fn() {arguments.toSorted(() => {})}", None),
        ("function fn() {arguments.toSpliced(0)}", None),
        ("function fn() {arguments.with(1, 1)}", None),
    ];

    let fail = vec![
        ("function fn() {arguments['map'](() => {})}", None),
        ("function fn() {arguments[`map`](() => {})}", None),
        ("function fn() {arguments.at(0)}", None),
        ("function fn() {arguments.concat([])}", None),
        ("function fn() {arguments.copyWithin(0)}", None),
        ("function fn() {arguments.entries()}", None),
        ("function fn() {arguments.every(() => {})}", None),
        ("function fn() {arguments.fill(() => {})}", None),
        ("function fn() {arguments.filter(() => {})}", None),
        ("function fn() {arguments.find(() => {})}", None),
        ("function fn() {arguments.findIndex(() => {})}", None),
        ("function fn() {arguments.flat(() => {})}", None),
        ("function fn() {arguments.flatMap(() => {})}", None),
        ("function fn() {arguments.forEach(() => {})}", None),
        ("function fn() {arguments.includes(() => {})}", None),
        ("function fn() {arguments.indexOf(() => {})}", None),
        ("function fn() {arguments.join()}", None),
        ("function fn() {arguments.keys()}", None),
        ("function fn() {arguments.lastIndexOf('')}", None),
        ("function fn() {arguments.map(() => {})}", None),
        ("function fn() {arguments.pop()}", None),
        ("function fn() {arguments.push('')}", None),
        ("function fn() {arguments.reduce(() => {})}", None),
        ("function fn() {arguments.reduceRight(() => {})}", None),
        ("function fn() {arguments.reverse()}", None),
        ("function fn() {arguments.shift()}", None),
        ("function fn() {arguments.slice()}", None),
        ("function fn() {arguments.some(() => {})}", None),
        ("function fn() {arguments.sort(() => {})}", None),
        ("function fn() {arguments.splice(() => {})}", None),
        ("function fn() {arguments.unshift()}", None),
        ("function fn() {arguments.values()}", None),
        ("function fn() {arguments['@@iterator'](() => {})}", None),
    ];

    Tester::new(BadArrayMethodOnArguments::NAME, BadArrayMethodOnArguments::PLUGIN, pass, fail)
        .test_and_snapshot();
}

#[test]
fn test_array_is_sorted() {
    let mut sorted_array = ARRAY_METHODS.to_vec();
    sorted_array.sort_unstable();

    assert_eq!(sorted_array, ARRAY_METHODS);
}
