/// The result that pnr text parsed.
#[derive(Default, Debug)]
pub struct PNR {
    pub infos: Option<Vec<String>>,
    pub ssr_items: Option<Vec<SSR>>,
    pub osi_items: Option<Vec<OSI>>,
    pub seg_items: Option<Vec<SEG>>,
    pub nm_items: Option<Vec<NM>>,
    pub rmk_items: Option<Vec<RMK>>,
    pub other_items: Option<Vec<PnrItem>>,
    pub is_ticket_pnr: Option<bool>,
    pub is_cancelled_pnr: Option<bool>,
    pub is_group_pnr: Option<bool>,
    pub group_pnr_name: Option<String>,
    pub pax_count: Option<u8>,
    pub pnr_code: Option<String>,
    //pub raw_text: String,
    pub bpnr_code: Option<String>,
    pub has_married_segment: Option<bool>,
    pub office_no: Option<String>,
}

impl PNR {
    pub fn parse(text: &str) -> anyhow::Result<Self> {
        if text.is_empty() {
            return Err(anyhow::Error::msg(
                "pnr parameter shouldn't be empty.".to_owned(),
            ));
        }
        let mut is_ticket_pnr: Option<bool> = None;
        let mut is_cancelled_pnr: Option<bool> = None;
        let mut has_married_segment: Option<bool> = None;
        let mut info_parsed = false;
        let mut is_group_pnr: Option<bool> = None;
        let mut group_pnr_name: Option<String> = None;
        let mut pnr_code: Option<String> = None;
        let mut pax_count: Option<u8> = None;
        let mut infos: Option<Vec<String>> = None;
        let mut seg_items: Option<Vec<SEG>> = None;
        let mut nm_items: Option<Vec<NM>> = None;
        let mut ssr_items: Option<Vec<SSR>> = None;
        let mut osi_items: Option<Vec<OSI>> = None;
        let mut rmk_items: Option<Vec<RMK>> = None;
        let mut other_items: Option<Vec<PnrItem>> = None;
        let mut index = 0u8;
        for line in text.lines() {
            
            if !info_parsed && line.starts_with(" 1.") {
                info_parsed = true;
            }
            if !info_parsed {
                if line.contains("**ELECTRONIC TICKET PNR**") {
                    is_ticket_pnr = Some(true);
                }
                if line.contains("*THIS PNR WAS ENTIRELY CANCELLED*") {
                    is_cancelled_pnr = Some(true);
                }
                if line.contains("MARRIED SEGMENT EXIST IN THE PNR") {
                    has_married_segment = Some(true);
                }
                infos.get_or_insert(Vec::new()).push(line.to_owned());
            } else {
                match line.trim_start() {
                    x if x.starts_with("1.") => {
                        let re = regex::Regex::new(r"(?<NMS>1\.(.*))(?<PNRCODE>\w{6})\s*$")?;
                        match re.captures(line) {
                            Some(caps) => match (caps.name("NMS"), caps.name("PNRCODE")) {
                                (Some(nms), Some(pnrcode)) => {
                                    pnr_code = Some(pnrcode.as_str().to_owned());
                                    if let Ok(items) = NM::parse(index, nms.as_str()) {
                                        nm_items = Some(items);
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
                        is_group_pnr = Some(true);
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
                                    group_pnr_name = Some(pnr_groupname.as_str().to_owned());
                                    pnr_code = Some(pnrcode.as_str().to_owned());
                                    pax_count = paxcount.as_str().parse::<u8>().ok();
                                }
                                _ => {}
                            }
                        }
                    }
                    x if x.starts_with(&format!("{}. ", index)) => {
                        if let Ok(item) = SEG::parse(index, line) {
                            seg_items.get_or_insert(Vec::new()).push(item);
                        }
                    }
                    x if x.starts_with(&format!("{}SSR", index)) => {
                        if let Ok(item) = SSR::parse(index, line) {
                            ssr_items.get_or_insert(Vec::new()).push(item);
                        }
                    }
                    x if x.starts_with(&format!("{}OSI", index)) => {
                        if let Ok(item) = OSI::parse(index, line) {
                            osi_items.get_or_insert(Vec::new()).push(item);
                        }
                    }
                    x if x.starts_with(&format!("{}RMK", index)) => {
                        if let Ok(item) = RMK::parse(index, line) {
                            rmk_items.get_or_insert(Vec::new()).push(item);
                        }
                    }
                    _ => {
                        if let Ok(item) = PnrItem::parse(index, line) {
                            other_items.get_or_insert(Vec::new()).push(item);
                        }
                    }
                }
            }
            index += 1;
        }
        Self::fix_nm(&ssr_items, &mut nm_items);
        if pax_count.is_none() {
            pax_count = nm_items.as_ref().and_then(|x| Some(x.len() as u8));
        }
        let office_no = other_items.as_ref().and_then(|x| {
            x.iter().find_map(|n| {
                if &n.item_type == "OFFICE" {
                    Some(n.raw.trim().to_owned())
                } else {
                    None
                }
            })
        });
        let bpnr_code = rmk_items.as_ref().and_then(|x| {
            x.iter().find_map(|n| {
                if n.service_code.as_ref().is_some_and(|s| s == "CA") {
                    n.text.as_ref().and_then(|k| Some(k[0..6].to_owned()))
                } else {
                    None
                }
            })
        });
        Ok(Self {
            //raw_text,
            infos,
            is_ticket_pnr,
            is_cancelled_pnr,
            has_married_segment,
            is_group_pnr,
            group_pnr_name,
            pnr_code,
            seg_items,
            nm_items,
            ssr_items,
            osi_items,
            rmk_items,
            other_items,
            bpnr_code,
            office_no,
            pax_count,
            ..Default::default()
        })
    }

    /// fill id info with ssr.
    fn fix_nm(ssr_items: &Option<Vec<SSR>>, nm_items: &mut Option<Vec<NM>>) {
        match (ssr_items, nm_items) {
            (Some(ssrs), Some(nms)) => {
                nms.iter_mut().for_each(|x| {
                    if let Some(ssr) = ssrs.iter().find(|s| {
                        s.passenger_index.is_some_and(|n| n == x.index)
                            && s.service_code.as_ref().is_some_and(|n| n == "FOID")
                    }) {
                        if let Some(tx) = &ssr.text {
                            x.id_type = Some(tx[0..2].to_owned());
                            x.id_number = Some(tx[2..].to_owned());
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
pub struct PnrItem {
    pub index: u8,
    pub item_type: String,
    pub raw: String,
}

impl PnrItem {
    pub fn parse(index: u8, raw: &str) -> anyhow::Result<Self> {
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
            item_type: item_type.to_owned(),
            raw: raw.to_owned(),
        })
    }
}

/// The passenger infomation of pnr.
#[derive(Default, Debug)]
pub struct NM {
    pub index: u8,
    //item_type:Option<String>,
    pub raw: String,
    pub name: Option<String>,
    //pub ssr_items: Option<Vec<SSR>>,
    //pub osi_items: Option<Vec<OSI>>,
    pub id_number: Option<String>, //s.ServiceCode == "FOID")?.Text?.Substring(2);
    pub id_type: Option<String>,   //s.ServiceCode == "FOID")?.Text?.Substring(0, 2);
}

impl NM {
    pub fn parse(index: u8, raw: &str) -> anyhow::Result<Vec<Self>> {
        let re = regex::Regex::new(r"(\d+\.)")?;
        let nms = re
            .split(raw)
            .filter_map(|cap| match cap.trim() {
                x if x.is_empty() => None,
                x if x.ends_with(".") => None,
                x => Some(Self {
                    index,
                    raw: cap.trim().to_owned(),
                    name: Some(x.to_owned()),
                    ..Default::default()
                }),
            })
            .collect::<Vec<_>>();
        Ok(nms)
    }
}

/// The flight segment infomation of pnr.
#[derive(Default, Debug)]
pub struct SEG {
    pub index: u8,
    pub raw: String,
    pub org: Option<String>,
    pub dst: Option<String>,
    pub seat_class: Option<String>,
    pub flight_date: Option<String>,
    pub takeoff: Option<String>,
    pub landing: Option<String>,
    pub landing_addday: Option<u8>,
    pub action_code: Option<String>,
    pub action_code_qty: Option<u8>,
    pub flight_no: Option<String>,
    pub is_share: Option<bool>,
}

impl SEG {
    pub fn parse(index: u8, raw: &str) -> anyhow::Result<Self> {
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
                    raw: raw.to_owned(),
                    flight_no: Some(flight_no.as_str().to_owned()),
                    seat_class: Some(seat_class.as_str().to_owned()),
                    flight_date: Some(flight_date.as_str().to_owned()),
                    org: Some(org.as_str().to_owned()),
                    dst: Some(dst.as_str().to_owned()),
                    action_code: Some(action_code.as_str().to_owned()),
                    action_code_qty: action_code_qty.as_str().parse::<u8>().ok(), // action_code_qty.and_then(|x|x.as_str().parse::<u8>().ok()),
                    takeoff: Some(takeoff.as_str().to_owned()),
                    landing: Some(landing.as_str().to_owned()),
                    landing_addday: crate::regex_extact_value::<u8>(addday), // passenger_index.and_then(|x|x.as_str().parse::<u8>().ok()),
                    is_share: Some(flight_no.as_str().starts_with("*")),
                }),
                _ => Ok(Self {
                    index,
                    raw: raw.to_owned(),
                    ..Default::default()
                }),
            },
            _ => Ok(Self {
                index,
                raw: raw.to_owned(),
                ..Default::default()
            }),
        }
    }
}

/// The ssr infomation of pnr.
#[derive(Default, Debug)]
pub struct SSR {
    pub index: u8,
    pub raw: String,
    pub service_code: Option<String>,
    pub action_code: Option<String>,
    pub action_code_qty: Option<u8>,
    pub airline: Option<String>,
    pub text: Option<String>,
    pub passenger_index: Option<u8>,
    pub segment_index: Option<u8>,
}

impl SSR {
    pub fn parse(index: u8, raw: &str) -> anyhow::Result<Self> {
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
                    raw: raw.to_owned(),
                    service_code: Some(service_code.as_str().to_owned()),
                    airline: Some(airline.as_str().to_owned()),
                    action_code: action_code.map(|x| x.as_str().to_owned()),
                    action_code_qty: crate::regex_extact_value::<u8>(action_code_qty),
                    text: Some(text.as_str().to_owned()),
                    passenger_index: crate::regex_extact_value::<u8>(passenger_index),
                    segment_index: crate::regex_extact_value::<u8>(segment_index),
                }),
                _ => Ok(Self {
                    index,
                    raw: raw.to_owned(),
                    ..Default::default()
                }),
            },
            _ => Ok(Self {
                index,
                raw: raw.to_owned(),
                ..Default::default()
            }),
        }
    }
}

