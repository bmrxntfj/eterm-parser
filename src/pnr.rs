use crate::util;

/// The result that pnr text parsed.
#[derive(Default, Debug)]
pub struct Pnr<'a> {
    pub infos: Option<Vec<&'a str>>,
    pub ssr_items: Option<Vec<SSR<'a>>>,
    pub osi_items: Option<Vec<OSI<'a>>>,
    pub seg_items: Option<Vec<SEG<'a>>>,
    pub nm_items: Option<Vec<NM<'a>>>,
    pub rmk_items: Option<Vec<RMK<'a>>>,
    pub other_items: Option<Vec<OtherItem<'a>>>,
    pub is_ticket_pnr: Option<bool>,
    pub is_cancelled_pnr: Option<bool>,
    pub is_group_pnr: Option<bool>,
    pub group_pnr_name: Option<&'a str>,
    pub pax_count: Option<u8>,
    pub pnr_code: Option<&'a str>,
    pub bpnr_code: Option<&'a str>,
    pub has_married_segment: Option<bool>,
    pub office_no: Option<&'a str>,
}

impl<'a> Pnr<'a> {
    pub fn parse(text: &'a str) -> anyhow::Result<Self> {
        if text.is_empty() {
            return Err(anyhow::Error::msg(
                "pnr parameter shouldn't be empty.".to_owned(),
            ));
        }
        let mut pnr = Self {
            ..Default::default()
        };

        let mut info_parsed = false;
        let mut index = 0u8;
        for line in text.lines() {
            if !info_parsed && line.starts_with(" 1.") {
                info_parsed = true;
            }
            if !info_parsed {
                if line.contains("**ELECTRONIC TICKET PNR**") {
                    pnr.is_ticket_pnr = Some(true);
                }
                if line.contains("*THIS PNR WAS ENTIRELY CANCELLED*") {
                    pnr.is_cancelled_pnr = Some(true);
                }
                if line.contains("MARRIED SEGMENT EXIST IN THE PNR") {
                    pnr.has_married_segment = Some(true);
                }
                pnr.infos.get_or_insert(Vec::new()).push(line);
            } else {
                match line.trim_start() {
                    x if x.starts_with("1.") => {
                        let re = regex::Regex::new(r"(?<NMS>1\.(.*))(?<PNRCODE>\w{6})\s*$")?;
                        match re.captures(line) {
                            Some(caps) => match (caps.name("NMS"), caps.name("PNRCODE")) {
                                (Some(nms), Some(pnrcode)) => {
                                    pnr.pnr_code = Some(pnrcode.as_str());
                                    if let Ok(items) = NM::parse(index, nms.as_str()) {
                                        pnr.nm_items = Some(items);
                                    }
                                }
                                _ => {}
                            },
                            _ => {}
                        }
                    }
                    x if regex::Regex::is_match(
                        &regex::Regex::new(
                            r"(?<GROUPPNRNAME>.*)\s*NM(?<PAXCOUNT>\d+)\s+(?<PNRCODE>\w{6})/(\w{2})",
                        )?,
                        x,
                    ) =>
                    {
                        pnr.is_group_pnr = Some(true);
                        if let Some(caps) = regex::Regex::captures(
                            &regex::Regex::new(
                                r"(?<GROUPPNRNAME>.*)\s*NM(?<PAXCOUNT>\d+)\s+(?<PNRCODE>\w{6})/(\w{2})",
                            )?,
                            x,
                        ) {
                            match (
                                caps.name("GROUPPNRNAME"),
                                caps.name("PAXCOUNT"),
                                caps.name("PNRCODE"),
                            ) {
                                (Some(pnr_groupname), Some(paxcount), Some(pnrcode)) => {
                                    pnr.group_pnr_name = Some(pnr_groupname.as_str());
                                    pnr.pnr_code = Some(pnrcode.as_str());
                                    pnr.pax_count = paxcount.as_str().parse::<u8>().ok();
                                }
                                _ => {}
                            }
                        }
                    }
                    x if x.starts_with(&format!("{}. ", index)) => {
                        if let Ok(item) = SEG::parse(index, line) {
                            pnr.seg_items.get_or_insert(Vec::new()).push(item);
                        }
                    }
                    x if x.starts_with(&format!("{}SSR", index)) => {
                        if let Ok(item) = SSR::parse(index, line) {
                            pnr.ssr_items.get_or_insert(Vec::new()).push(item);
                        }
                    }
                    x if x.starts_with(&format!("{}OSI", index)) => {
                        if let Ok(item) = OSI::parse(index, line) {
                            pnr.osi_items.get_or_insert(Vec::new()).push(item);
                        }
                    }
                    x if x.starts_with(&format!("{}RMK", index)) => {
                        if let Ok(item) = RMK::parse(index, line) {
                            pnr.rmk_items.get_or_insert(Vec::new()).push(item);
                        }
                    }
                    _ => {
                        if let Ok(item) = OtherItem::parse(index, line) {
                            pnr.other_items.get_or_insert(Vec::new()).push(item);
                        }
                    }
                }
            }
            index += 1;
        }
        Self::fix_nm(&mut pnr);
        if pnr.pax_count.is_none() {
            pnr.pax_count = pnr.nm_items.as_ref().and_then(|x| Some(x.len() as u8));
        }
        pnr.office_no = pnr.other_items.as_ref().and_then(|x| {
            x.iter().find_map(|n| {
                if n.item_type == "OFFICE" {
                    Some(n.raw.trim())
                } else {
                    None
                }
            })
        });
        pnr.bpnr_code = pnr.rmk_items.as_ref().and_then(|x| {
            x.iter().find_map(|n| {
                if n.service_code.is_some_and(|s| s == "CA") {
                    n.text.as_ref().and_then(|k| Some(&k[0..6]))
                } else {
                    None
                }
            })
        });
        Ok(pnr)
    }

