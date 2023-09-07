#[inline]
pub fn regex_extact<'a>(pattern: &str, text: &'a str) -> anyhow::Result<Option<&'a str>> {
    let re = regex::Regex::new(pattern)?;
    re.captures(&text).map_or(Ok(None), |caps| {
        caps.get(1)
            .map_or(Ok(None), |x| Ok(Some(x.as_str().trim())))
    })
}

#[inline]
pub fn regex_extact_text(cap: Option<regex::Match<'_>>) -> Option<&str> {
    cap.and_then(|x| Some(x.as_str().trim()))
    //cap.map_or(None, |x| Some(x.as_str().trim().to_owned()))
}

#[inline]
pub fn regex_extact_value<F: std::str::FromStr>(cap: Option<regex::Match<'_>>) -> Option<F> {
    cap.and_then(|x| x.as_str().parse::<F>().ok())
}