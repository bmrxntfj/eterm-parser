/// The result that av text parsed.
#[derive(Default, Debug)]
pub struct AV {
    pub dpt: Option<String>,
    pub arr: Option<String>,
    pub date: Option<String>,
    pub query: Option<String>,
    pub flights: Vec<AVFlight>,
    pub raw_text: String,
}

/// The flights of an AV.
#[derive(Default, Debug)]
pub struct AVFlight {
    pub index: u8,
    pub is_share_flight: bool,
    pub flight_no: String,
    pub real_flight_no: Option<String>,
    pub flight_status: String,
    pub dpt: String,
    pub arr: String,
    pub take_off: String,
    pub landing: String,
    pub model: String,
    pub dpt_terminal: Option<String>,
    pub arr_terminal: Option<String>,
    pub duration: Option<String>,
    pub is_eticket: bool,
    pub meal: String,
    pub stops: u8,
    pub cabins: Vec<AVCabin>,
    pub raw_text: String,
    pub is_marriage_flight: bool,
    pub union_flights: Vec<AVFlight>,
    pub asr: bool,
}

/// The cabins of an AVFlight.
#[derive(Default, Debug)]
pub struct AVCabin {
    pub name: String,
    pub state: String,
    pub is_sub_cabin: bool,
    pub raw_text: String,
}

impl AVCabin {
    /// Return whether the seat of the cabin is number.
    /// such as 1-9.
    pub fn is_num_state(&self) -> bool {
        match self.state.chars().next() {
            Some(c) => c.is_ascii_digit(),
            _ => false,
        }
    }

    /// Return whether the seat of the cabin is available.
    /// such as 1-9,A.
    pub fn is_available(&self) -> bool {
        match self.state.chars().next() {
            Some('0') | Some('A') => false,
            Some(c) => c.is_ascii_digit(),
            _ => false,
        }
    }

    /// Return whether the seat of the cabin is locked.
    /// such as C,Q.
    pub fn is_locked(&self) -> bool {
        match self.state.as_str() {
            "C" | "Q" => true,
            _ => false,
        }
    }

    /// Return whether the seat of the cabin is soldout.
    /// it's not available and locked.
    pub fn is_soldout(&self) -> bool {
        !self.is_available() && !self.is_locked()
    }

    /// Return quantity of the seat of the cabin.
    /// when it's number, return the number.
    /// when it's 'A',return 9.
    pub fn seat_quantity(&self) -> Option<u8> {
        match self.state.chars().next() {
            Some('A') => Some(9),
            Some(c) if c.is_ascii_digit() => c.to_digit(10).map_or(None, |x| Some(x as u8)),
            _ => None,
        }
    }
}

impl AV {
    pub fn parse(text: &str) -> anyhow::Result<Self> {
        if text.is_empty() {
            return Err(anyhow::Error::msg(
                "av parameter shouldn't be empty.".to_owned(),
            ));
        }
        let mut avinfo = Self {
            raw_text: text.to_owned(),
            ..Default::default()
        };
        match text.find("\n1") {
            Some(end) => {
                let firstline = &text[0..end].trim();
                let re = regex::Regex::new(
                    r"(?<DATE>\d{2}[A-Z]{3}(?:\d{2})?)\((?<WEEK>[A-Z]{3})\)[\x1D\s](?<ORG>[A-Z]{3})(?<DST>[A-Z]{3})(.*)",
                )?;
                match re.captures(firstline) {
                    Some(caps) => match (caps.name("DATE"), caps.name("ORG"), caps.name("DST")) {
                        (Some(date), Some(org), Some(dst)) => {
                            avinfo.date = Some(date.as_str().to_owned());
                            avinfo.dpt = Some(org.as_str().to_owned());
                            avinfo.arr = Some(dst.as_str().to_owned());
                        }
                        _ => {}
                    },
                    _ => {}
                }
            }
            _ => {}
        }

        let mut start = 0;
        let mut index = 1;
        loop {
            match text.rfind(&format!("\n{}", index)) {
                Some(end) => {
                    let xs = &text[start..end].trim_start();
                    start = end;
                    if avinfo.dpt.is_none() {
                        let _ = Self::parse_query(xs, &mut avinfo);
                    } else {
                        let curr_flight = Self::parse_flight(xs)?;
                        avinfo.flights.push(curr_flight);
                    }
                }
                _ => {
                    let xs = &text[start..].trim_start();
                    let curr_flight = Self::parse_flight(xs)?;
                    avinfo.flights.push(curr_flight);
                    break;
                }
            }
            index += 1;
        }

        Ok(avinfo)
    }

    pub fn parse_query(text: &str, avinfo: &mut AV) -> anyhow::Result<()> {
        match regex::Regex::new(
            r"(?<DATE>\d{2}[A-Z]{3}(?:\d{2})?)\(([A-Z]{3})\)[\x1D\s](?<DPT>[A-Z]{3})(?<ARR>[A-Z]{3})",
            )?
            .captures(text)
            {
                Some(caps) => match (caps.name("DATE"),caps.name("DPT"), caps.name("ARR")) {
                    (Some(date),Some(dpt), Some(arr)) => {
                        avinfo.dpt = Some(dpt.as_str().to_owned());
                        avinfo.arr = Some(arr.as_str().to_owned());
                        avinfo.date=Some(date.as_str().to_owned());
                        avinfo.query=Some(text.to_owned());
                    }
                    _ => {},
                },
                _ => {},
            }
        Ok(())
    }

