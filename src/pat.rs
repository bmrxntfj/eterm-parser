#[derive(Default, Debug)]
pub struct PAT {
    pub raw_text: String,
    pub items: Option<Vec<PATItem>>,
}

impl PAT {
    pub fn parse(text: &str) -> anyhow::Result<Self> {
        if text.is_empty() {
            return Err(anyhow::Error::msg(
                "pnr parameter shouldn't be empty.".to_owned(),
            ));
        }
        let re = regex::Regex::new(
            r"(?<INDEX>\d+)\s+(?<SEATCLASS>[/\w+]+)\s+FARE:(?<FARE>[\w.]+)\s+TAX:(?<TAX>[\w.]+)\s+YQ:(?<YQ>[\w.]+)\s+TOTAL:(?<TOTAL>[\w.]+)",
        )?;

        let pat = Self {
            raw_text: text.to_owned(),
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
                            (index, seatclass, fare, tax, yq, total) => Some(PATItem {
                                index: crate::regex_extact_value::<u8>(index),
                                seat_class: crate::regex_extact_text(seatclass),
                                fare: fare.and_then(|x| PATPrice::parse(x.as_str()).ok()),
                                tax: tax.and_then(|x| PATPrice::parse(x.as_str()).ok()),
                                yq: yq.and_then(|x| PATPrice::parse(x.as_str()).ok()),
                                total: crate::regex_extact_value::<f32>(total),
                                raw_text: caps.get(0).and_then(|x| Some(x.as_str().to_owned())),
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
pub struct PATItem {
    pub index: Option<u8>,
    pub seat_class: Option<String>,
    pub fare: Option<PATPrice>,
    pub tax: Option<PATPrice>,
    pub yq: Option<PATPrice>,
    pub total: Option<f32>,
    pub raw_text: Option<String>,
}

#[derive(Default, Debug)]
pub struct PATPrice {
    pub currency: Option<String>,
    pub price: Option<f32>,
    pub is_exemption: Option<bool>,
}

impl PATPrice {
    pub fn parse(text: &str) -> anyhow::Result<Self> {
        let mut pat_price = Self {
            ..Default::default()
        };
        let tx = text.trim();
        if tx.to_uppercase().starts_with("TEXEMPT") {
            pat_price.price = Some(0f32);
            pat_price.is_exemption = Some(true);
        } else {
            pat_price.currency = Some(tx[0..3].to_owned());
            pat_price.price = tx[3..].parse::<f32>().ok();
        }
        Ok(pat_price)
    }
}
