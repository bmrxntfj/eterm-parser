use crate::util;
use std::collections::HashMap;

/// The result that detr text parsed.
#[derive(Default, Debug)]
pub struct Detr<'a> {
    /// airline issued by.
    pub issued_by: Option<&'a str>,
    /// airport of departure
    pub org: Option<&'a str>,
    /// airport of arrival
    pub dst: Option<&'a str>,
    /// sale type of ticket,such as BSP-D,BSP-I,ARL-D,ARL-I.
    pub et: Option<&'a str>,
    /// remark/limit
    pub er: Option<&'a str>,
    /// code of tour
    pub tour_code: Option<&'a str>,
    /// show that whether the itinerary receipt has printed.
    pub receipt_printed: bool,
    /// the name of passenger.
    pub passenger: Option<&'a str>,
    /// proof of exchange.
    pub exchange: Option<&'a str>,
    /// conjoint ticket.
    pub conj_tickets: Option<&'a str>,
    /// the segments of flight.
    pub flight_segs: Vec<DetrFlightSeg<'a>>,
    /// fare of FC.
    pub fc: Option<&'a str>,
    /// face value of ticket.
    pub fare: Option<DetrFareItem<'a>>,
    /// tax value of ticket.
    pub taxs: Option<HashMap<Option<&'a str>, DetrFareItem<'a>>>,
    /// total value of ticket.
    pub total: Option<DetrFareItem<'a>>,
    /// pay method.
    pub fop: Option<&'a str>,
    pub oi: Option<&'a str>,
    /// ticket number.
    pub tktn: Option<&'a str>,
}

#[derive(Default, Debug, PartialEq)]
pub struct DetrFareItem<'a> {
    pub item_type: Option<&'a str>,
    pub amount: Option<f32>,
    pub currency: Option<&'a str>,
    pub is_exempt: bool,
}

#[derive(Default, Debug)]
pub struct DetrFlightSeg<'a> {
    /// the flag that show how long to transit. such as O is more than 24 hours,X is less than 24 hours.
    pub transit_flag: Option<&'a str>,
    /// index of segment.
    pub index: Option<i32>,
    /// airport of departure
    pub org: Option<&'a str>,
    /// airport of arrival
    pub dst: Option<&'a str>,
    /// airline code
    pub airline: Option<&'a str>,
    /// the code of operator airline
    pub carrier: Option<&'a str>,
    /// the number of flight.
    pub flight_no: Option<&'a str>,
    /// departure date
    pub flight_deptdate: Option<&'a str>,
    /// arrival time
    pub flight_depttime: Option<&'a str>,
    /// the cabin of flight.
    pub flight_class: Option<&'a str>,
    /// the status of seat, such as
    /// OK represent reserved (RR or HK),
    /// RQ represent candidate,
    /// NS represent no seat(like baby),
    /// SA represent free seat(like free),
    pub seat_status: Option<&'a str>,
    /// the fare price of basis.
    pub fare_basis: Option<&'a str>,
    /// valid date.
    pub nvb: Option<&'a str>,
    /// valid date.
    pub nva: Option<&'a str>,
    /// baggage, such as
    /// K:km
    /// PC:piece
    /// NIL:no free luggage
    pub baggage: Option<&'a str>,
    /// the status of ticket, such as
    /// "OPEN FOR USE" represent unused,
    /// USED/FLOWN,
    /// VOID,
    /// REFUNDED,
    /// CHECK/IN,
    /// LIFT/BOARDED,
    /// SUSPENDED,
    /// EXCHANGED,
    /// AIRPORT CNTL,
    /// CPN NOTE,
    /// FIM EXCH.
    pub ticket_status: Option<&'a str>,
    /// the terminal of departure
    pub org_term: Option<&'a str>,
    /// the terminal of arrival
    pub dst_term: Option<&'a str>,
    /// big pnr
    pub bpnr: Option<&'a str>,
    /// pnr
    pub cpnr: Option<&'a str>,
    pub system: Option<&'a str>,
    /// the status of passenger, such as
    /// OFLK represent checkin and unboarded,
    /// NOSH represent miss flight.
    pub passenger_status_flag: Option<&'a str>,
}

