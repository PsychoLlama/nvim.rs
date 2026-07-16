extern "C" {
    static mut p_arshape: ::core::ffi::c_int;
    static mut p_tbidi: ::core::ffi::c_int;
}
pub type size_t = usize;
pub const a_ALEF: C2Rust_Unnamed = 1575;
pub const a_ALEF_HAMZA_BELOW: C2Rust_Unnamed = 1573;
pub const a_ALEF_HAMZA_ABOVE: C2Rust_Unnamed = 1571;
pub const a_ALEF_MADDA: C2Rust_Unnamed = 1570;
pub const a_LAM: C2Rust_Unnamed = 1604;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct achar {
    pub c: ::core::ffi::c_uint,
    pub isolated: ::core::ffi::c_uint,
    pub initial: ::core::ffi::c_uint,
    pub medial: ::core::ffi::c_uint,
    pub final_0: ::core::ffi::c_uint,
}
pub const a_FYEH: C2Rust_Unnamed = 1740;
pub const a_GAF: C2Rust_Unnamed = 1711;
pub const a_FKAF: C2Rust_Unnamed = 1705;
pub const a_JEH: C2Rust_Unnamed = 1688;
pub const a_TCHEH: C2Rust_Unnamed = 1670;
pub const a_PEH: C2Rust_Unnamed = 1662;
pub const a_HAMZA_BELOW: C2Rust_Unnamed = 1621;
pub const a_HAMZA_ABOVE: C2Rust_Unnamed = 1620;
pub const a_MADDA_ABOVE: C2Rust_Unnamed = 1619;
pub const a_SUKUN: C2Rust_Unnamed = 1618;
pub const a_SHADDA: C2Rust_Unnamed = 1617;
pub const a_KASRA: C2Rust_Unnamed = 1616;
pub const a_DAMMA: C2Rust_Unnamed = 1615;
pub const a_FATHA: C2Rust_Unnamed = 1614;
pub const a_KASRATAN: C2Rust_Unnamed = 1613;
pub const a_DAMMATAN: C2Rust_Unnamed = 1612;
pub const a_FATHATAN: C2Rust_Unnamed = 1611;
pub const a_YEH: C2Rust_Unnamed = 1610;
pub const a_ALEF_MAKSURA: C2Rust_Unnamed = 1609;
pub const a_WAW: C2Rust_Unnamed = 1608;
pub const a_HEH: C2Rust_Unnamed = 1607;
pub const a_NOON: C2Rust_Unnamed = 1606;
pub const a_MEEM: C2Rust_Unnamed = 1605;
pub const a_KAF: C2Rust_Unnamed = 1603;
pub const a_QAF: C2Rust_Unnamed = 1602;
pub const a_FEH: C2Rust_Unnamed = 1601;
pub const a_TATWEEL: C2Rust_Unnamed = 1600;
pub const a_GHAIN: C2Rust_Unnamed = 1594;
pub const a_AIN: C2Rust_Unnamed = 1593;
pub const a_ZAH: C2Rust_Unnamed = 1592;
pub const a_TAH: C2Rust_Unnamed = 1591;
pub const a_DAD: C2Rust_Unnamed = 1590;
pub const a_SAD: C2Rust_Unnamed = 1589;
pub const a_SHEEN: C2Rust_Unnamed = 1588;
pub const a_SEEN: C2Rust_Unnamed = 1587;
pub const a_ZAIN: C2Rust_Unnamed = 1586;
pub const a_REH: C2Rust_Unnamed = 1585;
pub const a_THAL: C2Rust_Unnamed = 1584;
pub const a_DAL: C2Rust_Unnamed = 1583;
pub const a_KHAH: C2Rust_Unnamed = 1582;
pub const a_HAH: C2Rust_Unnamed = 1581;
pub const a_JEEM: C2Rust_Unnamed = 1580;
pub const a_THEH: C2Rust_Unnamed = 1579;
pub const a_TEH: C2Rust_Unnamed = 1578;
pub const a_TEH_MARBUTA: C2Rust_Unnamed = 1577;
pub const a_BEH: C2Rust_Unnamed = 1576;
pub const a_YEH_HAMZA: C2Rust_Unnamed = 1574;
pub const a_WAW_HAMZA: C2Rust_Unnamed = 1572;
pub const a_HAMZA: C2Rust_Unnamed = 1569;
pub const a_s_LAM_ALEF: C2Rust_Unnamed = 65275;
pub const a_s_LAM_ALEF_HAMZA_BELOW: C2Rust_Unnamed = 65273;
pub const a_s_LAM_ALEF_HAMZA_ABOVE: C2Rust_Unnamed = 65271;
pub const a_s_LAM_ALEF_MADDA_ABOVE: C2Rust_Unnamed = 65269;
pub const a_f_LAM_ALEF: C2Rust_Unnamed = 65276;
pub const a_f_LAM_ALEF_HAMZA_BELOW: C2Rust_Unnamed = 65274;
pub const a_f_LAM_ALEF_HAMZA_ABOVE: C2Rust_Unnamed = 65272;
pub const a_f_LAM_ALEF_MADDA_ABOVE: C2Rust_Unnamed = 65270;
pub type C2Rust_Unnamed = ::core::ffi::c_uint;
pub const false_0: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
pub const NULL: *mut ::core::ffi::c_void = ::core::ptr::null_mut::<::core::ffi::c_void>();
pub const NUL: ::core::ffi::c_int = '\0' as ::core::ffi::c_int;
static mut achars: [achar; 54] = [
    achar {
        c: a_HAMZA as ::core::ffi::c_int as ::core::ffi::c_uint,
        isolated: 0xfe80 as ::core::ffi::c_uint,
        initial: 0 as ::core::ffi::c_uint,
        medial: 0 as ::core::ffi::c_uint,
        final_0: 0 as ::core::ffi::c_uint,
    },
    achar {
        c: a_ALEF_MADDA as ::core::ffi::c_int as ::core::ffi::c_uint,
        isolated: 0xfe81 as ::core::ffi::c_uint,
        initial: 0 as ::core::ffi::c_uint,
        medial: 0 as ::core::ffi::c_uint,
        final_0: 0xfe82 as ::core::ffi::c_uint,
    },
    achar {
        c: a_ALEF_HAMZA_ABOVE as ::core::ffi::c_int as ::core::ffi::c_uint,
        isolated: 0xfe83 as ::core::ffi::c_uint,
        initial: 0 as ::core::ffi::c_uint,
        medial: 0 as ::core::ffi::c_uint,
        final_0: 0xfe84 as ::core::ffi::c_uint,
    },
    achar {
        c: a_WAW_HAMZA as ::core::ffi::c_int as ::core::ffi::c_uint,
        isolated: 0xfe85 as ::core::ffi::c_uint,
        initial: 0 as ::core::ffi::c_uint,
        medial: 0 as ::core::ffi::c_uint,
        final_0: 0xfe86 as ::core::ffi::c_uint,
    },
    achar {
        c: a_ALEF_HAMZA_BELOW as ::core::ffi::c_int as ::core::ffi::c_uint,
        isolated: 0xfe87 as ::core::ffi::c_uint,
        initial: 0 as ::core::ffi::c_uint,
        medial: 0 as ::core::ffi::c_uint,
        final_0: 0xfe88 as ::core::ffi::c_uint,
    },
    achar {
        c: a_YEH_HAMZA as ::core::ffi::c_int as ::core::ffi::c_uint,
        isolated: 0xfe89 as ::core::ffi::c_uint,
        initial: 0xfe8b as ::core::ffi::c_uint,
        medial: 0xfe8c as ::core::ffi::c_uint,
        final_0: 0xfe8a as ::core::ffi::c_uint,
    },
    achar {
        c: a_ALEF as ::core::ffi::c_int as ::core::ffi::c_uint,
        isolated: 0xfe8d as ::core::ffi::c_uint,
        initial: 0 as ::core::ffi::c_uint,
        medial: 0 as ::core::ffi::c_uint,
        final_0: 0xfe8e as ::core::ffi::c_uint,
    },
    achar {
        c: a_BEH as ::core::ffi::c_int as ::core::ffi::c_uint,
        isolated: 0xfe8f as ::core::ffi::c_uint,
        initial: 0xfe91 as ::core::ffi::c_uint,
        medial: 0xfe92 as ::core::ffi::c_uint,
        final_0: 0xfe90 as ::core::ffi::c_uint,
    },
    achar {
        c: a_TEH_MARBUTA as ::core::ffi::c_int as ::core::ffi::c_uint,
        isolated: 0xfe93 as ::core::ffi::c_uint,
        initial: 0 as ::core::ffi::c_uint,
        medial: 0 as ::core::ffi::c_uint,
        final_0: 0xfe94 as ::core::ffi::c_uint,
    },
    achar {
        c: a_TEH as ::core::ffi::c_int as ::core::ffi::c_uint,
        isolated: 0xfe95 as ::core::ffi::c_uint,
        initial: 0xfe97 as ::core::ffi::c_uint,
        medial: 0xfe98 as ::core::ffi::c_uint,
        final_0: 0xfe96 as ::core::ffi::c_uint,
    },
    achar {
        c: a_THEH as ::core::ffi::c_int as ::core::ffi::c_uint,
        isolated: 0xfe99 as ::core::ffi::c_uint,
        initial: 0xfe9b as ::core::ffi::c_uint,
        medial: 0xfe9c as ::core::ffi::c_uint,
        final_0: 0xfe9a as ::core::ffi::c_uint,
    },
    achar {
        c: a_JEEM as ::core::ffi::c_int as ::core::ffi::c_uint,
        isolated: 0xfe9d as ::core::ffi::c_uint,
        initial: 0xfe9f as ::core::ffi::c_uint,
        medial: 0xfea0 as ::core::ffi::c_uint,
        final_0: 0xfe9e as ::core::ffi::c_uint,
    },
    achar {
        c: a_HAH as ::core::ffi::c_int as ::core::ffi::c_uint,
        isolated: 0xfea1 as ::core::ffi::c_uint,
        initial: 0xfea3 as ::core::ffi::c_uint,
        medial: 0xfea4 as ::core::ffi::c_uint,
        final_0: 0xfea2 as ::core::ffi::c_uint,
    },
    achar {
        c: a_KHAH as ::core::ffi::c_int as ::core::ffi::c_uint,
        isolated: 0xfea5 as ::core::ffi::c_uint,
        initial: 0xfea7 as ::core::ffi::c_uint,
        medial: 0xfea8 as ::core::ffi::c_uint,
        final_0: 0xfea6 as ::core::ffi::c_uint,
    },
    achar {
        c: a_DAL as ::core::ffi::c_int as ::core::ffi::c_uint,
        isolated: 0xfea9 as ::core::ffi::c_uint,
        initial: 0 as ::core::ffi::c_uint,
        medial: 0 as ::core::ffi::c_uint,
        final_0: 0xfeaa as ::core::ffi::c_uint,
    },
    achar {
        c: a_THAL as ::core::ffi::c_int as ::core::ffi::c_uint,
        isolated: 0xfeab as ::core::ffi::c_uint,
        initial: 0 as ::core::ffi::c_uint,
        medial: 0 as ::core::ffi::c_uint,
        final_0: 0xfeac as ::core::ffi::c_uint,
    },
    achar {
        c: a_REH as ::core::ffi::c_int as ::core::ffi::c_uint,
        isolated: 0xfead as ::core::ffi::c_uint,
        initial: 0 as ::core::ffi::c_uint,
        medial: 0 as ::core::ffi::c_uint,
        final_0: 0xfeae as ::core::ffi::c_uint,
    },
    achar {
        c: a_ZAIN as ::core::ffi::c_int as ::core::ffi::c_uint,
        isolated: 0xfeaf as ::core::ffi::c_uint,
        initial: 0 as ::core::ffi::c_uint,
        medial: 0 as ::core::ffi::c_uint,
        final_0: 0xfeb0 as ::core::ffi::c_uint,
    },
    achar {
        c: a_SEEN as ::core::ffi::c_int as ::core::ffi::c_uint,
        isolated: 0xfeb1 as ::core::ffi::c_uint,
        initial: 0xfeb3 as ::core::ffi::c_uint,
        medial: 0xfeb4 as ::core::ffi::c_uint,
        final_0: 0xfeb2 as ::core::ffi::c_uint,
    },
    achar {
        c: a_SHEEN as ::core::ffi::c_int as ::core::ffi::c_uint,
        isolated: 0xfeb5 as ::core::ffi::c_uint,
        initial: 0xfeb7 as ::core::ffi::c_uint,
        medial: 0xfeb8 as ::core::ffi::c_uint,
        final_0: 0xfeb6 as ::core::ffi::c_uint,
    },
    achar {
        c: a_SAD as ::core::ffi::c_int as ::core::ffi::c_uint,
        isolated: 0xfeb9 as ::core::ffi::c_uint,
        initial: 0xfebb as ::core::ffi::c_uint,
        medial: 0xfebc as ::core::ffi::c_uint,
        final_0: 0xfeba as ::core::ffi::c_uint,
    },
    achar {
        c: a_DAD as ::core::ffi::c_int as ::core::ffi::c_uint,
        isolated: 0xfebd as ::core::ffi::c_uint,
        initial: 0xfebf as ::core::ffi::c_uint,
        medial: 0xfec0 as ::core::ffi::c_uint,
        final_0: 0xfebe as ::core::ffi::c_uint,
    },
    achar {
        c: a_TAH as ::core::ffi::c_int as ::core::ffi::c_uint,
        isolated: 0xfec1 as ::core::ffi::c_uint,
        initial: 0xfec3 as ::core::ffi::c_uint,
        medial: 0xfec4 as ::core::ffi::c_uint,
        final_0: 0xfec2 as ::core::ffi::c_uint,
    },
    achar {
        c: a_ZAH as ::core::ffi::c_int as ::core::ffi::c_uint,
        isolated: 0xfec5 as ::core::ffi::c_uint,
        initial: 0xfec7 as ::core::ffi::c_uint,
        medial: 0xfec8 as ::core::ffi::c_uint,
        final_0: 0xfec6 as ::core::ffi::c_uint,
    },
    achar {
        c: a_AIN as ::core::ffi::c_int as ::core::ffi::c_uint,
        isolated: 0xfec9 as ::core::ffi::c_uint,
        initial: 0xfecb as ::core::ffi::c_uint,
        medial: 0xfecc as ::core::ffi::c_uint,
        final_0: 0xfeca as ::core::ffi::c_uint,
    },
    achar {
        c: a_GHAIN as ::core::ffi::c_int as ::core::ffi::c_uint,
        isolated: 0xfecd as ::core::ffi::c_uint,
        initial: 0xfecf as ::core::ffi::c_uint,
        medial: 0xfed0 as ::core::ffi::c_uint,
        final_0: 0xfece as ::core::ffi::c_uint,
    },
    achar {
        c: a_TATWEEL as ::core::ffi::c_int as ::core::ffi::c_uint,
        isolated: 0 as ::core::ffi::c_uint,
        initial: 0x640 as ::core::ffi::c_uint,
        medial: 0x640 as ::core::ffi::c_uint,
        final_0: 0x640 as ::core::ffi::c_uint,
    },
    achar {
        c: a_FEH as ::core::ffi::c_int as ::core::ffi::c_uint,
        isolated: 0xfed1 as ::core::ffi::c_uint,
        initial: 0xfed3 as ::core::ffi::c_uint,
        medial: 0xfed4 as ::core::ffi::c_uint,
        final_0: 0xfed2 as ::core::ffi::c_uint,
    },
    achar {
        c: a_QAF as ::core::ffi::c_int as ::core::ffi::c_uint,
        isolated: 0xfed5 as ::core::ffi::c_uint,
        initial: 0xfed7 as ::core::ffi::c_uint,
        medial: 0xfed8 as ::core::ffi::c_uint,
        final_0: 0xfed6 as ::core::ffi::c_uint,
    },
    achar {
        c: a_KAF as ::core::ffi::c_int as ::core::ffi::c_uint,
        isolated: 0xfed9 as ::core::ffi::c_uint,
        initial: 0xfedb as ::core::ffi::c_uint,
        medial: 0xfedc as ::core::ffi::c_uint,
        final_0: 0xfeda as ::core::ffi::c_uint,
    },
    achar {
        c: a_LAM as ::core::ffi::c_int as ::core::ffi::c_uint,
        isolated: 0xfedd as ::core::ffi::c_uint,
        initial: 0xfedf as ::core::ffi::c_uint,
        medial: 0xfee0 as ::core::ffi::c_uint,
        final_0: 0xfede as ::core::ffi::c_uint,
    },
    achar {
        c: a_MEEM as ::core::ffi::c_int as ::core::ffi::c_uint,
        isolated: 0xfee1 as ::core::ffi::c_uint,
        initial: 0xfee3 as ::core::ffi::c_uint,
        medial: 0xfee4 as ::core::ffi::c_uint,
        final_0: 0xfee2 as ::core::ffi::c_uint,
    },
    achar {
        c: a_NOON as ::core::ffi::c_int as ::core::ffi::c_uint,
        isolated: 0xfee5 as ::core::ffi::c_uint,
        initial: 0xfee7 as ::core::ffi::c_uint,
        medial: 0xfee8 as ::core::ffi::c_uint,
        final_0: 0xfee6 as ::core::ffi::c_uint,
    },
    achar {
        c: a_HEH as ::core::ffi::c_int as ::core::ffi::c_uint,
        isolated: 0xfee9 as ::core::ffi::c_uint,
        initial: 0xfeeb as ::core::ffi::c_uint,
        medial: 0xfeec as ::core::ffi::c_uint,
        final_0: 0xfeea as ::core::ffi::c_uint,
    },
    achar {
        c: a_WAW as ::core::ffi::c_int as ::core::ffi::c_uint,
        isolated: 0xfeed as ::core::ffi::c_uint,
        initial: 0 as ::core::ffi::c_uint,
        medial: 0 as ::core::ffi::c_uint,
        final_0: 0xfeee as ::core::ffi::c_uint,
    },
    achar {
        c: a_ALEF_MAKSURA as ::core::ffi::c_int as ::core::ffi::c_uint,
        isolated: 0xfeef as ::core::ffi::c_uint,
        initial: 0 as ::core::ffi::c_uint,
        medial: 0 as ::core::ffi::c_uint,
        final_0: 0xfef0 as ::core::ffi::c_uint,
    },
    achar {
        c: a_YEH as ::core::ffi::c_int as ::core::ffi::c_uint,
        isolated: 0xfef1 as ::core::ffi::c_uint,
        initial: 0xfef3 as ::core::ffi::c_uint,
        medial: 0xfef4 as ::core::ffi::c_uint,
        final_0: 0xfef2 as ::core::ffi::c_uint,
    },
    achar {
        c: a_FATHATAN as ::core::ffi::c_int as ::core::ffi::c_uint,
        isolated: 0xfe70 as ::core::ffi::c_uint,
        initial: 0 as ::core::ffi::c_uint,
        medial: 0 as ::core::ffi::c_uint,
        final_0: 0 as ::core::ffi::c_uint,
    },
    achar {
        c: a_DAMMATAN as ::core::ffi::c_int as ::core::ffi::c_uint,
        isolated: 0xfe72 as ::core::ffi::c_uint,
        initial: 0 as ::core::ffi::c_uint,
        medial: 0 as ::core::ffi::c_uint,
        final_0: 0 as ::core::ffi::c_uint,
    },
    achar {
        c: a_KASRATAN as ::core::ffi::c_int as ::core::ffi::c_uint,
        isolated: 0xfe74 as ::core::ffi::c_uint,
        initial: 0 as ::core::ffi::c_uint,
        medial: 0 as ::core::ffi::c_uint,
        final_0: 0 as ::core::ffi::c_uint,
    },
    achar {
        c: a_FATHA as ::core::ffi::c_int as ::core::ffi::c_uint,
        isolated: 0xfe76 as ::core::ffi::c_uint,
        initial: 0 as ::core::ffi::c_uint,
        medial: 0xfe77 as ::core::ffi::c_uint,
        final_0: 0 as ::core::ffi::c_uint,
    },
    achar {
        c: a_DAMMA as ::core::ffi::c_int as ::core::ffi::c_uint,
        isolated: 0xfe78 as ::core::ffi::c_uint,
        initial: 0 as ::core::ffi::c_uint,
        medial: 0xfe79 as ::core::ffi::c_uint,
        final_0: 0 as ::core::ffi::c_uint,
    },
    achar {
        c: a_KASRA as ::core::ffi::c_int as ::core::ffi::c_uint,
        isolated: 0xfe7a as ::core::ffi::c_uint,
        initial: 0 as ::core::ffi::c_uint,
        medial: 0xfe7b as ::core::ffi::c_uint,
        final_0: 0 as ::core::ffi::c_uint,
    },
    achar {
        c: a_SHADDA as ::core::ffi::c_int as ::core::ffi::c_uint,
        isolated: 0xfe7c as ::core::ffi::c_uint,
        initial: 0 as ::core::ffi::c_uint,
        medial: 0xfe7c as ::core::ffi::c_uint,
        final_0: 0 as ::core::ffi::c_uint,
    },
    achar {
        c: a_SUKUN as ::core::ffi::c_int as ::core::ffi::c_uint,
        isolated: 0xfe7e as ::core::ffi::c_uint,
        initial: 0 as ::core::ffi::c_uint,
        medial: 0xfe7f as ::core::ffi::c_uint,
        final_0: 0 as ::core::ffi::c_uint,
    },
    achar {
        c: a_MADDA_ABOVE as ::core::ffi::c_int as ::core::ffi::c_uint,
        isolated: 0 as ::core::ffi::c_uint,
        initial: 0 as ::core::ffi::c_uint,
        medial: 0 as ::core::ffi::c_uint,
        final_0: 0 as ::core::ffi::c_uint,
    },
    achar {
        c: a_HAMZA_ABOVE as ::core::ffi::c_int as ::core::ffi::c_uint,
        isolated: 0 as ::core::ffi::c_uint,
        initial: 0 as ::core::ffi::c_uint,
        medial: 0 as ::core::ffi::c_uint,
        final_0: 0 as ::core::ffi::c_uint,
    },
    achar {
        c: a_HAMZA_BELOW as ::core::ffi::c_int as ::core::ffi::c_uint,
        isolated: 0 as ::core::ffi::c_uint,
        initial: 0 as ::core::ffi::c_uint,
        medial: 0 as ::core::ffi::c_uint,
        final_0: 0 as ::core::ffi::c_uint,
    },
    achar {
        c: a_PEH as ::core::ffi::c_int as ::core::ffi::c_uint,
        isolated: 0xfb56 as ::core::ffi::c_uint,
        initial: 0xfb58 as ::core::ffi::c_uint,
        medial: 0xfb59 as ::core::ffi::c_uint,
        final_0: 0xfb57 as ::core::ffi::c_uint,
    },
    achar {
        c: a_TCHEH as ::core::ffi::c_int as ::core::ffi::c_uint,
        isolated: 0xfb7a as ::core::ffi::c_uint,
        initial: 0xfb7c as ::core::ffi::c_uint,
        medial: 0xfb7d as ::core::ffi::c_uint,
        final_0: 0xfb7b as ::core::ffi::c_uint,
    },
    achar {
        c: a_JEH as ::core::ffi::c_int as ::core::ffi::c_uint,
        isolated: 0xfb8a as ::core::ffi::c_uint,
        initial: 0 as ::core::ffi::c_uint,
        medial: 0 as ::core::ffi::c_uint,
        final_0: 0xfb8b as ::core::ffi::c_uint,
    },
    achar {
        c: a_FKAF as ::core::ffi::c_int as ::core::ffi::c_uint,
        isolated: 0xfb8e as ::core::ffi::c_uint,
        initial: 0xfb90 as ::core::ffi::c_uint,
        medial: 0xfb91 as ::core::ffi::c_uint,
        final_0: 0xfb8f as ::core::ffi::c_uint,
    },
    achar {
        c: a_GAF as ::core::ffi::c_int as ::core::ffi::c_uint,
        isolated: 0xfb92 as ::core::ffi::c_uint,
        initial: 0xfb94 as ::core::ffi::c_uint,
        medial: 0xfb95 as ::core::ffi::c_uint,
        final_0: 0xfb93 as ::core::ffi::c_uint,
    },
    achar {
        c: a_FYEH as ::core::ffi::c_int as ::core::ffi::c_uint,
        isolated: 0xfbfc as ::core::ffi::c_uint,
        initial: 0xfbfe as ::core::ffi::c_uint,
        medial: 0xfbff as ::core::ffi::c_uint,
        final_0: 0xfbfd as ::core::ffi::c_uint,
    },
];
pub const a_BYTE_ORDER_MARK: ::core::ffi::c_int = 0xfeff as ::core::ffi::c_int;
unsafe extern "C" fn find_achar(mut c: ::core::ffi::c_int) -> *mut achar {
    let mut h: ::core::ffi::c_int = ::core::mem::size_of::<[achar; 54]>()
        .wrapping_div(::core::mem::size_of::<achar>())
        .wrapping_div(
            (::core::mem::size_of::<[achar; 54]>().wrapping_rem(::core::mem::size_of::<achar>())
                == 0) as ::core::ffi::c_int as usize,
        ) as ::core::ffi::c_int;
    let mut l: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    while l < h {
        let mut m: ::core::ffi::c_int = (h + l) / 2 as ::core::ffi::c_int;
        if achars[m as usize].c == c as ::core::ffi::c_uint {
            return (&raw mut achars as *mut achar).offset(m as isize) as *mut achar;
        }
        if (c as ::core::ffi::c_uint) < achars[m as usize].c {
            h = m;
        } else {
            l = m + 1 as ::core::ffi::c_int;
        }
    }
    return ::core::ptr::null_mut::<achar>();
}
unsafe extern "C" fn chg_c_laa2i(mut hid_c: ::core::ffi::c_int) -> ::core::ffi::c_int {
    let mut tempc: ::core::ffi::c_int = 0;
    match hid_c {
        1570 => {
            tempc = a_s_LAM_ALEF_MADDA_ABOVE as ::core::ffi::c_int;
        }
        1571 => {
            tempc = a_s_LAM_ALEF_HAMZA_ABOVE as ::core::ffi::c_int;
        }
        1573 => {
            tempc = a_s_LAM_ALEF_HAMZA_BELOW as ::core::ffi::c_int;
        }
        1575 => {
            tempc = a_s_LAM_ALEF as ::core::ffi::c_int;
        }
        _ => {
            tempc = 0 as ::core::ffi::c_int;
        }
    }
    return tempc;
}
unsafe extern "C" fn chg_c_laa2f(mut hid_c: ::core::ffi::c_int) -> ::core::ffi::c_int {
    let mut tempc: ::core::ffi::c_int = 0;
    match hid_c {
        1570 => {
            tempc = a_f_LAM_ALEF_MADDA_ABOVE as ::core::ffi::c_int;
        }
        1571 => {
            tempc = a_f_LAM_ALEF_HAMZA_ABOVE as ::core::ffi::c_int;
        }
        1573 => {
            tempc = a_f_LAM_ALEF_HAMZA_BELOW as ::core::ffi::c_int;
        }
        1575 => {
            tempc = a_f_LAM_ALEF as ::core::ffi::c_int;
        }
        _ => {
            tempc = 0 as ::core::ffi::c_int;
        }
    }
    return tempc;
}
unsafe extern "C" fn can_join(
    mut c1: ::core::ffi::c_int,
    mut c2: ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    let mut a1: *mut achar = find_achar(c1);
    let mut a2: *mut achar = find_achar(c2);
    return (!a1.is_null()
        && !a2.is_null()
        && ((*a1).initial != 0 || (*a1).medial != 0)
        && ((*a2).final_0 != 0 || (*a2).medial != 0)) as ::core::ffi::c_int;
}
#[no_mangle]
pub unsafe extern "C" fn arabic_maycombine(mut two: ::core::ffi::c_int) -> bool {
    if p_arshape != 0 && p_tbidi == 0 {
        return two == a_ALEF_MADDA as ::core::ffi::c_int
            || two == a_ALEF_HAMZA_ABOVE as ::core::ffi::c_int
            || two == a_ALEF_HAMZA_BELOW as ::core::ffi::c_int
            || two == a_ALEF as ::core::ffi::c_int;
    }
    return false_0 != 0;
}
#[no_mangle]
pub unsafe extern "C" fn arabic_combine(
    mut one: ::core::ffi::c_int,
    mut two: ::core::ffi::c_int,
) -> bool {
    if one == a_LAM as ::core::ffi::c_int {
        return arabic_maycombine(two);
    }
    return false_0 != 0;
}
unsafe extern "C" fn A_is_iso(mut c: ::core::ffi::c_int) -> bool {
    return !find_achar(c).is_null();
}
unsafe extern "C" fn A_is_ok(mut c: ::core::ffi::c_int) -> bool {
    return A_is_iso(c) as ::core::ffi::c_int != 0 || c == a_BYTE_ORDER_MARK;
}
unsafe extern "C" fn A_is_valid(mut c: ::core::ffi::c_int) -> bool {
    return A_is_ok(c) as ::core::ffi::c_int != 0 && c != a_HAMZA as ::core::ffi::c_int;
}
#[no_mangle]
pub unsafe extern "C" fn arabic_shape(
    mut c: ::core::ffi::c_int,
    mut c1p: *mut ::core::ffi::c_int,
    mut prev_c: ::core::ffi::c_int,
    mut prev_c1: ::core::ffi::c_int,
    mut next_c: ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    if !A_is_ok(c) {
        return c;
    }
    let mut curr_c: ::core::ffi::c_int = 0;
    let mut curr_laa: bool = arabic_combine(c, *c1p);
    let mut prev_laa: bool = arabic_combine(prev_c, prev_c1);
    if curr_laa {
        if A_is_valid(prev_c) as ::core::ffi::c_int != 0
            && can_join(prev_c, a_LAM as ::core::ffi::c_int) != 0
            && !prev_laa
        {
            curr_c = chg_c_laa2f(*c1p);
        } else {
            curr_c = chg_c_laa2i(*c1p);
        }
        *c1p = 0 as ::core::ffi::c_int;
    } else {
        let mut curr_a: *mut achar = find_achar(c);
        let mut backward_combine: ::core::ffi::c_int =
            (!prev_laa && can_join(prev_c, c) != 0) as ::core::ffi::c_int;
        let mut forward_combine: ::core::ffi::c_int = can_join(c, next_c);
        if backward_combine != 0 {
            if forward_combine != 0 {
                curr_c = (*curr_a).medial as ::core::ffi::c_int;
            } else {
                curr_c = (*curr_a).final_0 as ::core::ffi::c_int;
            }
        } else if forward_combine != 0 {
            curr_c = (*curr_a).initial as ::core::ffi::c_int;
        } else {
            curr_c = (*curr_a).isolated as ::core::ffi::c_int;
        }
    }
    if curr_c == NUL {
        curr_c = c;
    }
    return curr_c;
}
