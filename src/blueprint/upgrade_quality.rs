use serde_json::json;

const QUALITIES: [&str; 6] = [
    "normal",
    "uncommon",
    "rare",
    "epic",
    "legendary",
    "legendary",
];

fn upgrade_quality(thing: &mut serde_json::Value) {
    if let Some(quality) = thing.get_mut("quality") {
        let s = quality.as_str().unwrap();
        let new_quality = QUALITIES
            .windows(2)
            .find(|pair| pair[0] == s)
            .unwrap_or_else(|| panic!("can't find quality {s}"))[1];
        *quality = json!(new_quality);
    } else {
        // TODO handle normal
    }
}

fn upgrade_circuit_condition(circuit_condition: &mut serde_json::Value) {
    if let Some(first_signal) = circuit_condition.get_mut("first_signal") {
        upgrade_quality(first_signal);
    }
    // TODO what about second signal?
}

pub(crate) fn upgrade(mut json: serde_json::Value) -> serde_json::Value {
    let bp = json
        .get_mut("blueprint")
        .expect("blueprint books not supported yet");
    let entities = bp.get_mut("entities").unwrap().as_array_mut().unwrap();
    for entity in entities {
        if let Some(control_behavior) = entity.get_mut("control_behavior") {
            if let Some(circuit_condition) = control_behavior.get_mut("circuit_condition") {
                upgrade_circuit_condition(circuit_condition);
            }
            if let Some(logistic_condition) = control_behavior.get_mut("logistic_condition") {
                upgrade_circuit_condition(logistic_condition);
            }
        }
        if let Some(filter) = entity.get_mut("filter") {
            upgrade_quality(filter);
        }
        if let Some(filters) = entity.get_mut("filters") {
            for filter in filters.as_array_mut().unwrap() {
                upgrade_quality(filter);
            }
        }
        if let Some(quality) = entity.get_mut("recipe_quality") {
            let s = quality.as_str().unwrap();
            let new_quality = QUALITIES.windows(2).find(|pair| pair[0] == s).unwrap()[1];
            *quality = json!(new_quality);
        } else {
            // TODO figure out what things are supposed to have a recipe quality when the recipe is normal
        }
        if let Some(request_filters) = entity.get_mut("request_filters") {
            if let Some(sections) = request_filters.get_mut("sections") {
                for section in sections.as_array_mut().unwrap() {
                    if let Some(filters) = section.get_mut("filters") {
                        for filter in filters.as_array_mut().unwrap() {
                            upgrade_quality(filter);
                        }
                    }
                }
            }
        }
    }
    json
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::str::FromStr;

    macro_rules! test_bp {
        {bp: $bp: literal, $json: expr} => {{
            let json = crate::blueprint::blueprint_to_json($bp);
            let json = serde_json::Value::from_str(&json).unwrap();
            let expected: serde_json::Value = $json;
            if json != expected {
                eprintln!("blueprint json:");
                eprintln!("json!({json:#})");
                panic!("json mismatch");
            }
            json
        }}
    }

    fn jaq_one(program: &str, input: serde_json::Value) -> serde_json::Value {
        // copied nearly straight from the jaq-core example
        use jaq_core::{Ctx, RcIter, load};
        use jaq_json::Val;

        let program = File {
            code: program,
            path: (),
        };

        use load::{Arena, File, Loader};

        let loader = Loader::new(jaq_std::defs().chain(jaq_json::defs()));
        let arena = Arena::default();

        // parse the filter
        let modules = loader
            .load(&arena, program)
            .unwrap_or_else(|errs| panic!("jaq parse error: {errs:?}"));

        // compile the filter
        let filter = jaq_core::Compiler::default()
            .with_funs(jaq_std::funs().chain(jaq_json::funs()))
            .compile(modules)
            .unwrap();

        let inputs = RcIter::new(core::iter::empty());

        // iterator over the output values
        let mut out = filter.run((Ctx::new([], &inputs), Val::from(input)));

        let result = out
            .next()
            .expect("filter should produce one value")
            .unwrap_or_else(|err| panic!("jaq error: {err}"));
        if let Some(unexpected) = out.next() {
            panic!("more than one result from filter: {unexpected:?}");
        }
        result.into()
    }

    #[test]
    fn test_inserter_filters() {
        let json = test_bp! {
            bp: "0eNqFUW1ugzAMvQry71AVWloVaQfYGaYJBXBbaxCYcbqhirvPKS2Ttkn7mRe/Dz9foWw89kxOIL8CVZ0bIH+5wkAnZ5uAOdsi5FD65i0mNyALMkwGyNX4CXkyvRpAJySEM/X2GAvn21In88T8LWGg7wZldS64qNJmvVtlBkbI4zRZrzL1qImxmkeS1ICmE+6aosSzvVDHgVgRV56kQGfLBmvIhT2aBVZKvZgciQcpvleTsQ+5LsTiFVmCzhMxXpBHOZM7hX1DNWJDT+vwaHvLVkIGeIJJ/4/U6FpzBY9uFsWq63vkuGNUl3c104YU9k6FWg33W9EsKul/KmxvwE8FvYsfsFhyhWICSIJtuMZyeAONLVEbgef7caIPknOURneP6KFhQCsZbm1mu/SwPRyy/SbbbPfpNH0BVCbHeg==",
            json!({
              "blueprint": {
                "entities": [
                  {
                    "control_behavior": {
                      "circuit_condition": {
                        "comparator": "=",
                        "constant": 0,
                        "first_signal": {
                          "name": "signal-everything",
                          "type": "virtual"
                        }
                      },
                      "circuit_enabled": true
                    },
                    "direction": 12,
                    "entity_number": 1,
                    "filters": [
                      {
                        "comparator": "=",
                        "index": 1,
                        "name": "copper-ore",
                        "quality": "uncommon"
                      },
                      {
                        "comparator": "=",
                        "index": 2,
                        "name": "copper-ore",
                        "quality": "rare"
                      }
                    ],
                    "name": "bulk-inserter",
                    "position": {
                      "x": 306.5,
                      "y": -210.5
                    },
                    "use_filters": true
                  }
                ],
                "icons": [
                  {
                    "index": 1,
                    "signal": {
                      "name": "bulk-inserter"
                    }
                  }
                ],
                "item": "blueprint",
                "label": "Inserter with 2 quality filters",
                "version": 562949957353472i64
              }
            })
        };
        let upgraded = upgrade(json);
        let entity = jaq_one(".blueprint.entities[]", upgraded);
        assert_eq!(
            jaq_one("[.filters[].quality]", entity),
            json!(["rare", "epic"])
        );
    }

    #[test]
    fn test_inserter_circuit_conditions() {
        let bp = test_bp!(
            bp: "0eNp9kt1uwyAMhd/F16QaSdOu0d6kqiJCvM0agcxAu6ri3QeZVO0n2iVHPp/tY24wmIgzkw3Q3YC0sx664w08vVhlimbVhNDBEM1bRdYjB2RIAsiO+AGdTGKlesKR4lShQR2YdDU7g99MdToJQBsoEH71Wx7X3sZpyPhOin9JAmbns9nZ0jMDm4fdphVwha6qZbNpU5nqF7MW67uswB5/wgSMxLn/UiIzJ8cU2Jl+wFd1JsfFqIl1pNCjVYPBEbrAEcVdzpbx3uSZ2If+T2rGXaoRbR7mWvls1yFyWfY9KpO1XMGKlxw9Ft5fwmyUDzmlQfGqT7tpVqxCGRmeIKVyiEterlzhKIUUtZCnrFHAqSR1/x0Czsh+mb/d1Yft4dDum7bZ7uuUPgEVLcRl",
            json!({
              "blueprint": {
                "entities": [
                  {
                    "entity_number": 1,
                    "name": "medium-electric-pole",
                    "position": {
                      "x": 306.5,
                      "y": -213.5
                    }
                  },
                  {
                    "control_behavior": {
                      "circuit_condition": {
                        "comparator": "<",
                        "first_signal": {
                          "name": "low-density-structure",
                          "quality": "rare"
                        },
                        "second_signal": {
                          "name": "plastic-bar",
                          "quality": "rare"
                        }
                      },
                      "circuit_enabled": true
                    },
                    "direction": 12,
                    "entity_number": 2,
                    "name": "bulk-inserter",
                    "position": {
                      "x": 308.5,
                      "y": -213.5
                    }
                  }
                ],
                "icons": [
                  {
                    "index": 1,
                    "signal": {
                      "name": "bulk-inserter"
                    }
                  },
                  {
                    "index": 2,
                    "signal": {
                      "name": "medium-electric-pole"
                    }
                  }
                ],
                "item": "blueprint",
                "version": 562949957353472i64,
                "wires": [
                  [
                    1,
                    1,
                    2,
                    1
                  ]
                ]
              }
            })
        );
        let upgraded = upgrade(bp);
        let inserter = jaq_one(
            r#".blueprint.entities[] | select(.name == "bulk-inserter")"#,
            upgraded,
        );
        assert_eq!(
            jaq_one(".control_behavior.circuit_condition", inserter),
            json!({
              "comparator": "<",
              "first_signal": {
                "name": "low-density-structure",
                "quality": "epic"
              },
              "second_signal": {
                "name": "plastic-bar",
                // TODO upgrade second signal
                "quality": "rare"
              }
            })
        );
    }

    #[test]
    fn test_inserter_logistic_conditions() {
        let bp = test_bp!(
            bp: "0eNptUc1ugzAMfhefQyWgrCOHvcg0oQBeZzUkzDHtqop3n9NKXNpjPn2/zg16v+DMFATsDWiIIYH9vEGiY3A+Y8FNCBb6xZ8KCglZkGE1QGHEP7Dl+mUAg5AQPqT3x7ULy9Qr05bmtYWBOSZVxZBT1Kku611j4Aq2qMr3XaMZIzEOD0pZGdB2wtF3Pf64M0XOQsWCcjqJnY9HSkJDF1AukU9ghRc0sOHKHbfEb+Ik3dPO2bvMLXqXK/4uzusYxdkx5tkJs8uzzsdLMWLQRdciae4giwpeOQxxmh07yf3hA9Y1X5AEp3yi7TcMnJHTvWvzVrX7tm0OdVPvD9W6/gOS8Jcm",
            json!({
              "blueprint": {
                "entities": [
                  {
                    "control_behavior": {
                      "connect_to_logistic_network": true,
                      "logistic_condition": {
                        "comparator": ">",
                        "first_signal": {
                          "name": "plastic-bar",
                          "quality": "rare"
                        },
                        "second_signal": {
                          "name": "low-density-structure",
                          "quality": "rare"
                        }
                      }
                    },
                    "direction": 12,
                    "entity_number": 1,
                    "name": "bulk-inserter",
                    "position": {
                      "x": 313.5,
                      "y": -218.5
                    }
                  }
                ],
                "icons": [
                  {
                    "index": 1,
                    "signal": {
                      "name": "bulk-inserter"
                    }
                  }
                ],
                "item": "blueprint",
                "version": 562949957353472i64
              }
            })
        );
        let upgraded = upgrade(bp);
        let inserter = jaq_one(r#".blueprint.entities[]"#, upgraded);
        assert_eq!(
            jaq_one(".control_behavior.logistic_condition", inserter),
            json!({
              "comparator": ">",
              "first_signal": {
                "name": "plastic-bar",
                "quality": "epic"
              },
              "second_signal": {
                "name": "low-density-structure",
                // TODO upgrade second signal
                "quality": "rare"
              }
            })
        );
    }

    #[test]
    fn test_splitter_filter() {
        let bp = test_bp!(
            bp: "0eNplj9FtwzAMRHfht1LAdpTAWqUoAsVlCgKyqEpU0MDwANkis3WSUs5HPvrJw73j3QLnUDFligJuAZo4FnDvCxT6ij40LfoZwUFJgUQww2qA4if+gOvWDwMYhYTwSW3H7RTrfFan68w/2kDiogDHlq0hQ2cN3MDt+u7wZjX8QqEZX58nTgnzjjMq/V29Jqkfst+EiefksxdWBH7vj9aPq6QqJ53F+WkOeBFodUlw1vu12sAVc9n62EM/7sfRHgc77I/9uv4BmRBgAA==",
            json!({
              "blueprint": {
                "entities": [
                  {
                    "entity_number": 1,
                    "filter": {
                      "comparator": "≠",
                      "name": "copper-ore",
                      "quality": "rare"
                    },
                    "name": "splitter",
                    "output_priority": "left",
                    "position": {
                      "x": 315,
                      "y": -216.5
                    }
                  }
                ],
                "icons": [
                  {
                    "index": 1,
                    "signal": {
                      "name": "splitter"
                    }
                  }
                ],
                "item": "blueprint",
                "version": 562949957353472i64
              }
            })
        );
        let upgraded = upgrade(bp);
        let splitter = jaq_one(r#".blueprint.entities[]"#, upgraded);
        assert_eq!(jaq_one(".filter.quality", splitter), json!("epic"));
    }

    #[test]
    fn test_splitter_filter_bare_quality() {
        let bp = test_bp!(
            bp: "0eNplj1FOAzEMRO8y3wFpd5tWm6sgVKXFRZaySUicimq1B+AWnI2T4LQffPBpz7zxeMUpNMqFo8Ct4HOKFe5lReX36EPfRb8QHGoOLEIFmwHHN/qEG7ZXA4rCwvSg7sPtGNtyUqcbzD/aIKeqQIo9W0OmwRrc4J7GYf9sNfzCoRtV/WheKdVQfCFFz2nJvnhJKuPn67t3SU1yk6O+kMrDHOgi6NVYaNH570ODK5V6v23347ybZ3uY7LQ7jNv2C3LlWXU=",
            json!({
              "blueprint": {
                "entities": [
                  {
                    "entity_number": 1,
                    "filter": {
                      "comparator": "≠",
                      "quality": "rare"
                    },
                    "name": "splitter",
                    "output_priority": "left",
                    "position": {
                      "x": 315,
                      "y": -216.5
                    }
                  }
                ],
                "icons": [
                  {
                    "index": 1,
                    "signal": {
                      "name": "splitter"
                    }
                  }
                ],
                "item": "blueprint",
                "version": 562949957353472i64
              }
            })
        );
        let upgraded = upgrade(bp);
        let splitter = jaq_one(r#".blueprint.entities[]"#, upgraded);
        assert_eq!(jaq_one(".filter.quality", splitter), json!("epic"));
    }

    #[test]
    fn test_assembler_recipe() {
        let bp = test_bp!(
            bp: "0eNqFkNGqwjAMht8l163oZtXtVURGN8MMtOlsu3OOjL77iQp6I3iZP8mXjyzQuxmnSJyhXYCGwAna4wKJRrbunrH1CC3YlND3jnjU3g4XYtQ1KLjO1lG+ycDMQ/A+MBQFxGf8g3ZTTgqQM2XCJ/ZR3DqefY9RBtQX/BSSLAtURARYbw4ro0DO6apar0z5KKAg4kDTHUsxsB7RRv17QXSvVvfJW2Qpo5fs/RQFPxjTw8DsqmbbNGZfm3q7r0r5B0Auaik=",
            json!({
              "blueprint": {
                "entities": [
                  {
                    "entity_number": 1,
                    "name": "assembling-machine-3",
                    "position": {
                      "x": 318.5,
                      "y": -220.5
                    },
                    "quality": "uncommon",
                    "recipe": "iron-gear-wheel",
                    "recipe_quality": "uncommon"
                  }
                ],
                "icons": [
                  {
                    "index": 1,
                    "signal": {
                      "name": "assembling-machine-3",
                      "quality": "uncommon"
                    }
                  }
                ],
                "item": "blueprint",
                "version": 562949957353472i64
              }
            })
        );
        let upgraded = upgrade(bp);
        let assembler = jaq_one(r#".blueprint.entities[]"#, upgraded);
        // assembler quality should stay the same
        assert_eq!(jaq_one(".quality", assembler.clone()), json!("uncommon"));
        // but recipe quality should increase
        assert_eq!(jaq_one(".recipe_quality", assembler), json!("rare"));
    }

    #[test]
    fn test_storage_chest_filter() {
        let bp = test_bp!(
            bp: "0eNptkN1uwjAMhd/F1ykaLQE10p4EIRSCYZEaJ83PNFT13XHKAE2acpXjY3/HnuA0FAzRUgY1gTWeEqj9BMleSQ9VI+0QFKTso75iY74wZZgFWDrjD6j1fBCAlG22+GhdPrcjFXfCyAbx/wgBwSfu8lQpPKn76FdSwA1U0663K8mMiGNh7/Fih4wxVWNCU3seqGcGAS/HH/UXbHwIGBsfkalj0QPnY7mQ8c4xX7DDBR01B2T9cxFKPQlvx6+uaDM6rr3PJeCbgUt8uW37Td/LXSe7za6d5zs2XnI1",
            json!({
              "blueprint": {
                "entities": [
                  {
                    "entity_number": 1,
                    "name": "storage-chest",
                    "position": {
                      "x": 309.5,
                      "y": -216.5
                    },
                    "request_filters": {
                      "sections": [
                        {
                          "filters": [
                            {
                              "comparator": "=",
                              "count": 1,
                              "index": 1,
                              "name": "copper-ore",
                              "quality": "uncommon"
                            }
                          ],
                          "index": 1
                        }
                      ]
                    }
                  }
                ],
                "icons": [
                  {
                    "index": 1,
                    "signal": {
                      "name": "storage-chest"
                    }
                  }
                ],
                "item": "blueprint",
                "version": 562949957353472i64
              }
            })
        );
        let upgraded = upgrade(bp);
        let chest = jaq_one(r#".blueprint.entities[]"#, upgraded);
        assert_eq!(
            jaq_one(".request_filters.sections[].filters[].quality", chest),
            json!("rare")
        )
    }

    #[test]
    fn test_requesting_chest_filters() {
        let bp = test_bp!(
            bp: "0eNrtlN1qwzAMhd9F125pkrqlgT1JCcVxlc4Q26ljj4WQd5+c/oyMbB29G+wy4uh8so6dHso6YOOU8ZD3oKQ1LeT7Hlp1MqKONSM0Qg5lqCp0C/mKrYeBgTJHfIc8GdiM2OE5kG5Gnw4FAzReeYUX0vjRHUzQJToyZN+ZMGhsS33WRBJ5ZUmy5Aw6yBdpsllyolx7DpWqqbGNwhZl7LnAblMzuCsm1Sta2qYhrnVI1HMQNU1I5WCk1Zr4jBS6EU54SyPDy1gIcYd8FRdyO+wjQyfGwg9mxdTumamxUfL3kOw5SI0nNEfhukekYoiwL6F/LmpyzWYSX/0n/gcTpzevPOqY7/13w+CNkGO2fJPu1rsd32Y8W2/TYfgAQzKDGQ==",
            json!({
              "blueprint": {
                "entities": [
                  {
                    "entity_number": 1,
                    "name": "requester-chest",
                    "position": {
                      "x": 311.5,
                      "y": -216.5
                    },
                    "request_filters": {
                      "sections": [
                        {
                          "filters": [
                            {
                              "comparator": "=",
                              "count": 50,
                              "index": 1,
                              "name": "copper-ore",
                              "quality": "uncommon"
                            },
                            {
                              "comparator": "=",
                              "count": 50,
                              "index": 2,
                              "name": "copper-ore",
                              "quality": "rare"
                            }
                          ],
                          "index": 1
                        },
                        {
                          "filters": [
                            {
                              "comparator": "=",
                              "count": 50,
                              "index": 1,
                              "name": "copper-ore",
                              "quality": "epic"
                            }
                          ],
                          "index": 2
                        },
                        {
                          "filters": [
                            {
                              "comparator": "=",
                              "count": 50,
                              "index": 1,
                              "name": "copper-ore",
                              "quality": "legendary"
                            }
                          ],
                          "index": 3
                        }
                      ]
                    }
                  },
                  {
                    "entity_number": 2,
                    "name": "buffer-chest",
                    "position": {
                      "x": 310.5,
                      "y": -216.5
                    },
                    "request_filters": {
                      "sections": [
                        {
                          "filters": [
                            {
                              "comparator": "=",
                              "count": 50,
                              "index": 1,
                              "name": "copper-ore",
                              "quality": "uncommon"
                            },
                            {
                              "comparator": "=",
                              "count": 50,
                              "index": 2,
                              "name": "copper-ore",
                              "quality": "rare"
                            }
                          ],
                          "index": 1
                        },
                        {
                          "filters": [
                            {
                              "comparator": "=",
                              "count": 50,
                              "index": 1,
                              "name": "copper-ore",
                              "quality": "epic"
                            }
                          ],
                          "index": 2
                        },
                        {
                          "filters": [
                            {
                              "comparator": "=",
                              "count": 50,
                              "index": 1,
                              "name": "copper-ore",
                              "quality": "legendary"
                            }
                          ],
                          "index": 3
                        }
                      ]
                    }
                  }
                ],
                "icons": [
                  {
                    "index": 1,
                    "signal": {
                      "name": "buffer-chest"
                    }
                  },
                  {
                    "index": 2,
                    "signal": {
                      "name": "requester-chest"
                    }
                  }
                ],
                "item": "blueprint",
                "version": 562949957353472i64
              }
            })
        );
        let upgraded = upgrade(bp);
        let serde_json::Value::Array(chests) = jaq_one(r#"[.blueprint.entities[]]"#, upgraded)
        else {
            panic!("expected array")
        };
        assert_eq!(chests.len(), 2);
        for chest in chests {
            assert_eq!(
                jaq_one(
                    "[.request_filters.sections[] | [.filters[].quality]]",
                    chest
                ),
                json!([["rare", "epic"], ["legendary"], ["legendary"]])
            )
        }
    }

    // TODO: implement constant combinators
    #[test]
    #[should_panic = "wrong qualities"]
    fn test_constant_combinator_signals() {
        let bp = test_bp!(
            bp: "0eNqdktFqwzAMRf9Fz85YkmYlhn3JCMVJtc5gy67jlIXgf5+csY4Wuo6+yVfSvQfbC/RmQh80RZAL6MHRCPJtgVEfSJmskbIIEnInKorF4GyvSUUXIAnQtMdPkGXqBCBFHTV+G6yHeUeT7THwgPjLSIB3I+86yonsVz+3T42AGWRRlVxyEu/F4Myuxw910rzEkyMOeWm8rDn9B0vAuzYRw7V6ZvEeQ+ECMsJxUoaRWZ6I2SzD5FTrVVghJbyuwpTvqkzi7Ffd8wtqFW57dZdujzCj18O/M+rHMgwekPYqzHeCupTyf9ARLTd/f5iAE0eur9y8VO2mbZtt3dSbbZXSF0iP2Dc=",
            json!({
              "blueprint": {
                "entities": [
                  {
                    "control_behavior": {
                      "sections": {
                        "sections": [
                          {
                            "filters": [
                              {
                                "comparator": "=",
                                "count": 1,
                                "index": 1,
                                "name": "copper-ore",
                                "quality": "uncommon"
                              },
                              {
                                "comparator": "=",
                                "count": 1,
                                "index": 2,
                                "name": "copper-ore",
                                "quality": "rare"
                              }
                            ],
                            "index": 1
                          },
                          {
                            "filters": [
                              {
                                "comparator": "=",
                                "count": 1,
                                "index": 1,
                                "name": "copper-ore",
                                "quality": "epic"
                              }
                            ],
                            "index": 2
                          },
                          {
                            "filters": [
                              {
                                "comparator": "=",
                                "count": 1,
                                "index": 1,
                                "name": "copper-ore",
                                "quality": "legendary"
                              }
                            ],
                            "index": 3
                          }
                        ]
                      }
                    },
                    "entity_number": 1,
                    "name": "constant-combinator",
                    "position": {
                      "x": 309.5,
                      "y": -219.5
                    }
                  }
                ],
                "icons": [
                  {
                    "index": 1,
                    "signal": {
                      "name": "constant-combinator"
                    }
                  }
                ],
                "item": "blueprint",
                "version": 562949957353472i64
              }
            })
        );
        let upgraded = upgrade(bp);
        let constant_combinator = jaq_one(r#".blueprint.entities[]"#, upgraded);
        eprintln!("{constant_combinator:#}");
        assert_eq!(
            jaq_one(
                "[.control_behavior.sections.sections[] | [.filters[].quality]]",
                constant_combinator
            ),
            json!([["rare", "epic"], ["legendary"], ["legendary"]]),
            "wrong qualities"
        )
    }

    // TODO: implement arithmetic combinators
    #[test]
    #[should_panic = "wrong signals"]
    fn test_arithmetic_combinator() {
        let bp = test_bp!(
            bp: "0eNqNkc1ugzAQhN9lrzVVwk8jeJWqQsZsm5XwT9d21Aj53bvAIYdKVXxjduebMV5hWjIGJpdgWIGMdxGG9xUifTm9bJrTFmEAzZSuFhOZyng7kdPJMxQF5Gb8geFcPhSgS5QID8T+cR9dthOyLKj/UQqCj+L2bksVYnM+KbjDUNX16bWTpJkYzbHQKpCqif0yTnjVNxKAuB7kUcbzTovb4JM4pvHPpYwPAbnyjBL/nfUihUXOTnpZyZHQiBvpaStrEcTmZaiPrvAiGz6nkJ9vgIEMFDnyUymhFenxUApuyHFnd2913/Z9d2m6pr3UpfwCsbKgAQ==",
            json!({
              "blueprint": {
                "entities": [
                  {
                    "control_behavior": {
                      "arithmetic_conditions": {
                        "first_signal": {
                          "name": "copper-ore",
                          "quality": "uncommon"
                        },
                        "operation": "+",
                        "output_signal": {
                          "name": "copper-ore",
                          "quality": "epic"
                        },
                        "second_signal": {
                          "name": "copper-ore",
                          "quality": "rare"
                        }
                      }
                    },
                    "direction": 4,
                    "entity_number": 1,
                    "name": "arithmetic-combinator",
                    "position": {
                      "x": 310,
                      "y": -220.5
                    }
                  }
                ],
                "icons": [
                  {
                    "index": 1,
                    "signal": {
                      "name": "arithmetic-combinator"
                    }
                  }
                ],
                "item": "blueprint",
                "version": 562949957353472i64
              }
            })
        );
        let upgraded = upgrade(bp);
        let combinator = jaq_one(r#".blueprint.entities[]"#, upgraded);
        let signals = jaq_one(
            ".control_behavior.arithmetic_conditions | {first_signal, second_signal, output_signal}",
            combinator,
        );
        assert_eq!(
            signals,
            json!({
              "first_signal": {
                "name": "copper-ore",
                "quality": "rare"
              },
              "second_signal": {
                "name": "copper-ore",
                "quality": "epic"
              },
              "output_signal": {
                "name": "copper-ore",
                "quality": "legendary"
              },
            }),
            "wrong signals"
        );
    }

    #[test]
    // TODO: implement for decider combinators
    #[should_panic = "wrong qualities"]
    fn test_decider_combinator() {
        let bp = test_bp!(
            bp: "0eNqdkutOwzAMhd/FvzPELmVqXwVNUdp6YKl1Qi4T1ZR3x2kngQSMjZ859Tn+juoztENC54kjNGegznKA5vkMgV7YDEVjMyI00GNHPfpVZ8eW2ETrISsg7vEdmnU+KECOFAkX//yYNKexRS8D6kqOAmeDWC2XfRK3XT8qmKBZbdb1QyVrevLYLQM7BQIZvR10i6/mRBIgrkuslm/9HBWK+vUlUEfyIepv1TrrnBBZj4Lylswg5CInFsZRdgpAwBJ1s9UbEXJW965cfHevQ0ddsQmvkwQdJ1emDfdwhYG85X+V/tF4qSx3YFN0Kf5yRn82cJP8w8RRH70dNbFEQXM0Q8BS5TaWJS0f8sxDEUcRPw9dwQl9mK+petrUu7qu9ttqu9tvcv4Afs8QFg==",
            json!({
              "blueprint": {
                "entities": [
                  {
                    "control_behavior": {
                      "decider_conditions": {
                        "conditions": [
                          {
                            "first_signal": {
                              "name": "copper-ore",
                              "quality": "uncommon"
                            },
                            "second_signal": {
                              "name": "copper-ore",
                              "quality": "rare"
                            }
                          },
                          {
                            "compare_type": "and",
                            "first_signal": {
                              "name": "copper-ore",
                              "quality": "rare"
                            },
                            "second_signal": {
                              "name": "copper-ore",
                              "quality": "epic"
                            }
                          },
                          {
                            "first_signal": {
                              "name": "iron-ore",
                              "quality": "uncommon"
                            },
                            "second_signal": {
                              "name": "iron-ore",
                              "quality": "rare"
                            }
                          }
                        ],
                        "outputs": [
                          {
                            "copy_count_from_input": false,
                            "signal": {
                              "name": "copper-ore",
                              "quality": "epic"
                            }
                          },
                          {
                            "signal": {
                              "name": "iron-ore",
                              "quality": "epic"
                            }
                          }
                        ]
                      }
                    },
                    "direction": 4,
                    "entity_number": 1,
                    "name": "decider-combinator",
                    "position": {
                      "x": 310,
                      "y": -219.5
                    }
                  }
                ],
                "icons": [
                  {
                    "index": 1,
                    "signal": {
                      "name": "decider-combinator"
                    }
                  }
                ],
                "item": "blueprint",
                "version": 562949957353472i64
              }
            })
        );
        let upgraded = upgrade(bp);
        let combinator = jaq_one(r#".blueprint.entities[]"#, upgraded);
        let conditions = jaq_one(
            ".control_behavior.decider_conditions.conditions",
            combinator.clone(),
        );
        let condition_qualities = jaq_one(
            "[.[] | [.first_signal.quality, .second_signal.quality]]",
            conditions,
        );
        assert_eq!(
            condition_qualities,
            json!([["rare", "epic"], ["epic", "legendary"], ["rare", "epic"]]),
            "wrong qualities"
        );
        let output_qualities = jaq_one(
            "[.control_behavior.decider_conditions.outputs[].signal.quality]",
            combinator,
        );
        assert_eq!(output_qualities, json!(["legendary", "legendary"]));
    }

    #[test]
    fn test_selector_combinator() {
        let bp = test_bp!(
            bp: "0eNqtlN1ugzAMhd/F12Eaf+uK1CeZJhSCu0WChDmhWlXl3Wd+1G6l6wXbFcE6Pv4OSThB1fTYkTYeihNoZY2D4uUETr8Z2Qw1I1uEAhw2qLylSNm20kbyEoIAbWr8hCIOrwLQeO01Tgbjy7E0fVshsUDcMxLQWce91gwT2S+NHwUcoYiS+Pkh5zm1Ju4aBZkAxvRkm7LCd3nQbMBdtkOSk2KewbbTomwle3rqcQYuF/E0WRNZQu756GXD6FzETisIIYhFnGR1nM2KOMr2ZkgzPpfsynYsXtCT5MJN+nQ1fbKCfiaKPEnj9kgXyNLZnhSWzrNWfUv0O3q2Gj39A/peN/4H+Fxg+dX3HjzbTtKIU8Dudop8dYr4fzZgvhfnOGTb87ma7sn1Ft05dOGirtH5IQUPXrbI+iCNwjpSmlSv/c27xv8R7bHlyuXPJOCA5MYw+VOyzbbbfJPmabZJQvgCXQumXg==",
            json!({
              "blueprint": {
                "entities": [
                  {
                    "control_behavior": {
                      "index_signal": {
                        "name": "iron-ore",
                        "quality": "epic"
                      },
                      "operation": "select",
                      "select_max": true
                    },
                    "direction": 4,
                    "entity_number": 1,
                    "name": "selector-combinator",
                    "position": {
                      "x": 310,
                      "y": -218.5
                    }
                  },
                  {
                    "control_behavior": {
                      "count_signal": {
                        "name": "copper-ore",
                        "quality": "rare"
                      },
                      "operation": "count"
                    },
                    "direction": 4,
                    "entity_number": 2,
                    "name": "selector-combinator",
                    "position": {
                      "x": 310,
                      "y": -217.5
                    }
                  },
                  {
                    "control_behavior": {
                      "operation": "quality-transfer",
                      "quality_source_static": {
                        "name": "rare"
                      }
                    },
                    "direction": 4,
                    "entity_number": 3,
                    "name": "selector-combinator",
                    "position": {
                      "x": 310,
                      "y": -212.5
                    }
                  },
                  {
                    "control_behavior": {
                      "operation": "quality-filter",
                      "quality_filter": {
                        "comparator": "=",
                        "quality": "rare"
                      }
                    },
                    "direction": 4,
                    "entity_number": 4,
                    "name": "selector-combinator",
                    "position": {
                      "x": 310,
                      "y": -213.5
                    }
                  },
                  {
                    "control_behavior": {
                      "operation": "quality-transfer",
                      "quality_destination_signal": {
                        "name": "advanced-circuit",
                        "quality": "epic"
                      },
                      "quality_source_signal": {
                        "name": "copper-ore"
                      },
                      "select_quality_from_signal": true
                    },
                    "direction": 4,
                    "entity_number": 5,
                    "name": "selector-combinator",
                    "position": {
                      "x": 310,
                      "y": -211.5
                    }
                  }
                ],
                "icons": [
                  {
                    "index": 1,
                    "signal": {
                      "name": "selector-combinator"
                    }
                  }
                ],
                "item": "blueprint",
                "version": 562949957353472i64
              }
            })
        );
        let upgraded = upgrade(bp);
        // TODO implement and add assertions
        drop(upgraded)
    }

    // #[test]
    // fn test_template() {
    //     let bp = test_bp!(
    //         bp: "0eAGrrgUAAXUA+Q==",
    //         json!({})
    //     );
    //     let upgraded = upgrade(bp);
    //     let entity = jaq_one(r#".blueprint.entities[]"#, upgraded);
    // }
}
