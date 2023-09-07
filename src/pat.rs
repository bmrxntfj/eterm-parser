use crate::util;

#[derive(Default, Debug)]
pub struct Pat<'a> {
    pub raw_text: &'a str,
    pub items: Option<Vec<PatItem<'a>>>,
}

impl<'a> Pat<'a> {
    pub fn parse(text: &'a str) -> anyhow::Result<Self> {
        if text.is_empty() {
            return Err(anyhow::Error::msg(
                "pnr parameter shouldn't be empty.".to_owned(),
            ));
        }
        let re = regex::Regex::new(
            r"(?<INDEX>\d+)\s+(?<SEATCLASS>[/\w+]+)\s+FARE:(?<FARE>[\w.]+)\s+TAX:(?<TAX>[\w.]+)\s+YQ:(?<YQ>[\w.]+)\s+TOTAL:(?<TOTAL>[\w.]+)",
        )?;

        let pat = Self {
            raw_text: text,
            items: Some(
                re.captures_iter(text)
                    .filter_map(|caps| {
                        match (
                            caps.name("INDEX"),
                            caps.name("SEATCLASS"),
                            caps.name("FARE"),
                            caps.name("TAX"),
                            caps.name("YQ"),
                            caps.name("TOTAL"),
                        ) {
                            (index, seatclass, fare, tax, yq, total) => Some(PatItem {
                                index: util::regex_extact_value::<u8>(index),
                                seat_class: seatclass.and_then(|x| Some(x.as_str().trim())), // crate::regex_extact_text(seatclass),
                                fare: fare.and_then(|x| PatPrice::parse(x.as_str()).ok()),
                                tax: tax.and_then(|x| PatPrice::parse(x.as_str()).ok()),
                                yq: yq.and_then(|x| PatPrice::parse(x.as_str()).ok()),
                                total: util::regex_extact_value::<f32>(total),
                                raw_text: caps.get(0).and_then(|x| Some(x.as_str())),
                            }),
                        }
                    })
                    .collect::<Vec<_>>(),
            ),
        };
        return Ok(pat);
    }
}

#[derive(Default, Debug)]
pub struct PatItem<'a> {
    pub index: Option<u8>,
    pub seat_class: Option<&'a str>,
    pub fare: Option<PatPrice<'a>>,
    pub tax: Option<PatPrice<'a>>,
    pub yq: Option<PatPrice<'a>>,
    pub total: Option<f32>,
    pub raw_text: Option<&'a str>,
}

#[derive(Default, Debug, PartialEq)]
pub struct PatPrice<'a> {
    pub currency: Option<&'a str>,
    pub price: Option<f32>,
    pub is_exemption: bool,
}

impl<'a> PatPrice<'a> {
    pub fn parse(text: &'a str) -> anyhow::Result<Self> {
        let mut pat_price = Self {
            ..Default::default()
        };
        let tx = text.trim();
        if tx.to_uppercase().starts_with("TEXEMPT") {
            pat_price.price = Some(0f32);
            pat_price.is_exemption = true;
        } else {
            pat_price.currency = Some(&tx[0..3]);
            pat_price.price = tx[3..].parse::<f32>().ok();
            pat_price.is_exemption = false;
        }
        Ok(pat_price)
    }
}
