/// Mapping from common abbreviations and city names to IANA timezone identifiers
pub static TIMEZONE_ALIASES: &[(&str, &str)] = &[
    // Abbreviations
    ("UTC", "UTC"),
    ("GMT", "GMT"),
    ("EST", "US/Eastern"),
    ("EDT", "US/Eastern"),
    ("CST", "US/Central"),
    ("CDT", "US/Central"),
    ("MST", "US/Mountain"),
    ("MDT", "US/Mountain"),
    ("PST", "US/Pacific"),
    ("PDT", "US/Pacific"),
    ("CET", "Europe/Paris"),
    ("CEST", "Europe/Paris"),
    ("EET", "Europe/Helsinki"),
    ("EEST", "Europe/Helsinki"),
    ("WET", "Europe/Lisbon"),
    ("WEST", "Europe/Lisbon"),
    ("JST", "Asia/Tokyo"),
    ("KST", "Asia/Seoul"),
    ("CST_CN", "Asia/Shanghai"),
    ("IST", "Asia/Kolkata"),
    ("AEST", "Australia/Sydney"),
    ("AEDT", "Australia/Sydney"),
    ("ACST", "Australia/Adelaide"),
    ("AWST", "Australia/Perth"),
    ("NZST", "Pacific/Auckland"),
    ("NZDT", "Pacific/Auckland"),
    ("HST", "Pacific/Honolulu"),
    ("AKST", "America/Anchorage"),
    ("AST", "America/Halifax"),
    ("NST", "America/St_Johns"),
    ("BRT", "America/Sao_Paulo"),
    ("ART", "America/Argentina/Buenos_Aires"),
    ("SGT", "Asia/Singapore"),
    ("HKT", "Asia/Hong_Kong"),
    ("ICT", "Asia/Bangkok"),
    ("PKT", "Asia/Karachi"),
    ("GST", "Asia/Dubai"),
    ("MSK", "Europe/Moscow"),
    ("SAST", "Africa/Johannesburg"),
    ("WAT", "Africa/Lagos"),
    ("EAT", "Africa/Nairobi"),

    // City names
    ("New York", "America/New_York"),
    ("Los Angeles", "America/Los_Angeles"),
    ("Chicago", "America/Chicago"),
    ("Denver", "America/Denver"),
    ("Phoenix", "America/Phoenix"),
    ("Anchorage", "America/Anchorage"),
    ("Honolulu", "Pacific/Honolulu"),
    ("Toronto", "America/Toronto"),
    ("Vancouver", "America/Vancouver"),
    ("Mexico City", "America/Mexico_City"),
    ("Sao Paulo", "America/Sao_Paulo"),
    ("Buenos Aires", "America/Argentina/Buenos_Aires"),
    ("Santiago", "America/Santiago"),
    ("London", "Europe/London"),
    ("Paris", "Europe/Paris"),
    ("Berlin", "Europe/Berlin"),
    ("Madrid", "Europe/Madrid"),
    ("Rome", "Europe/Rome"),
    ("Amsterdam", "Europe/Amsterdam"),
    ("Brussels", "Europe/Brussels"),
    ("Vienna", "Europe/Vienna"),
    ("Zurich", "Europe/Zurich"),
    ("Stockholm", "Europe/Stockholm"),
    ("Oslo", "Europe/Oslo"),
    ("Copenhagen", "Europe/Copenhagen"),
    ("Helsinki", "Europe/Helsinki"),
    ("Warsaw", "Europe/Warsaw"),
    ("Prague", "Europe/Prague"),
    ("Budapest", "Europe/Budapest"),
    ("Bucharest", "Europe/Bucharest"),
    ("Athens", "Europe/Athens"),
    ("Istanbul", "Europe/Istanbul"),
    ("Moscow", "Europe/Moscow"),
    ("Dubai", "Asia/Dubai"),
    ("Mumbai", "Asia/Kolkata"),
    ("Delhi", "Asia/Kolkata"),
    ("Kolkata", "Asia/Kolkata"),
    ("Karachi", "Asia/Karachi"),
    ("Dhaka", "Asia/Dhaka"),
    ("Bangkok", "Asia/Bangkok"),
    ("Jakarta", "Asia/Jakarta"),
    ("Singapore", "Asia/Singapore"),
    ("Hong Kong", "Asia/Hong_Kong"),
    ("Shanghai", "Asia/Shanghai"),
    ("Beijing", "Asia/Shanghai"),
    ("Taipei", "Asia/Taipei"),
    ("Tokyo", "Asia/Tokyo"),
    ("Seoul", "Asia/Seoul"),
    ("Sydney", "Australia/Sydney"),
    ("Melbourne", "Australia/Melbourne"),
    ("Brisbane", "Australia/Brisbane"),
    ("Perth", "Australia/Perth"),
    ("Auckland", "Pacific/Auckland"),
    ("Johannesburg", "Africa/Johannesburg"),
    ("Cairo", "Africa/Cairo"),
    ("Lagos", "Africa/Lagos"),
    ("Nairobi", "Africa/Nairobi"),
    ("Casablanca", "Africa/Casablanca"),
];

/// Look up a timezone by abbreviation, city name, or IANA identifier.
/// Returns the IANA timezone string.
pub fn find_timezone(name: &str) -> Option<&'static str> {
    // Exact IANA match first (e.g., "America/New_York")
    if name.contains('/') {
        for &(_, iana) in TIMEZONE_ALIASES {
            if iana.eq_ignore_ascii_case(name) {
                return Some(iana);
            }
        }
        // Not in our alias table - return None and let the caller try parsing directly
        return None;
    }

    // Case-insensitive alias/city match
    for &(alias, iana) in TIMEZONE_ALIASES {
        if alias.eq_ignore_ascii_case(name) {
            return Some(iana);
        }
    }

    None
}
