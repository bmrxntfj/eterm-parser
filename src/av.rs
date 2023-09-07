/// The result that av text parsed.
#[derive(Default, Debug)]
pub struct Av<'a> {
    pub dpt: Option<&'a str>,
    pub arr: Option<&'a str>,
    pub date: Option<&'a str>,
    pub query: Option<&'a str>,
    pub flights: Vec<AvFlight<'a>>,
    pub raw_text: &'a str,
}

/// The flights of an Av.
#[derive(Default, Debug)]
pub struct AvFlight<'a> {
    pub index: u8,
    pub is_share_flight: bool,
    pub flight_no: &'a str,
    pub real_flight_no: Option<&'a str>,
    pub flight_status: &'a str,
    pub dpt: &'a str,
    pub arr: &'a str,
    pub take_off: &'a str,
    pub landing: &'a str,
    pub model: &'a str,
    pub dpt_terminal: Option<&'a str>,
    pub arr_terminal: Option<&'a str>,
    pub duration: Option<&'a str>,
    pub is_eticket: bool,
    pub meal: &'a str,
    pub stops: u8,
    pub cabins: Vec<AvCabin<'a>>,
    pub raw_text: &'a str,
    pub is_marriage_flight: bool,
    pub union_flights: Vec<AvFlight<'a>>,
    pub asr: bool,
}

/// The cabins of an AvFlight.
#[derive(Default, Debug)]
pub struct AvCabin<'a> {
    pub name: &'a str,
    pub state: &'a str,
    pub is_sub_cabin: bool,
    pub raw_text: &'a str,
}

