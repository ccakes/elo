/// Unit dimension families
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Dimension {
    Length,
    Area,
    Volume,
    Mass,
    Time,
    Temperature,
    Angle,
    Data,
    Css,
    Speed,
    Dimensionless,
}

/// A unit definition with conversion factor to the base unit in its dimension.
/// For temperature, special handling is needed (offset conversions).
#[derive(Debug, Clone)]
pub struct UnitDef {
    pub id: &'static str,
    pub names: &'static [&'static str],
    pub dimension: Dimension,
    /// Multiplier to convert TO the base unit: base = value * factor + offset
    pub factor: f64,
    /// Offset for temperature conversions
    pub offset: f64,
}

impl UnitDef {
    pub const fn linear(
        id: &'static str,
        names: &'static [&'static str],
        dimension: Dimension,
        factor: f64,
    ) -> Self {
        Self {
            id,
            names,
            dimension,
            factor,
            offset: 0.0,
        }
    }

    pub const fn with_offset(
        id: &'static str,
        names: &'static [&'static str],
        dimension: Dimension,
        factor: f64,
        offset: f64,
    ) -> Self {
        Self {
            id,
            names,
            dimension,
            factor,
            offset,
        }
    }

    /// Convert a value in this unit to the base unit
    pub fn to_base(&self, value: f64) -> f64 {
        value * self.factor + self.offset
    }

    /// Convert a value from the base unit to this unit
    pub fn from_base(&self, value: f64) -> f64 {
        (value - self.offset) / self.factor
    }
}

// Base units per dimension:
// Length: meter
// Area: square meter
// Volume: liter
// Mass: kilogram
// Time: second
// Temperature: kelvin
// Angle: radian
// Data: byte
// CSS: pixel (base reference)
// Speed: meter/second

