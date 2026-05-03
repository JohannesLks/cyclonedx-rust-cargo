mod v1_5 {
    use cyclonedx_bom::errors::XmlReadError;
    use cyclonedx_bom::models::bom::{Bom, SpecVersion};
    use cyclonedx_bom::validation::Validate;
    use test_utils::validate_json_with_schema;

    const XML_RECURSION_LIMIT: usize = 32;

    fn nested_components_bom(depth: usize) -> String {
        let mut xml = String::from(
            r#"<?xml version="1.0" encoding="UTF-8"?>
<bom xmlns="http://cyclonedx.org/schema/bom/1.5" version="1">
  <components>
"#,
        );

        for index in 0..depth {
            xml.push_str(&format!(
                r#"    <component type="library"><name>component-{index}</name>
"#
            ));
            if index + 1 < depth {
                xml.push_str("      <components>\n");
            }
        }

        for index in 0..depth {
            if index > 0 {
                xml.push_str("      </components>\n");
            }
            xml.push_str("    </component>\n");
        }

        xml.push_str("  </components>\n</bom>\n");
        xml
    }

    fn pedigree_nested_components_bom(depth: usize) -> String {
        let mut xml = String::from(
            r#"<?xml version="1.0" encoding="UTF-8"?>
<bom xmlns="http://cyclonedx.org/schema/bom/1.5" version="1">
  <components>
"#,
        );

        for index in 0..depth {
            xml.push_str(&format!(
                r#"    <component type="library"><name>component-{index}</name>
"#
            ));
            if index + 1 < depth {
                xml.push_str("      <pedigree><ancestors>\n");
            }
        }

        for index in (0..depth).rev() {
            xml.push_str("    </component>\n");
            if index > 0 {
                xml.push_str("      </ancestors></pedigree>\n");
            }
        }

        xml.push_str("  </components>\n</bom>\n");
        xml
    }

    fn lax_nested_elements_bom(depth: usize) -> String {
        let mut xml = String::from(
            r#"<?xml version="1.0" encoding="UTF-8"?>
<bom xmlns="http://cyclonedx.org/schema/bom/1.5" version="1">
  <components>
"#,
        );

        for index in 0..depth {
            xml.push_str(&format!("    <unknown{index}>\n"));
        }

        for index in (0..depth).rev() {
            xml.push_str(&format!("    </unknown{index}>\n"));
        }

        xml.push_str("  </components>\n</bom>\n");
        xml
    }

    fn nested_services_bom(depth: usize) -> String {
        let mut xml = String::from(
            r#"<?xml version="1.0" encoding="UTF-8"?>
<bom xmlns="http://cyclonedx.org/schema/bom/1.5" version="1">
  <services>
"#,
        );

        for index in 0..depth {
            xml.push_str(&format!("    <service><name>service-{index}</name>\n"));
            if index + 1 < depth {
                xml.push_str("      <services>\n");
            }
        }

        for index in 0..depth {
            if index > 0 {
                xml.push_str("      </services>\n");
            }
            xml.push_str("    </service>\n");
        }

        xml.push_str("  </services>\n</bom>\n");
        xml
    }

    fn utf16le_with_bom(xml: &str) -> Vec<u8> {
        let mut bytes = vec![0xff, 0xfe];

        for code_unit in xml.encode_utf16() {
            bytes.extend_from_slice(&code_unit.to_le_bytes());
        }

        bytes
    }

    #[test]
    fn it_should_parse_all_of_the_valid_xml_specifications() {
        insta::with_settings!({
            snapshot_path => "spec/snapshots/1.5",
            prepend_module_to_snapshot => false,
        }, {
            insta::glob!("spec/1.5/valid*.xml", |path| {
                let file = std::fs::File::open(path).unwrap_or_else(|_| panic!("Failed to read file: {path:?}"));
                let bom = Bom::parse_from_xml_v1_5(file).unwrap_or_else(|e| panic!("Failed to parse the document as an BOM: {path:?} {:#?}", e));

                let validation_result = bom.validate_version(SpecVersion::V1_5);
                if !validation_result.passed() {
                    dbg!(&validation_result);
                }
                assert!(
                    validation_result.passed(),
                    "{path:?} unexpectedly failed validation"
                );

                let mut output = Vec::new();
                bom.output_as_xml_v1_5(&mut output)
                    .unwrap_or_else(|_| panic!("Failed to output the file: {path:?}"));
                let bom_output = String::from_utf8_lossy(&output).to_string();

                insta::assert_snapshot!(bom_output);
            });
        });
    }

    #[test]
    fn it_should_parse_all_of_the_valid_json_specifications() {
        insta::with_settings!({
            snapshot_path => "spec/snapshots/1.5",
            prepend_module_to_snapshot => false,
        }, {
            insta::glob!("spec/1.5/valid*.json", |path| {
                let file = std::fs::File::open(path).unwrap_or_else(|_| panic!("Failed to read file: {path:?}"));
                let bom = Bom::parse_from_json_v1_5(file).unwrap_or_else(|e| panic!("Failed to parse the document as an BOM: {path:?} {:#?}", e));

                let validation_result = bom.validate_version(SpecVersion::V1_5);
                assert!(
                    validation_result.passed(),
                    "{path:?} unexpectedly failed validation"
                );

                let mut output = Vec::new();
                bom.output_as_json_v1_5(&mut output)
                    .unwrap_or_else(|_| panic!("Failed to output the file: {path:?}"));
                let bom_output = String::from_utf8_lossy(&output).to_string();

                // Check that the written JSON file validates against its schema.
                let json = serde_json::from_str(&bom_output).expect("Failed to parse JSON");
                validate_json_with_schema(&json, SpecVersion::V1_5)
                    .unwrap_or_else(|errors| panic!("Failed to validate output {path:?}, errors: {errors:?}"));

                insta::assert_snapshot!(bom_output);
            });
        });
    }

    #[test]
    fn it_should_fail_to_parse_all_of_the_invalid_xml_specifications() {
        insta::with_settings!({
            snapshot_path => "spec/snapshots/1.5",
            prepend_module_to_snapshot => false,
        }, {
            insta::glob!("spec/1.5/invalid*.xml", |path| {
                let file = std::fs::File::open(path).unwrap_or_else(|_| panic!("Failed to read file: {path:?}"));
                if let Ok(bom) = Bom::parse_from_xml_v1_5(file) {
                    let validation_result = bom.validate_version(SpecVersion::V1_5);
                    assert!(
                        validation_result.has_errors(),
                        "{path:?} unexpectedly passed validation"
                    );
                }
            });
        });
    }

    #[test]
    fn it_should_fail_to_parse_all_of_the_invalid_json_specifications() {
        insta::with_settings!({
            snapshot_path => "spec/snapshots/1.5",
            prepend_module_to_snapshot => false,
        }, {
            insta::glob!("spec/1.5/invalid*.json", |path| {
                let file = std::fs::File::open(path).unwrap_or_else(|_| panic!("Failed to read file: {path:?}"));
                if let Ok(bom) = Bom::parse_from_json_v1_5(file) {
                    let validation_result = bom.validate_version(SpecVersion::V1_5);
                    assert!(
                        validation_result.has_errors(),
                        "{path:?} unexpectedly passed validation"
                    );
                }
            });
        });
    }

    #[test]
    fn it_should_parse_nested_components_within_the_xml_depth_limit() {
        let bom = Bom::parse_from_xml_v1_5(nested_components_bom(XML_RECURSION_LIMIT).as_bytes())
            .expect("Expected nested components within the depth limit to parse");

        assert_eq!(bom.components.expect("Expected components").0.len(), 1);
    }

    #[test]
    fn it_should_reject_nested_components_beyond_the_xml_depth_limit() {
        let error = Bom::parse_from_xml_v1_5(nested_components_bom(120).as_bytes())
            .expect_err("Expected nested components beyond the depth limit to fail");

        match error {
            XmlReadError::RecursionLimitExceeded { element, limit } => {
                assert_eq!(element, "component");
                assert_eq!(limit, XML_RECURSION_LIMIT);
            }
            other => panic!("Expected recursion limit error, got {other:?}"),
        }
    }

    #[test]
    fn it_should_reject_pedigree_nested_components_beyond_the_xml_depth_limit() {
        let error = Bom::parse_from_xml_v1_5(pedigree_nested_components_bom(120).as_bytes())
            .expect_err("Expected pedigree nested components beyond the depth limit to fail");

        match error {
            XmlReadError::RecursionLimitExceeded { element, limit } => {
                assert_eq!(element, "component");
                assert_eq!(limit, XML_RECURSION_LIMIT);
            }
            other => panic!("Expected recursion limit error, got {other:?}"),
        }
    }

    #[test]
    fn it_should_reject_nested_services_beyond_the_xml_depth_limit() {
        let error = Bom::parse_from_xml_v1_5(nested_services_bom(120).as_bytes())
            .expect_err("Expected nested services beyond the depth limit to fail");

        match error {
            XmlReadError::RecursionLimitExceeded { element, limit } => {
                assert_eq!(element, "service");
                assert_eq!(limit, XML_RECURSION_LIMIT);
            }
            other => panic!("Expected recursion limit error, got {other:?}"),
        }
    }

    #[test]
    fn it_should_reset_xml_depth_after_rejection() {
        let _ = Bom::parse_from_xml_v1_5(nested_components_bom(120).as_bytes())
            .expect_err("Expected nested components beyond the depth limit to fail");

        Bom::parse_from_xml_v1_5(nested_components_bom(1).as_bytes())
            .expect("Expected later parse to start with a fresh depth counter");
    }

    #[test]
    fn it_should_reject_lax_validation_elements_beyond_the_xml_depth_limit() {
        let error = Bom::parse_from_xml_v1_5(lax_nested_elements_bom(120).as_bytes())
            .expect_err("Expected lax nested elements beyond the depth limit to fail");

        match error {
            XmlReadError::RecursionLimitExceeded { limit, .. } => {
                assert_eq!(limit, XML_RECURSION_LIMIT);
            }
            other => panic!("Expected recursion limit error, got {other:?}"),
        }
    }

    #[test]
    fn it_should_reject_utf16_xml_beyond_the_xml_depth_limit() {
        let xml = utf16le_with_bom(
            &nested_components_bom(120).replace("encoding=\"UTF-8\"", "encoding=\"UTF-16\""),
        );
        let error = Bom::parse_from_xml_v1_5(xml.as_slice())
            .expect_err("Expected UTF-16 XML beyond the depth limit to fail");

        match error {
            XmlReadError::RecursionLimitExceeded { limit, .. } => {
                assert_eq!(limit, XML_RECURSION_LIMIT);
            }
            other => panic!("Expected recursion limit error, got {other:?}"),
        }
    }
}