/// The osi infomation of pnr.
#[derive(Default, Debug)]
pub struct OSI {
    pub index: u8,
    pub raw: String,
    pub service_code: Option<String>,
    pub airline: Option<String>,
    pub text: Option<String>,
    pub passenger_index: Option<u8>,
}

impl OSI {
    pub fn parse(index: u8, raw: &str) -> anyhow::Result<Self> {
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
                    raw: raw.to_owned(),
                    service_code: Some(service_code.as_str().to_owned()),
                    airline: Some(airline.as_str().to_owned()),
                    text: Some(text.as_str().to_owned()),
                    passenger_index: crate::regex_extact_value::<u8>(passenger_index), // passenger_index.and_then(|x|x.as_str().parse::<u8>().ok()),
                }),
                _ => Ok(Self {
                    index,
                    raw: raw.to_owned(),
                    ..Default::default()
                }),
            },
            _ => Ok(Self {
                index,
                raw: raw.to_owned(),
                ..Default::default()
            }),
        }
    }
}

/// The remark infomation of pnr.
#[derive(Default, Debug)]
pub struct RMK {
    pub index: u8,
    pub raw: String,
    pub service_code: Option<String>,
    pub text: Option<String>,
    pub passenger_index: Option<u8>,
}

impl RMK {
    pub fn parse(index: u8, raw: &str) -> anyhow::Result<Self> {
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
                    raw: raw.to_owned(),
                    service_code: Some(service_code.as_str().to_owned()),
                    text: crate::regex_extact_text(text),
                    passenger_index: crate::regex_extact_value::<u8>(passenger_index), // passenger_index.and_then(|x|x.as_str().parse::<u8>().ok()),
                }),
                _ => Ok(Self {
                    index,
                    raw: raw.to_owned(),
                    ..Default::default()
                }),
            },
            _ => Ok(Self {
                index,
                raw: raw.to_owned(),
                ..Default::default()
            }),
        }
    }
}