    pub fn parse_flight(text: &str) -> anyhow::Result<AVFlight> {
        let mut flight = AVFlight {
            raw_text: text.to_owned(),
            ..Default::default()
        };

        for xs in text.split("\n ") {
            if flight.dpt.len() == 0 {
                let _ = Self::parse_first_flight(xs, &mut flight);
            } else {
                flight.is_marriage_flight = true;
                let union_flight = Self::parse_union_flight(xs.to_owned(), &flight)?;
                flight.union_flights.push(union_flight);
            }
        }
        Ok(flight)
    }

    pub fn parse_first_flight(text: &str, flight: &mut AVFlight) -> anyhow::Result<()> {
        for line in text.lines() {
            if regex::Regex::new(r"^\d").unwrap().is_match(line) {
                if line.len() < 78 {
                    continue;
                }
                flight.index = line[0..1].parse::<u8>()?;
                flight.is_share_flight = line[3..4].as_ptr() == "*".as_ptr();
                flight.flight_no = line[4..11].trim().to_owned();
                flight.dpt = line[47..50].trim().to_owned();
                flight.arr = line[50..53].trim().to_owned();
                flight.take_off = line[54..58].trim().to_owned();
                flight.landing = line[61..65].trim().to_owned();
                flight.model = line[68..71].trim().to_owned();
                flight.stops = line[72..73].parse::<u8>()?;
                flight.asr = line[73..74].as_ptr() == "^".as_ptr();
                flight.flight_status = line[12..15].trim().to_owned();
                flight.is_eticket = line[73..74].as_ptr() == "E".as_ptr();
                flight.meal = line[74..75].trim().to_owned();
                flight.cabins = Self::parse_cabin(line[15..47].as_ref());
            } else if line.starts_with(">") {
                flight.real_flight_no = Some(line[4..11].to_owned());
                match line.len() {
                    n if n > 73 => {
                        flight.duration = Some(line[74..].to_owned());
                    }
                    n if n > 71 => {
                        flight.arr_terminal = Some(line[71..73].to_owned());
                    }
                    n if n > 68 => {
                        flight.dpt_terminal = Some(line[68..70].to_owned());
                    }
                    _ => {}
                }
                let mut cabins = Self::parse_cabin(line[15..61].as_ref());
                flight.cabins.append(&mut cabins);
            } else if line.starts_with("               **") {
                let mut cabins = Self::parse_cabin(line[15..47].trim_end().as_ref());
                flight.cabins.append(&mut cabins);
            }
        }
        Ok(())
    }

    pub fn parse_union_flight(text: String, flight: &AVFlight) -> anyhow::Result<AVFlight> {
        let mut union_flight = AVFlight {
            raw_text: text,
            ..Default::default()
        };
        union_flight.raw_text.insert_str(0, " ");
        for line in union_flight.raw_text.lines() {
            if regex::Regex::new(r"^\s+\*?[A-Z0-9]{2}\d+")?.is_match(line) {
                union_flight.is_share_flight = line[3..4].as_ptr() == "*".as_ptr();
                union_flight.flight_no = line[4..11].trim().to_owned();
                union_flight.dpt = flight.arr.clone();
                union_flight.arr = line[50..53].trim().to_owned();
                union_flight.take_off = line[54..58].trim().to_owned();
                union_flight.landing = line[61..65].trim().to_owned();
                union_flight.model = line[68..71].trim().to_owned();
                union_flight.stops = line[72..73].parse::<u8>()?;
                union_flight.flight_status = line[12..15].trim().to_owned();
                match line.len() {
                    n if n > 77 => {
                        union_flight.is_eticket = line[77..78].as_ptr() == "E".as_ptr();
                        union_flight.meal = line[74..75].trim().to_owned();
                    }
                    n if n > 74 => {
                        union_flight.meal = line[74..75].trim().to_owned();
                    }
                    _ => {}
                }
                let mut cabins = Self::parse_cabin(line[15..61].as_ref());
                union_flight.cabins.append(&mut cabins);
            } else if line.starts_with(">") {
                union_flight.real_flight_no = Some(line[4..11].to_owned());
                match line.len() {
                    n if n > 73 => {
                        union_flight.duration = Some(line[74..].to_owned());
                    }
                    n if n > 71 => {
                        union_flight.arr_terminal = Some(line[71..73].to_owned());
                    }
                    n if n > 68 => {
                        union_flight.dpt_terminal = Some(line[68..70].to_owned());
                    }
                    _ => {}
                }
                let mut cabins = Self::parse_cabin(line[15..61].as_ref());
                union_flight.cabins.append(&mut cabins);
            } else if line.starts_with("               **") {
                let mut cabins = Self::parse_cabin(line[15..47].trim_end().as_ref());
                union_flight.cabins.append(&mut cabins);
            }
        }
        Ok(union_flight)
    }

    pub fn parse_cabin(text: &str) -> Vec<AVCabin> {
        let is_sub_cabin = text.starts_with("**");
        if is_sub_cabin {
            text[2..]
                .split_whitespace()
                .into_iter()
                .map(|x| AVCabin {
                    name: x[0..1].to_owned(),
                    state: x[1..1].to_owned(),
                    is_sub_cabin,
                    raw_text: x.to_owned(),
                })
                .collect::<Vec<_>>()
        } else {
            text.split_whitespace()
                .into_iter()
                .map(|x| AVCabin {
                    name: x[0..1].to_owned(),
                    state: x[1..1].to_owned(),
                    is_sub_cabin,
                    raw_text: x.to_owned(),
                })
                .collect::<Vec<_>>()
        }
    }
}