impl<'a> AvCabin<'a> {
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
        match self.state {
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

impl<'a> Av<'a> {
    pub fn parse(text: &'a str) -> anyhow::Result<Self> {
        if text.is_empty() {
            return Err(anyhow::Error::msg(
                "av parameter shouldn't be empty.".to_owned(),
            ));
        }
        let mut avinfo = Self {
            raw_text: text,
            ..Default::default()
        };

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

    fn parse_query(text: &'a str, avinfo: &mut Av<'a>) -> anyhow::Result<()> {
        match regex::Regex::new(
            r"(?<DATE>\d{2}[A-Z]{3}(?:\d{2})?)\(([A-Z]{3})\)[\x1D\s](?<DPT>[A-Z]{3})(?<ARR>[A-Z]{3})",
            )?
            .captures(text)
            {
                Some(caps) => match (caps.name("DATE"),caps.name("DPT"), caps.name("ARR")) {
                    (Some(date),Some(dpt), Some(arr)) => {
                        avinfo.dpt = Some(dpt.as_str());
                        avinfo.arr = Some(arr.as_str());
                        avinfo.date=Some(date.as_str());
                        avinfo.query=Some(text);
                    }
                    _ => {},
                },
                _ => {},
            }
        Ok(())
    }

    ///it easy to parse a text of flight of av specifically. 
    pub fn parse_flight(text: &'a str) -> anyhow::Result<AvFlight> {
        let mut flight = AvFlight {
            raw_text: text,
            ..Default::default()
        };

        for xs in text.split("\n ") {
            if flight.dpt.len() == 0 {
                let _ = Self::parse_first_flight(xs, &mut flight);
            } else {
                flight.is_marriage_flight = true;
                let union_flight = Self::parse_union_flight(xs, &flight)?;
                flight.union_flights.push(union_flight);
            }
        }
        Ok(flight)
    }

    fn parse_first_flight(text: &'a str, flight: &mut AvFlight<'a>) -> anyhow::Result<()> {
        for line in text.lines() {
            if regex::Regex::new(r"^\d").unwrap().is_match(line) {
                if line.len() < 78 {
                    continue;
                }
                flight.index = line[0..1].parse::<u8>()?;
                flight.is_share_flight = &line[3..4] == "*";
                flight.flight_no = line[4..11].trim();
                flight.dpt = line[47..50].trim();
                flight.arr = line[50..53].trim();
                flight.take_off = line[54..58].trim();
                flight.landing = line[61..65].trim();
                flight.model = line[68..71].trim();
                flight.stops = line[72..73].parse::<u8>()?;
                flight.asr = &line[73..74] == "^";
                flight.flight_status = line[12..15].trim();
                flight.is_eticket = &line[73..74] == "E";
                flight.meal = line[74..75].trim();
                flight.cabins = Self::parse_cabin(&line[15..47]);
            } else if line.starts_with(">") {
                flight.real_flight_no = Some(line[4..11].trim());
                match line.len() {
                    n if n > 73 => {
                        flight.duration = Some(&line[74..]);
                    }
                    n if n > 71 => {
                        flight.arr_terminal = Some(&line[71..73]);
                    }
                    n if n > 68 => {
                        flight.dpt_terminal = Some(&line[68..70]);
                    }
                    _ => {}
                }
                let mut cabins = Self::parse_cabin(&line[15..61]);
                flight.cabins.append(&mut cabins);
            } else if line.starts_with("               **") {
                let mut cabins = Self::parse_cabin(line[15..47].trim_end());
                flight.cabins.append(&mut cabins);
            }
        }
        Ok(())
    }

    fn parse_union_flight(text: &'a str, flight: &AvFlight<'a>) -> anyhow::Result<AvFlight<'a>> {
        //let mut raw_text=text.to_owned();
        //raw_text.insert_str(0, " ");
        //println!("{}",text);
        let mut union_flight = AvFlight {
            raw_text: &text,
            ..Default::default()
        };
        
        for line in union_flight.raw_text.lines() {
            if regex::Regex::new(r"^\s+\*?[A-Z0-9]{2}\d+")?.is_match(line) {
                union_flight.is_share_flight = &line[2..3] == "*";
                union_flight.flight_no = line[3..10].trim();
                union_flight.dpt = flight.arr;
                union_flight.arr = line[49..52].trim();
                union_flight.take_off = line[53..57].trim();
                union_flight.landing = line[60..64].trim();
                union_flight.model = line[67..70].trim();
                union_flight.stops = line[71..72].parse::<u8>()?;
                union_flight.flight_status = line[11..14].trim();
                match line.len() {
                    n if n > 76 => {
                        union_flight.is_eticket = &line[76..77] == "E";
                        union_flight.meal = line[73..74].trim();
                    }
                    n if n > 73 => {
                        union_flight.meal = line[73..74].trim();
                    }
                    _ => {}
                }
                let mut cabins = Self::parse_cabin(&line[14..60]);
                union_flight.cabins.append(&mut cabins);
            } else if line.starts_with(">") {
                union_flight.real_flight_no = Some(&line[4..11]);
                match line.len() {
                    n if n > 73 => {
                        union_flight.duration = Some(&line[74..]);
                    }
                    n if n > 71 => {
                        union_flight.arr_terminal = Some(&line[71..73]);
                    }
                    n if n > 68 => {
                        union_flight.dpt_terminal = Some(&line[68..70]);
                    }
                    _ => {}
                }
                let mut cabins = Self::parse_cabin(&line[15..61]);
                union_flight.cabins.append(&mut cabins);
            } else if line.starts_with("               **") {
                let mut cabins = Self::parse_cabin(line[15..47].trim_end());
                union_flight.cabins.append(&mut cabins);
            }
        }
        Ok(union_flight)
    }

    ///it easy to parse a text of cabins of flight specifically. 
    pub fn parse_cabin(text: &'a str) -> Vec<AvCabin> {
        let is_sub_cabin = text.starts_with("**");
        if is_sub_cabin {
            text[2..]
                .split_whitespace()
                .into_iter()
                .map(|x| AvCabin {
                    name: &x[0..1],
                    state: &x[1..1],
                    is_sub_cabin,
                    raw_text: x,
                })
                .collect::<Vec<_>>()
        } else {
            text.split_whitespace()
                .into_iter()
                .map(|x| AvCabin {
                    name: &x[0..1],
                    state: &x[1..1],
                    is_sub_cabin,
                    raw_text: x,
                })
                .collect::<Vec<_>>()
        }
    }
}
