/* automatically generated by rust-bindgen */

pub const true_: u32 = 1;
pub const false_: u32 = 0;
pub const __bool_true_false_are_defined: u32 = 1;
extern "C" {
    pub fn deskoveryMotor(
        pwrLeft: crate::libc::c_int,
        pwrRight: crate::libc::c_int,
        recovery: bool,
    ) -> bool;
}
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct PrxData {
    pub alarmRatio10: crate::libc::c_int,
    pub alarms: [bool; 4usize],
    pub alarm: bool,
    pub darkResponse: [crate::libc::c_int; 4usize],
    pub lightResponse: [crate::libc::c_int; 4usize],
}
#[test]
fn bindgen_test_layout_PrxData() {
    assert_eq!(
        ::core::mem::size_of::<PrxData>(),
        44usize,
        concat!("Size of: ", stringify!(PrxData))
    );
    assert_eq!(
        ::core::mem::align_of::<PrxData>(),
        4usize,
        concat!("Alignment of ", stringify!(PrxData))
    );
    assert_eq!(
        unsafe { &(*(::core::ptr::null::<PrxData>())).alarmRatio10 as *const _ as usize },
        0usize,
        concat!(
            "Offset of field: ",
            stringify!(PrxData),
            "::",
            stringify!(alarmRatio10)
        )
    );
    assert_eq!(
        unsafe { &(*(::core::ptr::null::<PrxData>())).alarms as *const _ as usize },
        4usize,
        concat!(
            "Offset of field: ",
            stringify!(PrxData),
            "::",
            stringify!(alarms)
        )
    );
    assert_eq!(
        unsafe { &(*(::core::ptr::null::<PrxData>())).alarm as *const _ as usize },
        8usize,
        concat!(
            "Offset of field: ",
            stringify!(PrxData),
            "::",
            stringify!(alarm)
        )
    );
    assert_eq!(
        unsafe { &(*(::core::ptr::null::<PrxData>())).darkResponse as *const _ as usize },
        12usize,
        concat!(
            "Offset of field: ",
            stringify!(PrxData),
            "::",
            stringify!(darkResponse)
        )
    );
    assert_eq!(
        unsafe { &(*(::core::ptr::null::<PrxData>())).lightResponse as *const _ as usize },
        28usize,
        concat!(
            "Offset of field: ",
            stringify!(PrxData),
            "::",
            stringify!(lightResponse)
        )
    );
}
extern "C" {
    pub static mut prxData: PrxData;
}
extern "C" {
    pub static mut left_ticks: crate::libc::c_long;
}
extern "C" {
    pub static mut right_ticks: crate::libc::c_long;
}
extern "C" {
    pub fn ledControl(on: bool);
}
extern "C" {
    pub fn getADCInjectedRank1Value() -> crate::libc::c_int;
}
extern "C" {
    pub fn getADCInjectedRank2Value() -> crate::libc::c_int;
}
extern "C" {
    pub fn getADCInjectedRank3Value() -> crate::libc::c_int;
}
extern "C" {
    pub fn getADCInjectedRank4Value() -> crate::libc::c_int;
}
extern "C" {
    pub fn motorTimerStopChannel1();
}
extern "C" {
    pub fn motorTimerStopChannel2();
}
extern "C" {
    pub fn motorTimerStartChannel1();
}
extern "C" {
    pub fn motorTimerStartChannel2();
}
extern "C" {
    pub fn displayBgControl(brightness: crate::libc::c_int);
}
extern "C" {
    pub fn LCD5110_write_char(c: crate::libc::c_uchar);
}
extern "C" {
    pub fn LCD5110_clear();
}
extern "C" {
    pub fn LCD5110_set_XY(X: crate::libc::c_uchar, Y: crate::libc::c_uchar);
}
extern "C" {
    pub fn LCD5110_write_string(s: *const crate::libc::c_uchar);
}
extern "C" {
    pub fn delayMs(ms: crate::libc::c_long);
}
