
#![no_main]
#![no_std]

extern crate cortex_m;
extern crate cortex_m_rt;
extern crate msp432p401r;
extern crate panic_halt;

use cortex_m_rt::entry;
use cortex_m::asm;

use msp432p401r::interrupt;

use lazy_static::lazy_static;

use spin::Mutex;

lazy_static! {
    pub static ref PERIPHERALS: Mutex<msp432p401r::Peripherals> = Mutex::new(msp432p401r::Peripherals::take().unwrap());
}

#[entry]
fn main() -> ! {

    // We can only hold onto the periphrials for the time we lock them for.
    // Note that if we interrupt while this is locked, we will deadlock. Because of that, this must be an interrupt free zone.
    // This is a critical section!
    cortex_m::interrupt::free(|_| {
        // See that odd little _ up there? You could put any variable name you want up there. Typically you'll put "cs" there for "critical section".
        // It's nothing but a token. By the time this compiles into the final product, there will be nothing left of it. Its purpose is to let functions
        // that demand you call them from a critical section know you called from a critical section. Because it's an argument to the function, the
        // code will refuse to compile unless you provide it with the critical section token, which can only exist in a critical section.
        // Hiza! Compile time saftey checks!

        // Our peripherals must be shared with the interrupt. We have to convince Rust that it's not going to try
        // accessing the periphrials while we're using them and cause memory coruption. 
        let p = PERIPHERALS.lock();

        // We take the cortex peripherals. These are only taken locally since we do not need to share them with the interrupt.
        let cortex_p = cortex_m::Peripherals::take().unwrap();

        // Get the Watchdog Timer
        let wdt = &p.WDT_A;

        // Get the Digital I/O module
        let dio = &p.DIO;

        // Get the Nested Vector Interrupt Controller.
        // We don't need access to this from the interrupt, so we just hold it locally and then drop it when we return from here.
        // Do however note that this means we cannot ever use it again, unless we do something like what was done with the MSP
        // periphrials.
        let nvic = &cortex_p.NVIC;

        // We shall disable the timer.
        wdt.wdtctl.write(|w| {
            unsafe {
                w.wdtpw().bits(0x5A);
            }
            w.wdthold().bit(true)
        });

        // Setup pin 1 on port 1 to be an input.
        // Enable the interrupt for it.

        // Configure P1.1 as input.
        dio.padir.modify(|r, w| unsafe { w.p1dir().bits(r.p1dir().bits() & !0x02) });

        // Enable P1.1 pullup resistor.
        dio.paren.modify(|r, w| unsafe { w.p1ren().bits(r.p1ren().bits() | 0x02) });  // Enable.
        dio.paout.modify(|r, w| unsafe { w.p1out().bits(r.p1out().bits() |  0x02) }); // Pull up.

        // Set device selection to GPIO, not something else.
        dio.pasel0.write(|w| unsafe { w.p1sel0().bits(0x00) });
        dio.pasel1.write(|w| unsafe { w.p1sel1().bits(0x00) });

        // Enable and configure interrupts.
        dio.paies.write(|w| unsafe { w.p1ies().bits(0x02) }); // Interrupt on high-to-low.
        dio.paifg.write(|w| unsafe { w.p1ifg().bits(0x00) }); // Clear all P1 interrupt flags.
        dio.paie.write(|w| unsafe { w.p1ie().bits(0x02) }); // Enable interrupt for P1.1.

        // Enable Port 1 interrupt on the NVIC
        // See section 2.4.3 of the manual for information.
        unsafe {
            nvic.iser[1].write(0x08)
        };

        // The red LED is on port 2 pin 0. Set it to be an output.
        dio.padir.modify(|r, w| unsafe { w.p2dir().bits(r.p2dir().bits() | 0x01) });
    });

    loop {
        // Will put the processor to sleep until the next interrupt happens.
        asm::wfi();
    }
}

// To use this macro, we had to enable the rt feature in the msp432p401r crate. See the Cargo.toml file for details.
#[interrupt]
fn PORT1_IRQ() {
    static mut STATE: bool = false;

    *STATE = !*STATE;

    cortex_m::interrupt::free(|_| {
        let p = PERIPHERALS.lock();

        // Get the Digital I/O module
        let dio = &p.DIO;

        if *STATE {
            // Set LED output to on.
            dio.paout.modify(|r, w| unsafe { w.p2out().bits(r.p2out().bits() | 1) });
        } else {
            // Set LED output to off.
            dio.paout.modify(|r, w| unsafe { w.p2out().bits(r.p2out().bits() & 0) });
        }

        dio.paifg.write(|w| unsafe { w.p1ifg().bits(0x00) }); // Clear all P1 interrupt flags.
    });
}