    /// fill id info with ssr.
    fn fix_nm(pnr: &mut Pnr) {
        match (&pnr.ssr_items, &mut pnr.nm_items) {
            (Some(ssrs), Some(nms)) => {
                nms.iter_mut().for_each(|x| {
                    if let Some(ssr) = ssrs.iter().find(|s| {
                        s.passenger_index.is_some_and(|n| n == x.index)
                            && s.service_code.is_some_and(|n| n == "FOID")
                    }) {
                        if let Some(tx) = &ssr.text {
                            x.id_type = Some(&tx[0..2]);
                            x.id_number = Some(&tx[2..]);
                        }
                    }
                });
            }
            _ => {}
        }
    }
}

/// This is a simple item, except NM,SSR,OSI,SEG,RMK, etc.
#[derive(Default, Debug)]
pub struct OtherItem<'a> {
    pub index: u8,
    pub item_type: &'a str,
    pub raw: &'a str,
}

impl<'a> OtherItem<'a> {
    pub fn parse(index: u8, raw: &'a str) -> anyhow::Result<Self> {
        let item_type = match raw {
            x if x.starts_with(&format!("{}TL", index)) => "TL",
            x if x.starts_with(&format!("{}FN", index)) => "FN",
            x if x.starts_with(&format!("{}FC", index)) => "FC",
            x if x.starts_with(&format!("{}FP", index)) => "FP",
            x if x.starts_with(&format!("{}EI", index)) => "EI",
            x if x.starts_with(&format!("{}XN", index)) => "XN",
            x if x.starts_with(&format!("{}TC", index)) => "TC",
            x if x.starts_with(&format!("{}TN/", index)) => "TN",
            x if regex::Regex::is_match(&regex::Regex::new(r"^\d{2}\.[A-Z]{3}\d{3}$")?, x) => {
                "OFFICE"
            }
            _ => "TEXT",
        };
        Ok(Self {
            index,
            item_type: item_type,
            raw: raw,
        })
    }
}

/// The passenger infomation of pnr.
#[derive(Default, Debug)]
pub struct NM<'a> {
    pub index: u8,
    pub raw: &'a str,
    pub name: Option<&'a str>,
    //pub ssr_items: Option<Vec<SSR>>,
    //pub osi_items: Option<Vec<OSI>>,
    pub id_number: Option<&'a str>,
    pub id_type: Option<&'a str>,
}

impl<'a> NM<'a> {
    pub fn parse(index: u8, raw: &'a str) -> anyhow::Result<Vec<Self>> {
        let re = regex::Regex::new(r"(\d+\.)")?;
        let nms = re
            .split(raw)
            .filter_map(|cap| match cap.trim() {
                x if x.is_empty() => None,
                x if x.ends_with(".") => None,
                x => Some(Self {
                    index,
                    raw: cap.trim(),
                    name: Some(x),
                    ..Default::default()
                }),
            })
            .collect::<Vec<_>>();
        Ok(nms)
    }
}

