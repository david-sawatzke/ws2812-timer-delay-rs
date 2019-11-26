//! # Use ws2812 leds with timers
//!
//! - For usage with `smart-leds`
//! - Implements the `SmartLedsWrite` trait
//!
//! The `new` method needs a periodic timer running at 3 MHz
//!
//! If it's too slow (e.g.  e.g. all/some leds are white or display the wrong color)
//! you may want to try the `slow` feature.

#![no_std]

use embedded_hal as hal;

use crate::hal::digital::v2::OutputPin;
use crate::hal::timer::{CountDown, Periodic};
use core::borrow::BorrowMut;
use core::marker::PhantomData;
use smart_leds_trait::{SmartLedsWrite, RGB8};

use nb;
use nb::block;

pub struct Ws2812<TIMER, INNER_TIMER, PIN, INNER_PIN> {
    timer: TIMER,
    pin: PIN,
    _timer: PhantomData<INNER_TIMER>,
    _pin: PhantomData<INNER_PIN>,
}

impl<TIMER, INNER_TIMER, PIN, INNER_PIN> Ws2812<TIMER, INNER_TIMER, PIN, INNER_PIN>
where
    TIMER: BorrowMut<INNER_TIMER>,
    INNER_TIMER: CountDown + Periodic,
    PIN: BorrowMut<INNER_PIN>,
    INNER_PIN: OutputPin,
{
    /// The timer has to already run at with a frequency of 3 MHz
    pub fn new(timer: TIMER, mut pin: PIN) -> Self {
        pin.borrow_mut().set_low().ok();
        Self {
            timer,
            pin,
            _timer: PhantomData,
            _pin: PhantomData,
        }
    }

    /// Write a single color for ws2812 devices
    #[cfg(feature = "slow")]
    fn write_byte(&mut self, mut data: u8) {
        for _ in 0..8 {
            if (data & 0x80) != 0 {
                block!(self.timer.borrow_mut().wait()).ok();
                self.pin.borrow_mut().set_high().ok();
                block!(self.timer.borrow_mut().wait()).ok();
                block!(self.timer.borrow_mut().wait()).ok();
                self.pin.borrow_mut().set_low().ok();
            } else {
                block!(self.timer.borrow_mut().wait()).ok();
                self.pin.borrow_mut().set_high().ok();
                self.pin.borrow_mut().set_low().ok();
                block!(self.timer.borrow_mut().wait()).ok();
                block!(self.timer.borrow_mut().wait()).ok();
            }
            data <<= 1;
        }
    }

    /// Write a single color for ws2812 devices
    #[cfg(not(feature = "slow"))]
    fn write_byte(&mut self, mut data: u8) {
        for _ in 0..8 {
            if (data & 0x80) != 0 {
                block!(self.timer.borrow_mut().wait()).ok();
                self.pin.borrow_mut().set_high().ok();
                block!(self.timer.borrow_mut().wait()).ok();
                block!(self.timer.borrow_mut().wait()).ok();
                self.pin.borrow_mut().set_low().ok();
            } else {
                block!(self.timer.borrow_mut().wait()).ok();
                self.pin.borrow_mut().set_high().ok();
                block!(self.timer.borrow_mut().wait()).ok();
                self.pin.borrow_mut().set_low().ok();
                block!(self.timer.borrow_mut().wait()).ok();
            }
            data <<= 1;
        }
    }
}

impl<TIMER, INNER_TIMER, PIN, INNER_PIN> SmartLedsWrite
    for Ws2812<TIMER, INNER_TIMER, PIN, INNER_PIN>
where
    TIMER: BorrowMut<INNER_TIMER>,
    INNER_TIMER: CountDown + Periodic,
    PIN: BorrowMut<INNER_PIN>,
    INNER_PIN: OutputPin,
{
    type Error = ();
    type Color = RGB8;
    /// Write all the items of an iterator to a ws2812 strip
    fn write<T, I>(&mut self, iterator: T) -> Result<(), Self::Error>
    where
        T: Iterator<Item = I>,
        I: Into<Self::Color>,
    {
        for item in iterator {
            let item = item.into();
            self.write_byte(item.g);
            self.write_byte(item.r);
            self.write_byte(item.b);
        }
        // Get a timeout period of 300 ns
        for _ in 0..900 {
            block!(self.timer.borrow_mut().wait()).ok();
        }
        Ok(())
    }
}
