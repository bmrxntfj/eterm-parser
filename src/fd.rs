#[derive(Default, Debug)]
pub struct FD {
    pub org: Option<String>,
    pub dst: Option<String>,
    pub query_time: Option<String>,
    pub airline: Option<String>,
    pub command: Option<String>,
    pub currency: Option<String>,
    pub tpm: Option<String>,
    pub items: Option<Vec<FDItem>>,
    pub raw_text: String,
}

impl FD {
    pub fn parse(text: &str) -> anyhow::Result<Self> {
        if text.is_empty() {
            return Err(anyhow::Error::msg(
                "fd parameter shouldn't be empty.".to_owned(),
            ));
        }
        let lines = text.split(&['\r', '\n']);

        let mut fdinfo = Self {
            raw_text: text.to_owned(),
            ..Default::default()
        };
        for line in lines {
            if line.starts_with(">") {
                continue;
            } else if line.starts_with("PAGE") {
                continue;
            } else if line.trim().is_empty() {
                continue;
            } else if line.starts_with("FD:") {
                match regex::Regex::captures(
                    &regex::Regex::new(
                        r"(FD(?<COMMAND>.*)?)\s+/(?<CURRENCY>[^/]*)/TPM\s*(?<TPM>\d+)?",
                    )?,
                    line,
                ) {
                    Some(caps) => match (
                        caps.name("COMMAND"),
                        caps.name("CURRENCY"),
                        caps.name("TPM"),
                    ) {
                        (Some(command), Some(currency), tpm) => {
                            fdinfo.command = Some(command.as_str().to_owned());
                            fdinfo.currency = Some(currency.as_str().to_owned());
                            fdinfo.tpm = crate::regex_extact_text(tpm);

                            match regex::Regex::captures(
                                &regex::Regex::new(
                                    r"(?<ORG>[A-Z]{3})(?<DST>[A-Z]{3})/(?<QUERYTIME>\d{2}\w{3}(?:\d{2})?)(?:/(?<AIRLINE>\w{2}))?",
                                )?,
                                command.as_str(),
                            ) {
                                Some(ccaps) => match (
                                    ccaps.name("ORG"),
                                    ccaps.name("DST"),
                                    ccaps.name("QUERYTIME"),
                                    ccaps.name("AIRLINE"),
                                ) {
                                    (Some(org), Some(dst), Some(querytime), airline) => {
                                        fdinfo.org = Some(org.as_str().to_owned());
                                        fdinfo.dst = Some(dst.as_str().to_owned());
                                        fdinfo.query_time = Some(querytime.as_str().to_owned());
                                        fdinfo.airline = crate::regex_extact_text(airline);
                                    }
                                    _ => {}
                                },
                                _ => {}
                            }
                        }
                        _ => {}
                    },
                    _ => {}
                }
            } else {
                let mut item = FDItem {
                    ..Default::default()
                };
                let mut arr = line.split('/');
                let index_airline = arr.next();
                item.index = index_airline.and_then(|x| x[0..3].parse::<u8>().ok());
                item.carrier = index_airline.and_then(|x| Some(x[3..5].trim().to_owned()));
                item.ticket_type = arr.next().and_then(|x| Some(x.trim().to_owned()));
                if let Some(ow_rt_price) = arr.next() {
                    let mut ow_rt_price = ow_rt_price.split('=');
                    item.ow_price_raw = ow_rt_price.next().and_then(|x| Some(x.trim().to_owned()));
                    item.rt_price_raw = ow_rt_price.next().and_then(|x| Some(x.trim().to_owned()));
                }
                item.cabin = arr.next().and_then(|x| Some(x.trim().to_owned()));
                item.class = arr.next().and_then(|x| Some(x.trim().to_owned()));
                arr.next();
                item.begin_date = arr.next().and_then(|x| {
                    if x == "." {
                        fdinfo.query_time.to_owned()
                    } else {
                        Some(x.to_owned())
                    }
                });
                item.end_date = arr.next().and_then(|x| Some(x.trim().to_owned()));
                item.policy_no = arr.next().and_then(|x| Some(x[0..6].trim().to_owned()));

                if item.index == Some(0u8) {
                    continue;
                }
                match &item.ow_price_raw {
                    Some(ow_price_raw) => {
                        if ow_price_raw.contains('%') {
                            if let Some(yfd) = fdinfo.items.as_ref().and_then(|x| {
                                x.iter().find_map(|n| {
                                    if n.carrier == item.carrier
                                        && n.cabin == Some("Y".to_owned())
                                        && n.ow_price.is_some()
                                    {
                                        n.ow_price
                                    } else {
                                        None
                                    }
                                })
                            }) {
                                item.ow_price = Some(
                                    yfd * ow_price_raw
                                        .trim_end_matches('%')
                                        .parse::<f32>()
                                        .unwrap()
                                        / 100.0,
                                );
                            }
                        } else {
                            item.ow_price = ow_price_raw.parse::<f32>().ok();
                        }
                    }
                    _ => {}
                }
                match &item.rt_price_raw {
                    Some(rt_price_raw) => {
                        if rt_price_raw.contains('%') {
                            if let Some(yfd) = fdinfo.items.as_ref().and_then(|x| {
                                x.iter().find_map(|n| {
                                    if n.carrier == item.carrier
                                        && n.cabin == Some("Y".to_owned())
                                        && n.rt_price.is_some()
                                    {
                                        n.rt_price
                                    } else {
                                        None
                                    }
                                })
                            }) {
                                item.rt_price = Some(
                                    yfd * rt_price_raw
                                        .trim_end_matches('%')
                                        .parse::<f32>()
                                        .unwrap()
                                        / 100.0,
                                );
                            }
                        } else {
                            item.rt_price = rt_price_raw.parse::<f32>().ok();
                        }
                    }
                    _ => {}
                }
                fdinfo
                    .items
                    .as_mut()
                    .get_or_insert(&mut Vec::new())
                    .push(item);
            }
        }

        Ok(fdinfo)
    }
}

#[derive(Default, Debug)]
pub struct FDItem {
    pub index: Option<u8>,
    pub carrier: Option<String>,
    pub ticket_type: Option<String>,
    pub ow_price_raw: Option<String>,
    pub ow_price: Option<f32>,
    pub rt_price_raw: Option<String>,
    pub rt_price: Option<f32>,
    pub cabin: Option<String>,
    pub class: Option<String>,
    pub begin_date: Option<String>,
    pub end_date: Option<String>,
    pub policy_no: Option<String>,
}
