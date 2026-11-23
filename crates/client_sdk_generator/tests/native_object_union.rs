use std::{
    fs,
    path::PathBuf,
    sync::atomic::{AtomicUsize, Ordering},
};

use client_sdk_schema::{ParameterType, RESOURCE_INDEX, Resource, Schema};

const FIXTURE: &str = include_str!("fixtures/native-object-union.yml");

fn fixture() -> Schema {
    serde_yaml_ng::from_str(FIXTURE).unwrap()
}

struct TempDir(PathBuf);

impl TempDir {
    fn new() -> Self {
        static COUNTER: AtomicUsize = AtomicUsize::new(0);
        let path = std::env::temp_dir().join(format!(
            "native-object-union-{}-{}",
            std::process::id(),
            COUNTER.fetch_add(1, Ordering::Relaxed),
        ));
        fs::create_dir(&path).unwrap();
        Self(path)
    }

    fn read(&self, path: &str) -> String {
        fs::read_to_string(self.0.join(path)).unwrap()
    }
}

impl Drop for TempDir {
    fn drop(&mut self) {
        fs::remove_dir_all(&self.0).unwrap();
    }
}

#[test]
fn native_generators_use_a_typed_optional_union_with_no_wire_wrapper() {
    let mut schema = fixture();
    let lifted = schema.lift_native_request_object_unions();
    assert_eq!(lifted.len(), 1);
    assert!(lifted.contains("request/RequestBypass"));

    let output = TempDir::new();
    let index = schema.build_resource_index();
    RESOURCE_INDEX.set(&index, || {
        client_sdk_dart_codegen::generate_resources_module(
            &schema.resources,
            output.0.join("dart"),
            "package:fixture",
        );
        client_sdk_kotlin_codegen::generate_resources_module(
            &schema.resources,
            output.0.join("kotlin"),
            "fixture",
        );
        client_sdk_swift_codegen::generate_resources_module(
            &schema.resources,
            output.0.join("swift"),
            "Fixture",
            &lifted,
        );
    });

    let dart_request = output.read("dart/request/synthetic_request.dart");
    assert!(dart_request.contains("RequestBypass? bypass"));
    assert!(dart_request.contains("import 'package:fixture/request/request_bypass.dart';"));
    assert!(dart_request.contains("if (bypass != null) 'bypass': bypass!.toJson()"));
    let dart_union = output.read("dart/request/request_bypass.dart");
    assert!(dart_union.contains("class RequestBypassChoiceA extends RequestBypass"));
    assert!(dart_union.contains("class RequestBypassChoiceB extends RequestBypass"));
    assert_eq!(dart_union.matches("=> value.toJson();").count(), 2);

    let kotlin_request = output.read("kotlin/request/syntheticRequest.kt");
    assert!(kotlin_request.contains("val bypass: RequestBypass? = null"));
    assert!(kotlin_request.contains("bypass?.let { put(\"bypass\", bypass.toJson()) }"));
    let kotlin_union = output.read("kotlin/request/requestBypass.kt");
    assert!(kotlin_union.contains("data class ChoiceA(val value: _ChoiceA)"));
    assert!(kotlin_union.contains("data class ChoiceB(val value: _ChoiceB)"));
    assert_eq!(kotlin_union.matches("-> value.toJson()").count(), 2);

    let swift_request = output.read("swift/request/SyntheticRequest.swift");
    assert!(swift_request.contains("bypass: RequestBypass?"));
    let swift_union = output.read("swift/request/RequestBypass.swift");
    assert!(swift_union.contains("case choiceA(ChoiceA)"));
    assert!(swift_union.contains("case choiceB(ChoiceB)"));
    assert_eq!(
        swift_union.matches("try value.encode(to: encoder)").count(),
        2
    );
    assert_eq!(
        swift_union
            .matches("(try? JSONShape.of(value)) == shape")
            .count(),
        2
    );
    assert!(
        !output
            .read("swift/entity/ExistingUnion.swift")
            .contains("JSONShape")
    );
}

#[test]
fn unions_with_hide_if_empty_are_not_lifted() {
    let input = FIXTURE.replace(
        "name: RequestBypass",
        "name: RequestBypass\n          hideIfEmpty: true",
    );
    let mut schema: Schema = serde_yaml_ng::from_str(&input).unwrap();
    assert!(schema.lift_native_request_object_unions().is_empty());
    let index = schema.build_resource_index();
    let ParameterType::Object { properties, .. } = &index["request/SyntheticRequest"].r#type else {
        panic!("expected request object");
    };
    assert!(matches!(
        properties["bypass"].r#type,
        ParameterType::Union { .. }
    ));
}

#[test]
#[should_panic(expected = "native inline union resource conflicts with request/RequestBypass")]
fn lifting_cannot_overwrite_an_existing_resource() {
    let mut schema = fixture();
    let Resource::SubResources(sections) = &mut schema.resources else {
        panic!("expected sections");
    };
    let Resource::SubResources(requests) = &mut sections["request"] else {
        panic!("expected requests");
    };
    requests.insert("RequestBypass".into(), requests["SyntheticRequest"].clone());
    schema.lift_native_request_object_unions();
}
