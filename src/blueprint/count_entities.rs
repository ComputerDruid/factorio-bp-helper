use std::collections::BTreeMap;

use serde::Deserialize;

use super::Quality;

#[derive(Deserialize)]
struct Top {
    #[serde(rename = "blueprint")]
    blueprint: Option<Blueprint>,
    #[serde(rename = "blueprint_book")]
    book: Option<BlueprintBook>,
}

impl Top {
    fn count(self) -> BTreeMap<(String, Quality), u64> {
        let mut result = if let Some(blueprint) = self.blueprint {
            blueprint.count()
        } else {
            BTreeMap::new()
        };
        if let Some(BlueprintBook { blueprints }) = self.book {
            for bp in blueprints {
                let counts = bp.count();
                for (key, count) in counts {
                    *result.entry(key).or_insert(0) += count
                }
            }
        }
        result
    }
}

#[derive(Deserialize)]
struct Blueprint {
    entities: Vec<Entity>,
}

#[derive(Deserialize)]
struct BlueprintBook {
    blueprints: Vec<Top>,
}

impl Blueprint {
    fn count(self) -> BTreeMap<(String, Quality), u64> {
        let mut result = BTreeMap::<(String, Quality), u64>::new();
        for entity in self.entities {
            let name = entity.name;
            let (name, count) = match name.as_ref() {
                "curved-rail-a" => ("rail".to_string(), 3),
                "curved-rail-b" => ("rail".to_string(), 3),
                "half-diagonal-rail" => ("rail".to_string(), 2),
                "straight-rail" => ("rail".to_string(), 1),
                _ => (name, 1),
            };
            let quality = Quality(entity.quality);
            let entry = result.entry((name, quality)).or_insert(0);
            *entry = entry.checked_add(count).unwrap();
        }
        result
    }
}

#[derive(Deserialize)]
struct Entity {
    name: String,
    quality: Option<String>,
}