pub static UNITS: &[UnitDef] = &[
    // === Length (base: meter) ===
    UnitDef::linear("mm", &["mm", "millimeter", "millimeters", "millimetre", "millimetres"], Dimension::Length, 0.001),
    UnitDef::linear("cm", &["cm", "centimeter", "centimeters", "centimetre", "centimetres"], Dimension::Length, 0.01),
    UnitDef::linear("dm", &["dm", "decimeter", "decimeters"], Dimension::Length, 0.1),
    UnitDef::linear("m", &["meter", "meters", "metre", "metres"], Dimension::Length, 1.0),
    UnitDef::linear("km", &["km", "kilometer", "kilometers", "kilometre", "kilometres"], Dimension::Length, 1000.0),
    UnitDef::linear("in", &["inch", "inches", "in", "″"], Dimension::Length, 0.0254),
    UnitDef::linear("ft", &["ft", "foot", "feet"], Dimension::Length, 0.3048),
    UnitDef::linear("yd", &["yd", "yard", "yards"], Dimension::Length, 0.9144),
    UnitDef::linear("mi", &["mi", "mile", "miles"], Dimension::Length, 1609.344),
    UnitDef::linear("nmi", &["nmi", "nautical mile", "nautical miles"], Dimension::Length, 1852.0),
    UnitDef::linear("μm", &["μm", "micrometer", "micrometers", "micron", "microns"], Dimension::Length, 1e-6),
    UnitDef::linear("nm_len", &["nanometer", "nanometers", "nanometre"], Dimension::Length, 1e-9),
    UnitDef::linear("ly", &["ly", "light year", "light years", "lightyear", "lightyears"], Dimension::Length, 9.461e15),

    // === Area (base: square meter) ===
    UnitDef::linear("sqmm", &["sq mm", "mm²", "mm2", "square millimeter", "square millimeters"], Dimension::Area, 1e-6),
    UnitDef::linear("sqcm", &["sq cm", "cm²", "cm2", "square centimeter", "square centimeters"], Dimension::Area, 1e-4),
    UnitDef::linear("sqm", &["sq m", "m²", "m2", "square meter", "square meters"], Dimension::Area, 1.0),
    UnitDef::linear("sqkm", &["sq km", "km²", "km2", "square kilometer", "square kilometers"], Dimension::Area, 1e6),
    UnitDef::linear("sqin", &["sq in", "in²", "in2", "square inch", "square inches"], Dimension::Area, 0.00064516),
    UnitDef::linear("sqft", &["sq ft", "ft²", "ft2", "square foot", "square feet"], Dimension::Area, 0.09290304),
    UnitDef::linear("sqyd", &["sq yd", "yd²", "yd2", "square yard", "square yards"], Dimension::Area, 0.83612736),
    UnitDef::linear("sqmi", &["sq mi", "mi²", "mi2", "square mile", "square miles"], Dimension::Area, 2589988.110336),
    UnitDef::linear("acre", &["acre", "acres", "ac"], Dimension::Area, 4046.8564224),
    UnitDef::linear("hectare", &["hectare", "hectares", "ha"], Dimension::Area, 10000.0),

    // === Volume (base: liter) ===
    UnitDef::linear("ml", &["ml", "milliliter", "milliliters", "millilitre", "millilitres"], Dimension::Volume, 0.001),
    UnitDef::linear("cl", &["cl", "centiliter", "centiliters"], Dimension::Volume, 0.01),
    UnitDef::linear("l", &["l", "liter", "liters", "litre", "litres"], Dimension::Volume, 1.0),
    UnitDef::linear("gal", &["gal", "gallon", "gallons"], Dimension::Volume, 3.78541),
    UnitDef::linear("qt", &["qt", "quart", "quarts"], Dimension::Volume, 0.946353),
    UnitDef::linear("pt", &["pint", "pints"], Dimension::Volume, 0.473176),
    UnitDef::linear("cup", &["cup", "cups"], Dimension::Volume, 0.236588),
    UnitDef::linear("floz", &["fl oz", "fluid ounce", "fluid ounces", "floz"], Dimension::Volume, 0.0295735),
    UnitDef::linear("tbsp", &["tbsp", "tablespoon", "tablespoons"], Dimension::Volume, 0.0147868),
    UnitDef::linear("tsp", &["tsp", "teaspoon", "teaspoons"], Dimension::Volume, 0.00492892),
    UnitDef::linear("m3", &["m³", "m3", "cubic meter", "cubic meters"], Dimension::Volume, 1000.0),

    // === Mass (base: kilogram) ===
    UnitDef::linear("mg", &["mg", "milligram", "milligrams"], Dimension::Mass, 0.000001),
    UnitDef::linear("g", &["g", "gram", "grams"], Dimension::Mass, 0.001),
    UnitDef::linear("kg", &["kg", "kilogram", "kilograms", "kilo", "kilos"], Dimension::Mass, 1.0),
    UnitDef::linear("t", &["tonne", "tonnes", "metric ton", "metric tons"], Dimension::Mass, 1000.0),
    UnitDef::linear("oz", &["oz", "ounce", "ounces"], Dimension::Mass, 0.0283495),
    UnitDef::linear("lb", &["lb", "lbs", "pound", "pounds"], Dimension::Mass, 0.453592),
    UnitDef::linear("st", &["st", "stone", "stones"], Dimension::Mass, 6.35029),
    UnitDef::linear("ton_us", &["ton", "tons", "short ton", "short tons"], Dimension::Mass, 907.185),

    // === Time (base: second) ===
    UnitDef::linear("ms", &["ms", "millisecond", "milliseconds"], Dimension::Time, 0.001),
    UnitDef::linear("s", &["s", "sec", "second", "seconds"], Dimension::Time, 1.0),
    UnitDef::linear("min", &["min", "minute", "minutes"], Dimension::Time, 60.0),
    UnitDef::linear("hr", &["hr", "hour", "hours"], Dimension::Time, 3600.0),
    UnitDef::linear("day", &["day", "days"], Dimension::Time, 86400.0),
    UnitDef::linear("week", &["week", "weeks", "wk"], Dimension::Time, 604800.0),
    UnitDef::linear("month", &["month", "months", "mo"], Dimension::Time, 2592000.0), // ~30 days
    UnitDef::linear("year", &["year", "years", "yr"], Dimension::Time, 31557600.0), // 365.25 days

    // === Temperature (base: kelvin) ===
    UnitDef::with_offset("celsius", &["°C", "celsius", "C", "degC"], Dimension::Temperature, 1.0, 273.15),
    UnitDef::with_offset("fahrenheit", &["°F", "fahrenheit", "F", "degF"], Dimension::Temperature, 5.0 / 9.0, 255.372_222_222_222_24),
    UnitDef::linear("kelvin", &["K", "kelvin", "kelvins"], Dimension::Temperature, 1.0),

    // === Angle (base: radian) ===
    UnitDef::linear("rad", &["rad", "radian", "radians"], Dimension::Angle, 1.0),
    UnitDef::linear("deg", &["deg", "degree", "degrees", "°"], Dimension::Angle, std::f64::consts::PI / 180.0),
    UnitDef::linear("grad", &["grad", "gradian", "gradians", "gon"], Dimension::Angle, std::f64::consts::PI / 200.0),
    UnitDef::linear("turn", &["turn", "turns", "revolution", "revolutions"], Dimension::Angle, std::f64::consts::TAU),

    // === Data (base: byte) ===
    UnitDef::linear("bit", &["bit", "bits"], Dimension::Data, 0.125),
    UnitDef::linear("byte", &["byte", "bytes", "B"], Dimension::Data, 1.0),
    UnitDef::linear("kb", &["KB", "kilobyte", "kilobytes"], Dimension::Data, 1000.0),
    UnitDef::linear("mb", &["MB", "megabyte", "megabytes"], Dimension::Data, 1e6),
    UnitDef::linear("gb", &["GB", "gigabyte", "gigabytes"], Dimension::Data, 1e9),
    UnitDef::linear("tb", &["TB", "terabyte", "terabytes"], Dimension::Data, 1e12),
    UnitDef::linear("pb", &["PB", "petabyte", "petabytes"], Dimension::Data, 1e15),
    UnitDef::linear("kib", &["KiB", "kibibyte", "kibibytes"], Dimension::Data, 1024.0),
    UnitDef::linear("mib", &["MiB", "mebibyte", "mebibytes"], Dimension::Data, 1_048_576.0),
    UnitDef::linear("gib", &["GiB", "gibibyte", "gibibytes"], Dimension::Data, 1_073_741_824.0),
    UnitDef::linear("tib", &["TiB", "tebibyte", "tebibytes"], Dimension::Data, 1_099_511_627_776.0),

    // === CSS units (base: pixel) ===
    UnitDef::linear("px", &["px", "pixel", "pixels"], Dimension::Css, 1.0),
    UnitDef::linear("pt", &["pt", "point", "points"], Dimension::Css, 96.0 / 72.0),
    UnitDef::linear("pc", &["pc", "pica", "picas"], Dimension::Css, 16.0),
    UnitDef::linear("em", &["em"], Dimension::Css, 16.0),
    UnitDef::linear("rem", &["rem"], Dimension::Css, 16.0),
    UnitDef::linear("vw", &["vw"], Dimension::Css, 1.0), // relative, keep as 1:1 ratio
    UnitDef::linear("vh", &["vh"], Dimension::Css, 1.0),

    // === Speed (base: m/s) ===
    UnitDef::linear("mps", &["m/s", "mps", "meter per second", "meters per second"], Dimension::Speed, 1.0),
    UnitDef::linear("kmph", &["km/h", "kmph", "kph", "kilometer per hour", "kilometers per hour"], Dimension::Speed, 1.0 / 3.6),
    UnitDef::linear("mph", &["mph", "mile per hour", "miles per hour"], Dimension::Speed, 0.44704),
    UnitDef::linear("knot", &["knot", "knots", "kn", "kt"], Dimension::Speed, 0.514444),
];

/// Look up a unit by name (case-insensitive for most, case-sensitive for ambiguous ones)
pub fn find_unit(name: &str) -> Option<&'static UnitDef> {
    let lower = name.to_lowercase();

    // Case-sensitive matches first for ambiguous single-letter units
    for unit in UNITS {
        for &n in unit.names {
            if n == name {
                return Some(unit);
            }
        }
    }

    // Case-insensitive fallback
    for unit in UNITS {
        for &n in unit.names {
            if n.to_lowercase() == lower {
                return Some(unit);
            }
        }
    }

    None
}

/// Check if two units share the same dimension and can be converted
pub fn can_convert(from: &UnitDef, to: &UnitDef) -> bool {
    from.dimension == to.dimension
}

/// Convert a value from one unit to another
pub fn convert(value: f64, from: &UnitDef, to: &UnitDef) -> Option<f64> {
    if from.dimension != to.dimension {
        return None;
    }
    let base = from.to_base(value);
    Some(to.from_base(base))
}
