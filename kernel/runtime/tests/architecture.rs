use proc_macro2::{Delimiter, TokenStream, TokenTree};
use serde_json::Value;
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, Output, Stdio};
use syn::{Item, ItemMod, Meta, Visibility};

fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .to_path_buf()
}

fn matches_attribute(meta: &Meta, name: &str) -> bool {
    if meta.path().is_ident(name) {
        return true;
    }
    let Meta::List(list) = meta else { return false };
    if !list.path.is_ident("cfg_attr") {
        return false;
    }
    let tokens: Vec<_> = list.tokens.clone().into_iter().collect();
    tokens
        .split(
            |token| matches!(token, TokenTree::Punct(punctuation) if punctuation.as_char() == ','),
        )
        .skip(1)
        .filter_map(|attribute| syn::parse2::<Meta>(attribute.iter().cloned().collect()).ok())
        .any(|attribute| matches_attribute(&attribute, name))
}

fn private_module(module: &ItemMod) -> bool {
    match &module.vis {
        Visibility::Inherited => true,
        Visibility::Restricted(visibility) => visibility.path.is_ident("self"),
        Visibility::Public(_) => false,
    }
}

fn module_has_attribute(module: &ItemMod, name: &str) -> bool {
    module.attrs.iter().any(|attribute| matches_attribute(&attribute.meta, name))
}

fn private_tool_boundaries(file: &syn::File) -> bool {
    file.items.iter().all(|item| {
        let Item::Mod(tool) = item else { return false };
        let Some((_, items)) = &tool.content else { return false };
        private_module(tool)
            && !module_has_attribute(tool, "macro_use")
            && !module_has_attribute(tool, "path")
            && items.iter().all(|item| {
                matches!(item, Item::Mod(module) if private_module(module) && !module_has_attribute(module, "path"))
            })
    })
}

// Check the crate-root macro export, including occurrences in unexpanded macro bodies.
// Ordinary implementation syntax is checked by rustc rather than parsed as a module tree here.
fn exports_macro(tokens: TokenStream) -> bool {
    let mut attribute = false;
    for token in tokens {
        if let TokenTree::Group(group) = &token {
            if attribute
                && group.delimiter() == Delimiter::Bracket
                && syn::parse2::<Meta>(group.stream())
                    .is_ok_and(|meta| matches_attribute(&meta, "macro_export"))
            {
                return true;
            }
            if exports_macro(group.stream()) {
                return true;
            }
        }
        attribute = matches!(token, TokenTree::Punct(punctuation) if punctuation.as_char() == '#');
    }
    false
}

fn read(path: &Path) -> String {
    fs::read_to_string(path).unwrap_or_else(|error| panic!("{}: {error}", path.display()))
}

fn parse(path: &Path) -> syn::File {
    syn::parse_file(&read(path)).unwrap_or_else(|error| panic!("{}: {error}", path.display()))
}

fn macro_export_in(directory: &Path) -> Option<PathBuf> {
    for entry in fs::read_dir(directory).unwrap() {
        let entry = entry.unwrap();
        let path = entry.path();
        if entry.file_type().unwrap().is_dir() {
            if let Some(export) = macro_export_in(&path) {
                return Some(export);
            }
        } else if path.extension().is_some_and(|extension| extension == "rs") {
            let tokens = read(&path)
                .parse::<TokenStream>()
                .unwrap_or_else(|error| panic!("{}: {error}", path.display()));
            if exports_macro(tokens) {
                return Some(path);
            }
        }
    }
    None
}

#[test]
fn tool_implementations_stay_inside_private_modules() {
    let directory = root().join("kernel/runtime/tests/support/registration/tools");
    assert!(private_tool_boundaries(&parse(&directory.join("mod.rs"))));
    assert_eq!(macro_export_in(&directory), None);
}

macro_rules! source_cases {
    ($($test:ident: $check:ident($source:expr) => $expected:expr;)*) => {
        $(#[test]
        fn $test() {
            assert_eq!($check($source), $expected);
        })*
    };
}

fn checks_tool_declarations(source: &str) -> bool {
    private_tool_boundaries(&syn::parse_file(source).unwrap())
}

fn source_exports_macro(source: &str) -> bool {
    exports_macro(source.parse().unwrap())
}

