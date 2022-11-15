#![no_std]
#![no_main]

use esp32s3_hal::{
    clock::ClockControl,
    otg_fs::{UsbBus, USB},
    pac::Peripherals,
    prelude::*,
    timer::TimerGroup,
    Rtc, IO,
};
use esp_backtrace as _;
use usb_device::prelude::{UsbDeviceBuilder, UsbVidPid};
use usbd_hid::descriptor::SerializedDescriptor;
use usbd_hid::{descriptor::MouseReport, hid_class::HIDClass};

static mut EP_MEMORY: [u32; 1024] = [0; 1024];

pub const USB_VID: u16 = 0x0000;
pub const USB_PID: u16 = 0x0000;

#[xtensa_lx_rt::entry]
fn main() -> ! {
    let peripherals = Peripherals::take().unwrap();
    let mut system = peripherals.SYSTEM.split();
    let clocks = ClockControl::boot_defaults(system.clock_control).freeze();

    // Disable the RTC and TIMG watchdog timers
    let mut rtc = Rtc::new(peripherals.RTC_CNTL);
    let timer_group0 = TimerGroup::new(peripherals.TIMG0, &clocks);
    let mut wdt0 = timer_group0.wdt;
    let timer_group1 = TimerGroup::new(peripherals.TIMG1, &clocks);
    let mut wdt1 = timer_group1.wdt;

    rtc.rwdt.disable();
    wdt0.disable();
    wdt1.disable();

    let io = IO::new(peripherals.GPIO, peripherals.IO_MUX);

    let usb = USB::new(
        peripherals.USB0,
        io.pins.gpio18,
        io.pins.gpio19,
        io.pins.gpio20,
        &mut system.peripheral_clock_control,
    );

    let usb_bus = UsbBus::new(usb, unsafe { &mut EP_MEMORY });

    let mut hid = HIDClass::new(&usb_bus, MouseReport::desc(), 60);

    let mut usb_dev = UsbDeviceBuilder::new(&usb_bus, UsbVidPid(USB_VID, USB_PID))
        .manufacturer("m")
        .product("p")
        .serial_number("123")
        .device_class(3)
        .build();

    loop {
        if !usb_dev.poll(&mut [&mut hid]) {
            continue;
        }

        let _res = hid.push_input(&MouseReport {
            x: 0,
            y: 4,
            buttons: 0,
            wheel: 0,
            pan: 0,
        });
    }
}
