
#![no_main]
#![no_std]

extern crate cortex_m;
extern crate cortex_m_rt;
extern crate msp432p401r;

extern crate panic_halt;

use cortex_m_rt::entry;

#[entry]
fn main() -> ! {

    let p = msp432p401r::Peripherals::take().unwrap();

    // Get the Watchdog Timer
    let wdt = p.WDT_A;

    // We shall disable the timer.
    wdt.wdtctl.write(|w| {
        unsafe {
            w.wdtpw().bits(0x5A);
        }
        w.wdthold().bit(true)
    });

    // Get the Digital I/O module
    let dio = p.DIO;

    // Setup pin 1 on port 1 to be an input.
    // Enable the interrupt for it.

    // Configure P1.1 as input.
    dio.padir.modify(|r, w| unsafe { w.p1dir().bits(r.p1dir().bits() & !0x02) });
    dio.paout.modify(|r, w| unsafe { w.p1out().bits(r.p1out().bits() |  0x02) });

    // Enable P1.1 pullup resistor.
    dio.paren.modify(|r, w| unsafe { w.p1ren().bits(r.p1ren().bits() | 0x02) });
    dio.pasel0.write(|w| unsafe { w.p1sel0().bits(0x00) });
    dio.pasel1.write(|w| unsafe { w.p1sel1().bits(0x00) });

    // The red LED is on port 2 pin 0. Set it to be an output.
    dio.padir.modify(|r, w| unsafe { w.p2dir().bits(r.p2dir().bits() | 0x01) });

    loop {
        // We are pulling a pullup resistor low, so we check if this is zero to know that it is true.
        let is_pressed = dio.pain.read().bits() & 0x02 == 0;

        if is_pressed {
            // Set LED output to on.
            dio.paout.modify(|r, w| unsafe { w.p2out().bits(r.p2out().bits() | 1) });
        } else {
            // Set LED output to off.
            dio.paout.modify(|r, w| unsafe { w.p2out().bits(r.p2out().bits() & 0) });
        }
    }
}