pub(crate) fn count(json: &str) -> BTreeMap<(String, Quality), u64> {
    let top = serde_json::from_str::<Top>(json).unwrap();
    top.count()
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_count() {
        let bp = r"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";
        let json = crate::blueprint::blueprint_to_json(bp);
        let counts = count(&json);
        let expected = [
            ("arithmetic-combinator", 9),
            ("big-electric-pole", 5),
            ("constant-combinator", 9),
            ("rail", 250),
            ("fast-inserter", 46),
            ("iron-chest", 2),
            ("medium-electric-pole", 11),
            ("radar", 1),
            ("rail-chain-signal", 19),
            ("rail-signal", 2),
            ("roboport", 4),
            ("small-lamp", 19),
            ("splitter", 1),
            ("storage-chest", 43),
            ("train-stop", 2),
            ("transport-belt", 13),
        ]
        .into_iter()
        .map(|(k, v)| ((k.to_owned(), Quality(None)), v))
        .collect::<BTreeMap<(String, Quality), u64>>();
        assert_eq!(counts, expected);
    }

    #[test]
    fn test_count_rails() {
        let bp = r"0eNqdlt2OgjAQhd+l15jQTn+AV9lsNihdbYJlA2jWGN99qwKbxXGdckWA9sthOucwZ7auD/ardb5nxZm5TeM7VrydWee2vqyvz3y5t6xgbelqdkmY85X9ZgW/vCfM+t71zt533G5OH/6wX9s2LEjGnZtDe7TV6gpYlSxhX00XdjX+Cg+klcgSdgpXDQFfudZu7m+5uCQPWEHHShyrESqg1DVCBRionCBWkrE8x7GYWEUXa+5UZf5SMa2arlVNVIRjyAfE+cDJZqVMEWw2YXdl/bmqXLltQoPe6NjhjxoB05hPsK4PgO2uf8qB/zg8jVTFJ9rL8+B0A8HQPEoQsCJS8thChMbkQK+rxiWjWEnHphGVUHGVGC06r4TE2JruAoiQbJZgMVBGL+nQADInfDbdWGM1KViRRltBzjIlw7B8mRWkIUgWy5przsb8ICC6CyQlYoWMtq/Ur5tWqGXRLRWBraOTPGAxkFmW5E9oWXRizSuJHnwe/eeX8NoHkEb/+udYtJ+AR09VMEsDjnkLRPRYBYSUAYieWMFQ5MrokRUeEiYM2663+8D4ndcTdrRtd1ugtMhlnisDCqQJkf8DcPXVjw==";
        let json = crate::blueprint::blueprint_to_json(bp);
        let counts = count(&json);
        let expected = [("rail", 78)]
            .into_iter()
            .map(|(k, v)| ((k.to_owned(), Quality(None)), v))
            .collect::<BTreeMap<(String, Quality), u64>>();
        assert_eq!(counts, expected);
    }

    #[test]
    fn test_count_book() {
        let bp = "0eNrtWF2PoyAU/S8866SiaO1fmTSNH2SGVMFF7U4z8b/vpe62pisK0sdJ5mEUOOdyuedy7DfKq542kvHulAtxRofvx5sWHd4nj2qMFYKPr1v2wbNKveNZTdEB0a9G0rb1O5nxthGy83NadWjwEOMl/UKHYPBmlnW9zIV+ER6OHqK8Yx2jI/Ht4XrifZ1TCajeIpKHGtHCYsEVJwD6ePdGPHQF6PSNAFHJJC3GCZGK8Akf2+IHeyv80Bo/tcKPrPFjK3xijZ9Y4cebzzcMAH8GMdmOuJtH3HsrIliokluUq1lINzCk06hXGYKdyybMKAKXXWhOM8AbQBOr5AfhBorYjiJy2YVZ9onLLswoHMQaao7XQa1YA+kkV12cqQuoJlK8cxGMJlLspEJdpE4qxEYXcehCoUtG5CKK0ChuJ93NpAYMEetoDXgPa+ahC5XtbQKJcRqlKUlCEkYJflipnQrO1M1lbUvrvGL8w6+z4pNx6ofA8qvPKtgcTOh5IeoaCNcM3qovtLB4pgmMMPmbQD/cP2dwv+jyzCniO4Wl0dtCsZ+v30f55n119hlvqexgYAYr0Ycb4EVjVwmogs8MTqs0ZNjbOTtNsf1HkD4RzFSjh4CTNQq175ga9duCUV5Qv8mK83389FjLhayhYkdVjQXIykkBN1KUPezjosBq+L9S4Q33+TCdnxi/wPaEvI7rH09wQm2nmEf9zY4E2hGsHQmH4wB/S/Z2rSSUNbGpYPt7M1L9y6rqUicOjUo2eNtpB0lMOkjg1qViIw63NpVosuPWmWIN6qbWFBsVycTK1rRkfe3TChZIVviNqOgyhS4L8RbdxEa2OLENd8KQvMzEToWSvMzFTlHjl9nYqTSI0QXuJr/IiMNNfkSTHTf5RavG1v76Jpa+1v7+Jj/3t+7+xrGTBnX1kDih3moXbPpvqAmV4ndowcSDxkaOWz9HgrkPGf/2I7SHMnVe9PTvy2UBb/gD/vaEEw==";
        let json = crate::blueprint::blueprint_to_json(bp);
        let counts = count(&json);
        let expected = [
            (("assembling-machine-3", Some("uncommon")), 2),
            (("bulk-inserter", None), 4),
            (("express-transport-belt", None), 33),
            (("long-handed-inserter", None), 2),
            (("medium-electric-pole", None), 2),
            (("turbo-transport-belt", None), 9),
        ]
        .into_iter()
        .map(|((name, quality), v)| ((name.to_owned(), Quality(quality.map(|q| q.to_owned()))), v))
        .collect::<BTreeMap<(String, Quality), u64>>();
        assert_eq!(counts, expected);
    }

    #[test]
    fn test_count_book_with_book() {
        let bp = "0eNrtWF2PoyAU/S8866Si+NG/MmkatWSGjIKL2p2m8b/vpe62pisK0sdJ+tAKnHO53HM59oqKqqeNZLw7FkJ8of318aRF+/erxQQ1xkrBx8ct++B5pZ7xvKZoj+h3I2nb+p3MedsI2fkFrTo0eIjxE/1G+2DwZpZ1vSyEfhEeDh6ivGMdoyPx7cflyPu6oBJQvUUkDzWihcWCK04A9PHujXjoAtDZGwGiE5O0HCdEKsInfGyLH6RW+KE1fmaFH1njx1b4xBo/scKPN59vGAD+DGKyHXE3j5h6KyJYqJJblKtZyDYwZNOoVxmCncsmzCgCl11oTjPAG0ATq+QH4QaK2I4ictmFWfaJyy7MKBzEGmqO10GtWAPpJFddnJkLqCZSvHMRjCZS7KRCXaROKsRGF3HoQqFLRuQiitAobifdzaQGDBHraA14D2vmoTOV7W0CiXEWZRlJQhJGCX5YqZ0KztTN5W1L66Ji/MOv8/KTceqHwPKrzyvYHEzoeSnqGgjXDN6qL7SweKYJjDD5m0A/TJ8zmC66PHOK+E5hafS2UKTz9fso36KvvnzGWyo7GJjBSvThBnjR2FUCquAzh9M6GTKkds5OU2z/EWRPBDPV6CHgZI1C7TumRv22ZJSX1G/y8us+fnys5ULWULGjqsYCZKdJATdSnHrYx1mB1fC9UuEN9/kwnR8ZP8P2hLyM6x+/4ITaTjGP+psdCbQjWDsSDocBPkv2dq0klDWxqWD7ezNS/cuq6jInDo1KNnjbaQdJTDpI4NalYiMOtzaVaLLj1pliDeqm1hQbFcnEytb0xPrapxUskKz0G1HRZQpdFuItuomNbHFiG+6EIXmZiZ0KJXmZi52ixi+zsVNpEKML3E1+kRGHm/yIJjtu8otWja399U0sfa39/U1+7m/d/Y1jJw3q6iFxQr3VLtj031ATKsXv0IKJB42NHLa+jgRzLzL+7U9oD+XqvOjx35uL2evNS/CGP6QVqbc=";
        let json = crate::blueprint::blueprint_to_json(bp);
        let counts = count(&json);
        let expected = [
            (("assembling-machine-3", Some("uncommon")), 2),
            (("bulk-inserter", None), 4),
            (("express-transport-belt", None), 33),
            (("long-handed-inserter", None), 2),
            (("medium-electric-pole", None), 2),
            (("turbo-transport-belt", None), 9),
        ]
        .into_iter()
        .map(|((name, quality), v)| ((name.to_owned(), Quality(quality.map(|q| q.to_owned()))), v))
        .collect::<BTreeMap<(String, Quality), u64>>();
        assert_eq!(counts, expected);
    }
}
