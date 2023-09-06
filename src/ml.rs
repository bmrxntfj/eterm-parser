#[derive(Default, Debug)]
pub struct ML {
    pub raw_text: String,
    pub segments: Option<Vec<MLSegment>>,
    pub flight_no: Option<String>,
    pub flight_date: Option<String>,
    pub criteria: Option<String>,
}

impl ML {
    pub fn parse(text: &str) -> anyhow::Result<Self> {
        if text.is_empty() {
            return Err(anyhow::Error::msg(
                "ml parameter shouldn't be empty.".to_owned(),
            ));
        }
        let mut mlinfo = Self {
            raw_text: text.to_owned(),
            ..Default::default()
        };
        let mut lines = text.lines();
        if let Some(first_line) = lines.next() {
            if first_line.trim() == "NO-OP" {
                return Ok(mlinfo);
            } else if first_line.trim() != "MULTI" {
                return Err(anyhow::Error::msg("ml must start with 'MULTI'.".to_owned()));
            }
        } else {
            return Ok(mlinfo);
        }

        if let Some(second_line) = lines.next() {
            let mut info = second_line.split(&['\r', '\n']);
            mlinfo.flight_no = info.next().and_then(|x| Some(x.to_owned()));
            mlinfo.flight_date = info.next().and_then(|x| Some(x.to_owned()));
            mlinfo.criteria = info.next().and_then(|x| Some(x.to_owned()));
        } else {
            return Ok(mlinfo);
        }
        for line in lines {
            let line = line.trim_end_matches(&['+', '-']);

            if line.trim().len() == 6 {
                let segment = MLSegment {
                    org: line[0..3].to_owned(),
                    dst: line[3..].to_owned(),
                    ..Default::default()
                };
                mlinfo.segments.get_or_insert(Vec::new()).push(segment);
            } else if regex::Regex::is_match(&regex::Regex::new(r"^\s*\d+")?, line) {
                if let Some(ref mut segs) = &mut mlinfo.segments.as_mut() {
                    if let Some(seg) = segs.last_mut() {
                        let passenger = MLPassenger {
                            raw_text: line.to_owned(),
                            index: line[0..4].parse::<u8>().ok(),
                            group_count: line[7..8].parse::<u8>().ok(),
                            passenger_name: line[8..25].to_owned(),
                            pnr_code: line[25..31].to_owned(),
                            flight_class: line[31..33].to_owned(),
                            action_code: line[34..36].to_owned(),
                            seat_count: line[36..39].parse::<u8>().ok(),
                            office_code: line[40..46].to_owned(),
                            created_date: line[47..54].to_owned(),
                            passenger_info: line[60..].to_owned(),
                        };
                        if let Some(passengers) = &mut seg.passengers {
                            passengers.push(passenger);
                        }
                    }
                } else {
                    return Err(anyhow::Error::msg(
                        "parse failed,passenger must has segment.".to_owned(),
                    ));
                }
            }
        }
        Ok(mlinfo)
    }
}

#[derive(Default, Debug)]
pub struct MLSegment {
    pub org: String,
    pub dst: String,
    pub passengers: Option<Vec<MLPassenger>>,
}

#[derive(Default, Debug)]
pub struct MLPassenger {
    pub index: Option<u8>,
    pub group_count: Option<u8>,
    pub passenger_name: String,
    pub pnr_code: String,
    pub flight_class: String,
    pub action_code: String,
    pub seat_count: Option<u8>,
    pub office_code: String,
    pub created_date: String,
    pub passenger_info: String,
    pub raw_text: String,
}
