#![no_std]
#![allow(non_camel_case_types, non_upper_case_globals, non_snake_case)]
use core::panic::PanicInfo;

// TODO this is fugly
pub mod libc {
    pub type c_int = i32;
    pub type c_long = i64;
    pub type c_uchar = u8;
    pub type c_char = i8;
}
include!("descovery_bindings.rs");

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    //    let mut host_stderr = HStderr::new();

    // logs "panicked at '$reason', src/main.rs:27:4" to the host stderr
    //    writeln!(host_stderr, "{}", info).ok();
    loop {}
}

#[no_mangle]
pub extern "C" fn rust_main() {
    //    let s = "Hello, Embedded World";

    unsafe {
        //        outputStr(s.as_ptr(), s.len());
        let mut brightness: i32 = 0;
        loop {
            ledControl(true);
            delayMs(300);
            ledControl(false);
            brightness = (brightness + 10) % 102;
            displayBgControl(brightness);
            LCD5110_set_XY(0, 0);
            LCD5110_write_string(b"This is RUST!" as *const u8);
        }
    }
}

#[no_mangle]
pub unsafe extern "C" fn update_prx_data(prx_led_on: bool) {
    if prx_led_on {
        prxData.lightResponse[0] = getADCInjectedRank1Value();
        prxData.lightResponse[1] = getADCInjectedRank2Value();
        prxData.lightResponse[2] = getADCInjectedRank3Value();
        prxData.lightResponse[3] = getADCInjectedRank4Value();
        prxData.alarms[0] = (prxData.darkResponse[0] * 10 / (prxData.lightResponse[0] + 1)) < prxData.alarmRatio10;
        prxData.alarms[1] = (prxData.darkResponse[1] * 10 / (prxData.lightResponse[1] + 1)) < prxData.alarmRatio10;
        prxData.alarms[2] = (prxData.darkResponse[2] * 10 / (prxData.lightResponse[2] + 1)) < prxData.alarmRatio10;
        prxData.alarms[3] = (prxData.darkResponse[3] * 10 / (prxData.lightResponse[3] + 1)) < prxData.alarmRatio10;
        prxData.alarm = prxData.alarms[0] || prxData.alarms[1] || prxData.alarms[2] || prxData.alarms[3];
    } else {
        prxData.darkResponse[0] = getADCInjectedRank1Value();
        prxData.darkResponse[1] = getADCInjectedRank2Value();
        prxData.darkResponse[2] = getADCInjectedRank3Value();
        prxData.darkResponse[3] = getADCInjectedRank4Value();
    }
    if prxData.alarm {
        motorTimerStopChannel1();
        motorTimerStopChannel2();
    } else {
        motorTimerStartChannel1();
        motorTimerStartChannel2();
    }
}