impl<'a> Detr<'a> {
    pub fn parse(text: &'a str) -> anyhow::Result<Self> {
        if text.is_empty() {
            return Err(anyhow::Error::msg(
                "detr parameter shouldn't be empty.".to_owned(),
            ));
        }
        //let finalDest = Self::regex_extact(r"\s+TO: ([A-Z]{3})\s", &text)?;
        let re = regex::Regex::new(
            r"(?<TRANSITFLAG>[O|X]) (FM|TO):(?<INDEX>\d)(?<ORG>[A-Z]{3}) (?<AIRLINE>\w{2}) (?<CARRIER>..{2}) *(?<FLIGHTNO>\d+|OPEN)\s+(?<CABIN>[A-Z]) (?<DETPDATE>\d{2}[A-Z]{3}|OPEN ) (?<DEPTTIME>.{4}) (?<SEATSTATUS>.{2}) (?<FAREBASIS>.{10}) (?<NVB>.{6}).(?<NVA>.{6}) (?<BAGGAGE>.{3}) (?<TICKETSTATUS>[^\r\n]+)(\r|\n)+.....(?<ORGTERMINAL>..)(?<DSTTERMINAL>..) RL:(?:(?<BPNR>\w{6})(\s+)/((?<CPNR>\w{6})(?<SYSTEM>\w{2}))?)?(\s*[\r|\n]+\s+)TO:\s+(?<DST>[A-Z]{3})",
        )?;
        let detr_flight_segs = re
            .captures_iter(text)
            .filter_map(|caps| {
                match (
                    caps.name("TRANSITFLAG"),
                    caps.name("INDEX"),
                    caps.name("ORG"),
                    caps.name("AIRLINE"),
                    caps.name("CARRIER"),
                    caps.name("FLIGHTNO"),
                    caps.name("CABIN"),
                    caps.name("DETPDATE"),
                    caps.name("DEPTTIME"),
                    caps.name("SEATSTATUS"),
                    caps.name("FAREBASIS"),
                    caps.name("NVB"),
                    caps.name("NVA"),
                    caps.name("BAGGAGE"),
                    caps.name("TICKETSTATUS"),
                    caps.name("ORGTERMINAL"),
                    caps.name("DSTTERMINAL"),
                    caps.name("BPNR"),
                    caps.name("CPNR"),
                    caps.name("SYSTEM"),
                    caps.name("DST"),
                ) {
                    (
                        Some(cap_transit),
                        Some(cap_index),
                        cap_org,
                        cap_airline,
                        cap_carrier,
                        cap_flightno,
                        cap_cabin,
                        cap_deptdate,
                        cap_depttime,
                        cap_seatstatus,
                        cap_farebasis,
                        cap_nvb,
                        cap_nva,
                        cap_baggage,
                        cap_ticketstatus,
                        cap_orgterminal,
                        cap_dstterminal,
                        cap_bpnr,
                        cap_cpnr,
                        cap_system,
                        cap_dst,
                    ) => {
                        if cap_transit.as_str().contains("VOID") {
                            Some(DetrFlightSeg {
                                transit_flag: Some(cap_transit.as_str()),
                                index: cap_index.as_str().parse::<i32>().ok(),
                                org: util::regex_extact_text(cap_org),
                                seat_status: Some("VOID"),
                                ticket_status: Some("VOID"),
                                ..Default::default()
                            })
                        } else {
                            Some(DetrFlightSeg {
                                transit_flag: Some(cap_transit.as_str()),
                                index: cap_index.as_str().parse::<i32>().ok(),
                                org: util::regex_extact_text(cap_org),
                                airline: util::regex_extact_text(cap_airline),
                                carrier: util::regex_extact_text(cap_carrier),
                                flight_no: util::regex_extact_text(cap_flightno),
                                flight_class: util::regex_extact_text(cap_cabin),
                                flight_deptdate: util::regex_extact_text(cap_deptdate),
                                flight_depttime: util::regex_extact_text(cap_depttime),
                                seat_status: util::regex_extact_text(cap_seatstatus),
                                fare_basis: util::regex_extact_text(cap_farebasis),
                                nvb: util::regex_extact_text(cap_nvb),
                                nva: util::regex_extact_text(cap_nva),
                                baggage: util::regex_extact_text(cap_baggage),
                                ticket_status: util::regex_extact_text(cap_ticketstatus),
                                org_term: util::regex_extact_text(cap_orgterminal),
                                dst_term: util::regex_extact_text(cap_dstterminal),
                                bpnr: util::regex_extact_text(cap_bpnr),
                                cpnr: util::regex_extact_text(cap_cpnr),
                                system: util::regex_extact_text(cap_system),
                                dst: util::regex_extact_text(cap_dst),
                                ..Default::default()
                            })
                        }
                    }
                    _ => None,
                }
            })
            .collect::<Vec<_>>();

        let re = regex::Regex::new(r"FARE:\s+(?<CURRENCY>[A-Z]{3})\s*(?<AMOUNT>\d+\.\d{2})\|")?;
        let fare = match re.captures(text) {
            Some(caps) => match (caps.name("CURRENCY"), caps.name("AMOUNT")) {
                (Some(cap2), Some(cap3)) => Some(DetrFareItem {
                    amount: cap3.as_str().parse::<f32>().ok(),
                    currency: Some(cap2.as_str()),
                    is_exempt: false,
                    ..Default::default()
                }),
                _ => None,
            },
            _ => None,
        };

        let re = regex::Regex::new(r"TOTAL:\s+(?<CURRENCY>[A-Z]{3})\s*(?<AMOUNT>\d+\.\d{2})\|")?;
        let total = match re.captures(text) {
            Some(caps) => match (caps.name("CURRENCY"), caps.name("AMOUNT")) {
                (Some(cap2), Some(cap3)) => Some(DetrFareItem {
                    amount: cap3.as_str().parse::<f32>().ok(),
                    currency: Some(cap2.as_str()),
                    is_exempt: false,
                    ..Default::default()
                }),
                _ => None,
            },
            _ => None,
        };

        let re = regex::Regex::new(
            r"TAX:\s+(?:(?<EXEMPT>EXEMPT)|(?<CURRENCY>[A-Z]{3})\s*(?<PRICE>\d+\.\d{2}))(?<TYPE>[A-Z]{2})\|",
        )?;

        let items = re
            .captures_iter(text)
            .filter_map(|caps| {
                match (
                    caps.name("TYPE"),
                    caps.name("EXEMPT"),
                    caps.name("CURRENCY"),
                    caps.name("PRICE"),
                ) {
                    (Some(cap_type), cap_exempt, cap_curr, cap_price) => Some(DetrFareItem {
                        amount: cap_price.map_or(None, |x| x.as_str().parse::<f32>().ok()),
                        currency: util::regex_extact_text(cap_curr),
                        is_exempt: cap_exempt.map_or(false, |x| x.as_str() == "EXEMPT"),
                        item_type: Some(cap_type.as_str()),
                    }),
                    _ => None,
                }
            })
            .collect::<Vec<_>>();
        let taxs = if items.len() == 0 {
            None
        } else {
            let mut map = HashMap::new();
            for item in items {
                map.insert(item.item_type, item);
            }
            Some(map)
        };

        Ok(Self {
            issued_by: util::regex_extact(r"\bISSUED BY: ?(.*)ORG/DST:", text)?,
            org: util::regex_extact(r"ORG/DST: ?([A-Z]{3})/[A-Z]{3}", text)?,
            dst: util::regex_extact(r"ORG/DST: ?[A-Z]{3}/([A-Z]{3})", text)?,
            et: util::regex_extact(r"ORG/DST: ?[A-Z]{3}/[A-Z]{3}\s+([A-Z\-]+)", text)?,
            er: util::regex_extact(r"E/R: ?(.*?)(\r|\n)+", text)?,
            tour_code: util::regex_extact(r"TOUR CODE: ?(\S[^\r]*?)(\r|\n)+", text)?,
            receipt_printed: text.contains("RECEIPT PRINTED"),
            passenger: util::regex_extact(r"PASSENGER: ?(\S[^\r]*?)(\r|\n)+", text)?,
            exchange: util::regex_extact(r"EXCH: ?(\S[^\r]*?)\S", text)?,
            conj_tickets: util::regex_extact(r"CONJ TKT: ?(\S[^\r]*?)(\r|\n)+", text)?,
            flight_segs: detr_flight_segs,
            fc: util::regex_extact(r"FC: ?(\S[^\r]*?)(\r|\n)+", text)?,
            fare,
            taxs,
            total,
            fop: util::regex_extact(r"\|FOP: ?(\S[^\r]*?)(\r|\n)+", text)?,
            oi: util::regex_extact(r"\|OI: ?(\S[^\r]*?)(\r|\n)+", text)?,
            tktn: util::regex_extact(r"\|TKTN: ?(\S[^\r]*?)(\r|\n)+", text)?,
        })
    }
}
