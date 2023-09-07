#[derive(Default, Debug)]
pub struct Ml<'a> {
    pub raw_text: &'a str,
    pub segs: Option<Vec<MlSeg<'a>>>,
    pub flight_no: Option<&'a str>,
    pub flight_date: Option<&'a str>,
    pub criteria: Option<&'a str>,
}

impl<'a> Ml<'a> {
    pub fn parse(text: &'a str) -> anyhow::Result<Self> {
        if text.is_empty() {
            return Err(anyhow::Error::msg(
                "ml parameter shouldn't be empty.".to_owned(),
            ));
        }
        let mut mlinfo = Self {
            raw_text: text,
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
            let mut info = second_line.split(&[' ']).filter(|x|!x.is_empty());
            mlinfo.flight_no = info.next().and_then(|x| Some(x));
            mlinfo.flight_date = info.next().and_then(|x| Some(x));
            mlinfo.criteria = info.next().and_then(|x| Some(x));
        } else {
            return Ok(mlinfo);
        }
        for line in lines {
            let line = line.trim_end_matches(&['+', '-']);

            if line.trim().len() == 6 {
                let segment = MlSeg {
                    org: &line[0..3],
                    dst: &line[3..],
                    ..Default::default()
                };
                mlinfo.segs.get_or_insert(Vec::new()).push(segment);
            } else if regex::Regex::is_match(&regex::Regex::new(r"^\s*\d+")?, line) {
                if let Some(ref mut segs) = &mut mlinfo.segs.as_mut() {
                    if let Some(seg) = segs.last_mut() {
                        let passenger = MlPassenger {
                            raw_text: line,
                            index: line[0..4].parse::<u8>().ok(),
                            group_count: line[7..8].parse::<u8>().ok(),
                            passenger_name: &line[8..25],
                            pnr_code: &line[25..31],
                            flight_class: &line[31..33],
                            action_code: &line[34..36],
                            seat_count: line[36..39].parse::<u8>().ok(),
                            office_code: &line[40..46],
                            created_date: &line[47..54],
                            passenger_info: &line[60..],
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
pub struct MlSeg<'a> {
    pub org: &'a str,
    pub dst: &'a str,
    pub passengers: Option<Vec<MlPassenger<'a>>>,
}

#[derive(Default, Debug)]
pub struct MlPassenger<'a> {
    pub index: Option<u8>,
    pub group_count: Option<u8>,
    pub passenger_name: &'a str,
    pub pnr_code: &'a str,
    pub flight_class: &'a str,
    pub action_code: &'a str,
    pub seat_count: Option<u8>,
    pub office_code: &'a str,
    pub created_date: &'a str,
    pub passenger_info: &'a str,
    pub raw_text: &'a str,
}