/// The flight segment infomation of pnr.
#[derive(Default, Debug)]
pub struct SEG<'a> {
    pub index: u8,
    pub raw: &'a str,
    pub org: Option<&'a str>,
    pub dst: Option<&'a str>,
    pub seat_class: Option<&'a str>,
    pub flight_date: Option<&'a str>,
    pub takeoff: Option<&'a str>,
    pub landing: Option<&'a str>,
    pub landing_addday: Option<u8>,
    pub action_code: Option<&'a str>,
    pub action_code_qty: Option<u8>,
    pub flight_no: Option<&'a str>,
    pub is_share: Option<bool>,
}

impl<'a> SEG<'a> {
    pub fn parse(index: u8, raw: &'a str) -> anyhow::Result<Self> {
        let re = regex::Regex::new(
            r"(?<FLIGHTNO>\*?\w{5,6})\s+(?<SEATCLASS>[A-Z]\d?)\s+[A-Z]{2}(?<FLIGHTDATE>\d{2}[A-Z]{3}(?:\d{2})?)\s*(?<ORG>[A-Z]{3})(?<DST>[A-Z]{3})\s*(?<ACTIONCODE>[A-Z]{2})(?<ACTIONCODEQTY>\d{1,2})\s*(?<DEPTIME>\d{4})\s*(?<ARRTIME>\d{4})(?:\+(?<ADDDAY>\d))?",
        )?;
        match re.captures(raw) {
            Some(caps) => match (
                caps.name("FLIGHTNO"),
                caps.name("SEATCLASS"),
                caps.name("FLIGHTDATE"),
                caps.name("ORG"),
                caps.name("DST"),
                caps.name("ACTIONCODE"),
                caps.name("ACTIONCODEQTY"),
                caps.name("DEPTIME"),
                caps.name("ARRTIME"),
                caps.name("ADDDAY"),
            ) {
                (
                    Some(flight_no),
                    Some(seat_class),
                    Some(flight_date),
                    Some(org),
                    Some(dst),
                    Some(action_code),
                    Some(action_code_qty),
                    Some(takeoff),
                    Some(landing),
                    addday,
                ) => Ok(Self {
                    index,
                    raw: raw,
                    flight_no: Some(flight_no.as_str()),
                    seat_class: Some(seat_class.as_str()),
                    flight_date: Some(flight_date.as_str()),
                    org: Some(org.as_str()),
                    dst: Some(dst.as_str()),
                    action_code: Some(action_code.as_str()),
                    action_code_qty: action_code_qty.as_str().parse::<u8>().ok(), // action_code_qty.and_then(|x|x.as_str().parse::<u8>().ok()),
                    takeoff: Some(takeoff.as_str()),
                    landing: Some(landing.as_str()),
                    landing_addday: util::regex_extact_value::<u8>(addday), // passenger_index.and_then(|x|x.as_str().parse::<u8>().ok()),
                    is_share: Some(flight_no.as_str().starts_with("*")),
                }),
                _ => Ok(Self {
                    index,
                    raw: raw,
                    ..Default::default()
                }),
            },
            _ => Ok(Self {
                index,
                raw: raw,
                ..Default::default()
            }),
        }
    }
}

/// The ssr infomation of pnr.
#[derive(Default, Debug)]
pub struct SSR<'a> {
    pub index: u8,
    pub raw: &'a str,
    pub service_code: Option<&'a str>,
    pub action_code: Option<&'a str>,
    pub action_code_qty: Option<u8>,
    pub airline: Option<&'a str>,
    pub text: Option<&'a str>,
    pub passenger_index: Option<u8>,
    pub segment_index: Option<u8>,
}