source_cases! {
    accepts_independent_private_tool_boundaries: checks_tool_declarations("mod first { mod implementation; } mod second { mod operation { pub(crate) fn evaluate() {} } mod factory; }") => true;
    rejects_a_public_implementation_module: checks_tool_declarations("mod first { pub(super) mod implementation; }") => false;
    rejects_reexporting_the_implementation_from_the_tool_boundary: checks_tool_declarations("mod first { mod implementation; pub(super) use implementation::Item; }") => false;
    rejects_importing_tool_macros_into_the_container: checks_tool_declarations("#[cfg_attr(all(), macro_use)] mod first { mod implementation; }") => false;
    accepts_importing_macros_within_one_tool: checks_tool_declarations("mod first { #[macro_use] mod implementation; mod operations; }") => true;
    accepts_private_macros_and_source_inclusion: source_exports_macro("macro_rules! generate { () => { fn local() {} } } generate!(); include!(\"generated.rs\");") => false;
    rejects_an_export_emitted_by_a_macro: source_exports_macro("macro_rules! generate { () => { #[macro_export] macro_rules! escape { () => {} } } }") => true;
    accepts_a_string_describing_macro_export: source_exports_macro("const EXAMPLE: &str = \"#[macro_export]\";") => false;
}

#[test]
fn rejects_macro_exports_in_nested_tool_sources() {
    let fixture = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/support/architecture/tool");
    assert_eq!(macro_export_in(&fixture), Some(fixture.join("nested/exports.rs")));
}

fn compile_generated_references(foundation_reference: &str, tool_reference: &str) -> Output {
    let source = format!(
        r#"
        mod foundation {{
            pub(crate) const SEED: u64 = 21;
            macro_rules! generate {{
                () => {{ fn value() -> u64 {{ {foundation_reference} }} }};
            }}
            generate!();
        }}
        mod tools {{
            mod first {{
                mod implementation {{
                    macro_rules! generate {{
                        () => {{ pub(crate) fn value() -> u64 {{ crate::foundation::SEED * 2 }} }};
                    }}
                    generate!();
                }}
            }}
            mod second {{
                mod implementation {{
                    macro_rules! generate {{
                        () => {{ fn value() -> u64 {{ {tool_reference} }} }};
                    }}
                    generate!();
                }}
            }}
        }}
        "#
    );
    let mut compiler = Command::new(std::env::var_os("RUSTC").unwrap_or_else(|| "rustc".into()))
        .args([
            "--crate-name",
            "tool_privacy",
            "--crate-type=lib",
            "--edition=2024",
            "--emit=metadata=-",
            "--error-format=json",
            "-",
        ])
        .stdin(Stdio::piped())
        .stdout(Stdio::null())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    compiler.stdin.take().unwrap().write_all(source.as_bytes()).unwrap();
    compiler.wait_with_output().unwrap()
}

#[test]
fn generated_tool_code_can_reference_foundation() {
    let result = compile_generated_references("SEED", "crate::foundation::SEED * 3");
    assert!(result.status.success(), "{}", String::from_utf8_lossy(&result.stderr));
}

#[test]
fn generated_tool_code_cannot_reference_a_sibling_implementation() {
    assert_private(compile_generated_references(
        "SEED",
        "crate::tools::first::implementation::value()",
    ));
}

#[test]
fn generated_foundation_code_cannot_reference_a_tool_implementation() {
    assert_private(compile_generated_references(
        "crate::tools::first::implementation::value()",
        "crate::foundation::SEED",
    ));
}

fn assert_private(result: Output) {
    let diagnostics = String::from_utf8(result.stderr).unwrap();
    assert!(!result.status.success(), "{diagnostics}");
    assert!(
        diagnostics
            .lines()
            .filter_map(|line| serde_json::from_str::<Value>(line).ok())
            .any(|diagnostic| diagnostic["code"]["code"] == "E0603"),
        "{diagnostics}"
    );
}

#[test]
fn kernel_crates_only_have_kernel_project_dependencies() {
    let result = Command::new("cargo")
        .args(["metadata", "--locked", "--no-deps", "--format-version", "1"])
        .current_dir(root())
        .output()
        .unwrap();
    assert!(result.status.success(), "{}", String::from_utf8_lossy(&result.stderr));
    let metadata: Value = serde_json::from_slice(&result.stdout).unwrap();
    let kernel = root().join("kernel").canonicalize().unwrap();
    let mut checked_kernel = Vec::new();
    for package in metadata["packages"].as_array().unwrap() {
        let name = package["name"].as_str().unwrap();
        let manifest = Path::new(package["manifest_path"].as_str().unwrap());
        if !manifest.canonicalize().unwrap().starts_with(&kernel) {
            continue;
        }
        checked_kernel.push(name);
        for dependency in package["dependencies"].as_array().unwrap() {
            if let Some(path) = dependency["path"].as_str() {
                assert!(
                    Path::new(path).canonicalize().unwrap().starts_with(&kernel),
                    "{name}: project dependency {path} needs module-level verification before admission"
                );
            }
            let target = dependency["name"].as_str().unwrap();
            assert!(
                !matches!(target, "kodogu" | "kodogu-cli" | "anflug"),
                "{name} depends on {target}"
            );
        }
    }
    assert!(checked_kernel.contains(&"kodogu-runtime"));
}