impl<'a> SSR<'a> {
    pub fn parse(index: u8, raw: &'a str) -> anyhow::Result<Self> {
        let re = regex::Regex::new(
            r"SSR (?<SERVICECODE>[A-Z]+) (?<AIRLINE>\w{2}) (?:(?<ACTIONCODE>\w{2})(?<ACTIONCODEQTY>\d|/+)?\s+)?(?<TEXT>[^\r\n]*(?<!/P\d+)(?<!/S\d+))(/P(?<PASSENGERINDEX>\d+))?(/S(?<SEGMENTINDEX>\d+))?\s*$",
        )?;
        match re.captures(raw) {
            Some(caps) => match (
                caps.name("SERVICECODE"),
                caps.name("AIRLINE"),
                caps.name("ACTIONCODE"),
                caps.name("ACTIONCODEQTY"),
                caps.name("TEXT"),
                caps.name("PASSENGERINDEX"),
                caps.name("SEGMENTINDEX"),
            ) {
                (
                    Some(service_code),
                    Some(airline),
                    action_code,
                    action_code_qty,
                    Some(text),
                    passenger_index,
                    segment_index,
                ) => Ok(Self {
                    index,
                    raw: raw,
                    service_code: Some(service_code.as_str()),
                    airline: Some(airline.as_str()),
                    action_code: action_code.map(|x| x.as_str()),
                    action_code_qty: util::regex_extact_value::<u8>(action_code_qty),
                    text: Some(text.as_str()),
                    passenger_index: util::regex_extact_value::<u8>(passenger_index),
                    segment_index: util::regex_extact_value::<u8>(segment_index),
                }),
                _ => Ok(Self {
                    index,
                    raw: raw,
                    ..Default::default()
                }),
            },
            _ => Ok(Self {
                index,
                raw: raw,
                ..Default::default()
            }),
        }
    }
}

/// The osi infomation of pnr.
#[derive(Default, Debug)]
pub struct OSI<'a> {
    pub index: u8,
    pub raw: &'a str,
    pub service_code: Option<&'a str>,
    pub airline: Option<&'a str>,
    pub text: Option<&'a str>,
    pub passenger_index: Option<u8>,
}

impl<'a> OSI<'a> {
    pub fn parse(index: u8, raw: &'a str) -> anyhow::Result<Self> {
        let re = regex::Regex::new(
            r"OSI (?<AIRLINE>\w{2}) (?<SERVICECODE>[A-Z]+)?(?<TEXT>(.*(?<!/P\d+)))(/P(?<PASSENGERINDEX>\d+))?$",
        )?;
        match re.captures(raw) {
            Some(caps) => match (
                caps.name("AIRLINE"),
                caps.name("SERVICECODE"),
                caps.name("TEXT"),
                caps.name("PASSENGERINDEX"),
            ) {
                (Some(airline), Some(service_code), Some(text), passenger_index) => Ok(Self {
                    index,
                    raw: raw,
                    service_code: Some(service_code.as_str()),
                    airline: Some(airline.as_str()),
                    text: Some(text.as_str()),
                    passenger_index: util::regex_extact_value::<u8>(passenger_index), // passenger_index.and_then(|x|x.as_str().parse::<u8>().ok()),
                }),
                _ => Ok(Self {
                    index,
                    raw: raw,
                    ..Default::default()
                }),
            },
            _ => Ok(Self {
                index,
                raw: raw,
                ..Default::default()
            }),
        }
    }
}

/// The remark infomation of pnr.
#[derive(Default, Debug)]
pub struct RMK<'a> {
    pub index: u8,
    pub raw: &'a str,
    pub service_code: Option<&'a str>,
    pub text: Option<&'a str>,
    pub passenger_index: Option<u8>,
}

impl<'a> RMK<'a> {
    pub fn parse(index: u8, raw: &'a str) -> anyhow::Result<Self> {
        let re = regex::Regex::new(
            r"RMK[ :/](?<SERVICECODE>(MP|TJ AUTH|CA|CID|TID|EMAIL|1A|GMJC|RV|ORI))[ :/](?<TEXT>.*?)(/P(?<PASSENGERINDEX>\d))?$",
        )?;
        match re.captures(raw) {
            Some(caps) => match (
                caps.name("SERVICECODE"),
                caps.name("TEXT"),
                caps.name("PASSENGERINDEX"),
            ) {
                (Some(service_code), text, passenger_index) => Ok(Self {
                    index,
                    raw: raw,
                    service_code: Some(service_code.as_str()),
                    text: util::regex_extact_text(text),
                    passenger_index: util::regex_extact_value::<u8>(passenger_index),
                }),
                _ => Ok(Self {
                    index,
                    raw: raw,
                    ..Default::default()
                }),
            },
            _ => Ok(Self {
                index,
                raw: raw,
                ..Default::default()
            }),
        }
    }
}
