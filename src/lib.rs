#![no_std]

pub use stm32f103_pac as pac;

///data for shift register
pub type Data = [u8; 8];

pub static mut MOVING_FORWARD: bool = false;

pub mod clocks {
    use stm32f103_pac::{RCC, FLASH};

    pub struct Clocks {
        pub rcc:  RCC,
        pub flash: FLASH,
    }

    impl Clocks {
        pub fn new(rcc: RCC, flash: FLASH) -> Self {
            Clocks {
                rcc,
                flash,
            }
        }

        pub fn configure(&mut self) {
            self.flash.acr.modify(|_, w| unsafe { w
                .latency().bits(0b010) });//flash latency of two wait states
            self.flash.acr.modify(|_, w| w
                                  .prftbe().set_bit());//enable prefetch buffer

            self.rcc.cfgr.modify(|_, w| unsafe { w
                .ppre1().bits(0b100)//apb1 divide by 2
                    .ppre2().bits(0b000)//apb2 not divided
                    .pllxtpre().clear_bit()//hse entry into pll not divided
            });

            self.rcc.cr.write(|w| w.hseon().set_bit());//turn on hse
            while self.rcc.cr.read().hserdy().bit_is_clear() {}//wait until hse ready

            self.rcc.cfgr.modify(|_, w| unsafe { w
                .pllsrc().set_bit()//pll source set as pll
                    .pllmul().bits(0b0111)//pll mult factor set as 9
                    .hpre().bits(0b0000)//sysclk not divided
            });

            self.rcc.cr.modify(|_, w| w.pllon().set_bit());//turn pll on
            while self.rcc.cr.read().pllrdy().bit_is_clear() {}//wait until pll ready

            self.rcc.cfgr.modify(|_, w| unsafe {
                w.sw().bits(0b10)//pll selected as sysclk
            });

            while !self.rcc.cfgr.read().sws().eq(&0b10) {}//confirm selected clock

            //Clocks set so...
            //HCLK -> 72MHz
            //PCLK1 -> 36MHz
            //PCLK2 -> 72MHz
        }
    }
}

pub mod usart1 {
    use super::clocks::Clocks;
    use stm32f103_pac::USART1;

    pub struct Usart1 {
       pub usart1: USART1,
    }

    impl Usart1 {
        pub fn config(clocks: &Clocks, usart1: USART1) -> Self {
            //Enable clock to usart1
            clocks.rcc.apb2enr.modify(|_, w| w.usart1en().set_bit());

            usart1.cr1.reset();
            usart1.cr1.write(|w| w.ue().set_bit());//Enable usart1
            usart1.brr.write(|w| unsafe { w.div_mantissa().bits(468).div_fraction().bits(12) });//Program baud rate to 9600
            
            usart1.cr1.modify(|_, w| w.te().set_bit());//Enable transmitter. WORKS ONLY WITH 'MODIFY'
            usart1.cr1.modify(|_, w| w.re().set_bit());//Enable receiver

            Usart1 { usart1 }
        }

        pub fn transmit(&mut self, data: u16) {
            while self.usart1.sr.read().txe().bit_is_clear() {}//Wait until txe is set
            self.usart1.dr.write(|w| unsafe { w.dr().bits(data)});//Put data in data register
            while self.usart1.sr.read().tc().bit_is_clear() {}//Wait until tc is set
        }

        pub fn receive(&mut self) -> u16 {
            while self.usart1.sr.read().rxne().bit_is_clear() {}//Wait until data is available
            self.usart1.dr.read().dr().bits()
        }

        pub fn enable_interrupt(&mut self) {
            self.usart1.cr1.modify(|_, w| w.rxneie().set_bit());//usart interrupt when rxne set
        }

        pub fn disable_interrupt(&mut self) {
            self.usart1.cr1.modify(|_, w| w.rxneie().clear_bit());//usart interrupt rxnie disable
        }
    }
}

pub mod led {
    use super::clocks::Clocks;
    use stm32f103_pac::GPIOC;

    pub struct Led {
        portc: GPIOC,
    }

    impl Led {
        pub fn new(clocks: &Clocks, portc: GPIOC) -> Self {
            //Enable clock to gpioc
            clocks.rcc.apb2enr.modify(|_, w| w.iopcen().set_bit());

            //Configure pin 13 as output
            portc.crh
                .write(|w| unsafe {
                    w.mode13().bits(0b01)//Output mode, max speed 10 MHz.
                        .cnf13().bits(0b00)//General purpose output push-pull
                });

            portc.odr.modify(|_, w| w.odr13().set_bit());//Put OFF initially

            Led { portc }
        }

        pub fn toggle(&mut self) {
            self.portc.odr.modify(|r, w| w.odr13().bit(!r.odr13().bit()));
        }

        pub fn on(&mut self) {
            self.portc.odr.modify(|_, w| w.odr13().clear_bit());
        }

        pub fn off(&mut self) {
            self.portc.odr.modify(|_, w| w.odr13().set_bit());
        }
    }
}

pub mod pwm_mod {
    pub use super::clocks::Clocks;
    use stm32f103_pac::TIM2;

    pub struct Pwm {
        tim: TIM2,
    }

    impl Pwm {
        pub fn new(tim: TIM2) -> Self {
            Pwm {
                tim,
            }
        }

        pub fn configure(&mut self, clocks: &Clocks) {
            clocks.rcc.apb1enr.modify(|_, w| w
                                      .tim2en().set_bit()//enable timer 1
                                      );

            self.tim.psc.write(|w| unsafe { w.psc().bits(35) });//PSC = 36. FCLK = 1MHz. T = 1us
            self.tim.arr.write(|w| unsafe { w.arr().bits(20000) });//Set PWM freq at 50Hz. T = 20ms. ARR = 20000

            //CH 2 -> pa1
            self.tim.ccmr1_output().modify(|_, w| unsafe { w
                .cc2s().bits(0b00)//channel configured as output
                    .oc2m().bits(0b110)//channel cfg'd in pwm mode 1
                    //.oc2fe().set_bit()//enable o/p compare fast
                    .oc2pe().set_bit()//output compare preload enable
            });
            //CH 4 -> pa3
            self.tim.ccmr2_output().modify(|_, w| unsafe { w
                .cc4s().bits(0b00)//channel configured as output
                    .oc4m().bits(0b110)//channel cfg'd in pwm mode 1
                    //.oc3fe().set_bit()//enable o/p compare fast
                    .oc4pe().set_bit()//output compare preload enable
            });

            self.tim.cr1.modify(|_, w| w.arpe().set_bit());//auto reload preload enable

            self.tim.ccer.modify(|_, w| w
                                 .cc2p().clear_bit()//set output as active high
                                 .cc4p().clear_bit()//set output as active high
                                 .cc2e().set_bit()//capture compare output enable, CH 2
                                 .cc4e().set_bit()//capture compare output enable, CH 4
                                 );

            self.tim.egr.write(|w| w.ug().set_bit());//force update of registers
            self.tim.sr.modify(|_, w| w.uif().clear_bit());
        }

        pub fn enable(&mut self) {
            self.tim.cr1.modify(|_, w| w.cen().set_bit());//enable timer
        }

        pub fn disable(&mut self) {
            self.tim.cr1.modify(|_, w| w.cen().clear_bit());//disable timer
        }

        pub fn set_motor_duty(&mut self, duty: u16) {
            use micromath::F32Ext;

            let ccr: f32 = ((duty as f32/100.)*20000.).floor();
            self.tim.ccr2.modify(|_, w| unsafe { w
                .ccr2()
                    .bits(ccr as u16) });//ccr = (duty/100)*arr
        }

        pub fn set_servo_duty(&mut self, duty: u16) {
            use micromath::F32Ext;

            let ccr: f32 = ((duty as f32/100.)*20000.).floor();
            self.tim.ccr4.modify(|_, w| unsafe { w
                .ccr4()
                    .bits(ccr as u16) });//ccr = (duty/100)*arr
        }
    }
}

pub mod pins {
    use super::clocks::Clocks;
    use stm32f103_pac::{RCC, GPIOA, GPIOB};

    pub struct GPIOBPins {
        portb: GPIOB,
    }

    impl GPIOBPins {
        pub fn new(clocks: &Clocks, portb: GPIOB) -> Self {
            //Enable port b peripheral ie GPIOB
            clocks.rcc.apb2enr.modify(|_, w| w.iopben().set_bit());

            GPIOBPins { portb }
        }

        pub fn enable_trigger_pin(&mut self) {
            //Configure pin 10 as trigger output
            self.portb.crh
                .modify(|_, w| unsafe {
                    w.mode10().bits(0b10)//Output mode, max speed 2 MHz.
                        .cnf10().bits(0b01)//General purpose output push-pull
                });

            self.portb.odr.modify(|_, w| w.odr10().clear_bit());//Put OFF initially
        }

        pub fn trigger_toggle(&mut self) {
            self.portb.odr.modify(|r, w| w.odr10().bit(!r.odr10().bit()));
        }

        pub fn trigger_high(&mut self) {
            self.portb.odr.modify(|_, w| w.odr10().set_bit());
        }

        pub fn trigger_low(&mut self) {
            self.portb.odr.modify(|_, w| w.odr10().clear_bit());
        }
    }

    pub struct GPIOAPins {
        porta: GPIOA,
    }

    impl GPIOAPins {
        pub fn new(clocks: &Clocks, porta: GPIOA) -> Self {
            //Enable port a peripheral ie GPIOA
            clocks.rcc.apb2enr.modify(|_, w| w.iopaen().set_bit());

            GPIOAPins { porta }
        }

        pub fn enable_usart_pins(&self) {
            //Configure pa9 and pa10 as tx and rx pins respectively
            //Configure pin pa9 -> tx
            self.porta.crh.modify(|_, w| unsafe { w
                .mode9().bits(0b11)//Output mode, max speed 10MHz
                    .cnf9().bits(0b10)//Alternate function output, push-pull
            });

            //Configure pin pa10 -> rx
            self.porta.crh.modify(|_, w| unsafe { w
                .mode10().bits(0b00)//Input mode
                    .cnf10().bits(0b01)//Floating input, reset state
            });
        }

        pub fn enable_pwm_pins(&self) {
            //Configure pin pa1 -> motor drives.T2C2
            self.porta.crl.modify(|_, w| unsafe { w
                .mode1().bits(0b11)//Output mode, max speed 50MHz
                    .cnf1().bits(0b10)//Alternate function output, push-pull
            });


            //Configure pin pa3 -> servo.T2C4
            self.porta.crl.modify(|_, w| unsafe { w
                .mode3().bits(0b11)//Output mode, max speed 50MHz
                    .cnf3().bits(0b10)//Alternate function output, push-pull
            });
        }

        pub fn enable_echo_pin(&self) {
            //Configure pin 8 as t1c1
            self.porta.crh
                .modify(|_, w| unsafe {
                    w.mode8().bits(0b00)//Input mode.
                        .cnf8().bits(0b01)//Floating input (reset state)
                });
        }

        pub fn is_low(&self) -> bool {
            //Check whether echo pin 8 is low
            self.porta.odr.read().odr8().bit_is_clear()
        }

        pub fn is_high(&self) -> bool {
            //Check whether echo pin 8 is high
            self.porta.odr.read().odr8().bit_is_set()
        }

    }

    pub struct ShiftRegisterPins;
    
    impl ShiftRegisterPins {
        pub fn configure() -> Self {
            let rcc = unsafe { &(*RCC::ptr()) };
            rcc.apb2enr.modify(|_, w| w.iopben().set_bit());//enable clock

            let portb = unsafe { &(*GPIOB::ptr()) };
            
            //Configure pin 13 as latch pin
            portb.crh
                .modify(|_, w| unsafe {
                    w.mode13().bits(0b01)//Output mode, max speed 10 MHz.
                        .cnf13().bits(0b00)//General purpose output push-pull
                });

            portb.odr.modify(|_, w| w.odr13().clear_bit());//Put OFF initially

            //Configure pin 14 as data pin
            portb.crh
                .modify(|_, w| unsafe {
                    w.mode14().bits(0b01)//Output mode, max speed 10 MHz.
                        .cnf14().bits(0b00)//General purpose output push-pull
                });

            portb.odr.modify(|_, w| w.odr14().clear_bit());//Put OFF initially

            //Configure pin 12 as clock pin
            portb.crh
                .modify(|_, w| unsafe {
                    w.mode12().bits(0b01)//Output mode, max speed 10 MHz.
                        .cnf12().bits(0b00)//General purpose output push-pull
                });

            portb.odr.modify(|_, w| w.odr12().clear_bit());//Put OFF initially

            Self
        }

        pub fn latch_high() {
            let portb = unsafe { &(*GPIOB::ptr()) };
            portb.odr.modify(|_, w| w.odr13().set_bit());
        }

        pub fn latch_low() {
            let portb = unsafe { &(*GPIOB::ptr()) };
            portb.odr.modify(|_, w| w.odr13().clear_bit());
        }

        pub fn data_high() {
            let portb = unsafe { &(*GPIOB::ptr()) };
            portb.odr.modify(|_, w| w.odr14().set_bit());
        }

        pub fn data_low() {
            let portb = unsafe { &(*GPIOB::ptr()) };
            portb.odr.modify(|_, w| w.odr14().clear_bit());
        }

        pub fn clock_high() {
            let portb = unsafe { &(*GPIOB::ptr()) };
            portb.odr.modify(|_, w| w.odr12().set_bit());
        }

        pub fn clock_low() {
            let portb = unsafe { &(*GPIOB::ptr()) };
            portb.odr.modify(|_, w| w.odr12().clear_bit());
        }
    }
}

pub mod input_capture {
    use super::clocks::Clocks;
    use stm32f103_pac::TIM1;

    pub struct InputCapture {
        tim: TIM1,
    }

    impl InputCapture {
        pub fn configure(clocks: &Clocks, tim: TIM1) -> Self {
            //Enable clock to timer1
            clocks.rcc.apb2enr.modify(|_, w| w.tim1en().set_bit());   

            tim.psc.modify(|_, w| unsafe { w.psc().bits(71) });//psc = 72. Set fCLK to 1MHz, T = 1us
            tim.arr.modify(|_, w| unsafe { w.arr().bits(0xFFFF)});//Set arr to max
            tim.cr1.modify(|_, w| w.arpe().set_bit());//enable arr auto reload

            tim.cr1.modify(|_, w| w.urs().set_bit());//Only counter overflow/underflow generates an update interrupt
            tim.egr.write(|w| w.ug().set_bit());//Force update of registers
            tim.sr.modify(|_, w| w.uif().clear_bit());//Clear update flag
            //tim.cr1.modify(|_, w| w.urs().set_bit());//Only counter overflow/underflow generates an update interrupt

            tim.ccmr1_input().modify(|_, w| unsafe { w.cc1s().bits(0b01) });//CC1 channel is configured as input, IC1 is mapped on TI1
            tim.ccer.modify(|_, w| w
                            .cc1p().clear_bit()//non-inverted: capture is done on a rising edge of IC1
                            .cc1e().set_bit()//Capture enabled
                            );

            InputCapture { tim }
        }

        pub fn enable(&self) {
            self.tim.cr1.modify(|_, w| w.cen().set_bit());//enable counter
        }

        pub fn disable(&self) {
            self.tim.cr1.modify(|_, w| w.cen().clear_bit());//disable counter
        }

        pub fn switch_polarity(&self) {
            self.tim.ccer.modify(|r, w| w
                                 .cc1e().clear_bit()//first disable capture
                                 .cc1p().bit(!r.cc1p().bit())//switch polarity
                                 .cc1e().set_bit()//Capture enabled
                                 );
        }

        pub fn read_ccr(&self) -> u16 {
            self.tim.ccr1.read().ccr1().bits()
        }

        pub fn enable_cc1ie_interrupt(&self) {
            self.tim.dier.modify(|_, w| w.cc1ie().set_bit());//CC1 interrupt enabled
        }

        pub fn disable_cc1ie_interrupt(&self) {
            self.tim.dier.modify(|_, w| w.cc1ie().clear_bit());//CC1 interrupt disabled
        }

        pub fn is_capture(&self) -> bool {
            self.tim.sr.read().cc1if().bit()//checks if there's a capture
        }

        pub fn clear_capture(&self) {
            self.tim.sr.modify(|_, w| w.cc1if().clear_bit());//Can also be cleared by reading the ccr 
        }

        pub fn enable_update_interrupt(&self) {
            self.tim.dier.modify(|_, w| w.uie().set_bit());//Update interrupt enabled
        }

        pub fn disable_update_interrupt(&self) {
            self.tim.dier.modify(|_, w| w.uie().clear_bit());//Update interrupt disabled
        }

        pub fn is_overcapture(&self) -> bool {
            self.tim.sr.read().cc1of().bit()//checks if there's an overcapture
        }

        pub fn clear_overcapture(&self) {
            self.tim.sr.modify(|_, w| w.cc1of().clear_bit());//Clear overcapture flag
        }

        pub fn is_overflow(&self) -> bool {
            self.tim.sr.read().uif().bit()//checks if there's been an overflow update
        }

        pub fn clear_overflow(&self) {
            self.tim.sr.modify(|_, w| w.uif().clear_bit());//Clear overflow update flag
        }

        pub fn get_current_value(&self) -> u32 {
            self.tim.cnt.read().bits()
        }
    }
}

pub mod delay {
    use super::clocks::Clocks;
    use stm32f103_pac::{TIM3, TIM4};

    pub struct DelayMs;

    impl DelayMs {
        pub fn configure(clocks: &Clocks, tim: TIM4) {
            clocks.rcc.apb1enr.modify(|_, w| w.tim4en().set_bit());// Power on the TIM4 timer

            tim.cr1.modify(|_, w| w
                          .opm().set_bit()// OPM Select one pulse mode
                          .cen().clear_bit()// CEN Keep the counter disabled for now
                          //.arpe().set_bit()//enable arr auto reload
                          //.urs().set_bit()//Only counter overflow/underflow generates an update interrupt
                          );

            // Configure the prescaler to have the counter operate at 1 KHz
            // APB1_CLOCK = 36 MHz. TIM4CLK = 36 x 2 = 72MHz b/c APB1 PSC != 1
            // PSC = 7199
            // 72 MHz / (7199 + 1) = 10 KHz
            // The counter (CNT) will increase on every 10 milliseconds
            tim.psc.write(|w| unsafe { w.psc().bits(7199) });

            tim.egr.write(|w| w.ug().set_bit());//Force update of registers
            tim.sr.modify(|_, w| w.uif().clear_bit());//Clear update flag
        }

        #[inline(never)]
        pub fn delay_ms(ms: u16) {
            let tim = unsafe { &(*TIM4::ptr()) };//To enable use w/out ownership
            // Set the timer to go off in `ms` ticks
            // 1 tick = 10 ms
            tim.arr.write(|w| unsafe { w.arr().bits(10*ms) });//1ms = 1000us

            tim.cr1.modify(|_, w| w.cen().set_bit());// Enable the counter

            // Wait until the alarm goes off (until the update event occurs)
            while !tim.sr.read().uif().bit_is_set() {}

            tim.cr1.modify(|_, w| w.cen().clear_bit());//disable counter

            tim.sr.modify(|_, w| w.uif().clear_bit());// Clear the update event flag
        }
    }

    pub struct DelayUs;

    impl DelayUs {
        pub fn configure(clocks: &Clocks, tim: TIM3) {
            clocks.rcc.apb1enr.modify(|_, w| w.tim3en().set_bit());// Power on the TIM3

            tim.cr1.modify(|_, w| w
                          .opm().set_bit()// OPM Select one pulse mode
                          .cen().clear_bit()// CEN Keep the counter disabled for now
                          //.arpe().set_bit()//enable arr auto reload
                          //.urs().set_bit()//Only counter overflow/underflow generates an update interrupt
                          );

            // Configure the prescaler to have the counter operate at 1 KHz
            // APB1_CLOCK = 36 MHz. TIM4CLK = 36 x 2 = 72MHz b/c APB1 PSC != 1
            // PSC = 7199
            // 72 MHz / (71 + 1) = 1 MHz
            // The counter (CNT) will increase on every 1 microsecond
            tim.psc.write(|w| unsafe { w.psc().bits(71) });

            tim.egr.write(|w| w.ug().set_bit());//Force update of registers
            tim.sr.modify(|_, w| w.uif().clear_bit());//Clear update flag
        }

        #[inline(never)]
        pub fn delay_us(us: u16) {
            let tim = unsafe { &(*TIM3::ptr()) };//To enable use w/out ownership
            // Set the timer to go off in `ms` ticks
            // 1 tick = 1 us
            tim.arr.write(|w| unsafe { w.arr().bits(us) });

            tim.cr1.modify(|_, w| w.cen().set_bit());// Enable the counter

            // Wait until the alarm goes off (until the update event occurs)
            while !tim.sr.read().uif().bit_is_set() {}

            tim.cr1.modify(|_, w| w.cen().clear_bit());//disable counter

            tim.sr.modify(|_, w| w.uif().clear_bit());// Clear the update event flag
        }
    }
}

pub mod functions {
    use super::{Data, delay::DelayMs, pins::ShiftRegisterPins};
    use rtt_target::rprintln;
    use super::{MOVING_FORWARD, Command::{self, Forward, Reverse, RightTurn, LeftTurn, Brake, Stop, Donut}};

    pub fn drive_motors(command: &Command) {
        let mut data;
        //[0,   1,   2,    3,    4,   5,   6,   7]
        //[BR2, BR1, FR2, FR1, BL2, BL1, FR2, FR1]

        match command {
            Forward => {
                data = [1, 0, 1, 0, 1, 0, 1, 0];
                update_shift_register(data);
                rprintln!("forward...");
            },
            _ => {
                unsafe { MOVING_FORWARD = false; }

                match command {
                    Reverse => {
                        data = [0, 1, 0, 1, 0, 1, 0, 1];
                        update_shift_register(data);
                        rprintln!("reverse...");
                    },
                    RightTurn => { 
                        data = [0, 1, 0, 1, 1, 0, 1, 0];
                        update_shift_register(data);
                        rprintln!("right turn...");
                        DelayMs::delay_ms(250);//wait until turn finished
                        data = [0, 0, 0, 0, 0, 0, 0, 0];
                        update_shift_register(data);//reset shift register
                    },
                    LeftTurn => {
                        data = [1, 0, 1, 0, 0, 1, 0, 1];
                        update_shift_register(data);
                        rprintln!("left turn...");
                        DelayMs::delay_ms(250);//wait until turn finished
                        data = [0, 0, 0, 0, 0, 0, 0, 0];
                        update_shift_register(data);//reset shift register
                    },
                    Brake => { 
                        data = [0, 1, 0, 1, 0, 1, 0, 1];
                        update_shift_register(data);//hard reverse
                        rprintln!("brake...");
                        DelayMs::delay_ms(200);//wait until finished
                        data = [0, 0, 0, 0, 0, 0, 0, 0];
                        update_shift_register(data);//reset shift register
                    },
                    Stop => {
                        data = [0, 0, 0, 0, 0, 0, 0, 0];
                        update_shift_register(data);//reset shift register
                        rprintln!("stop...");
                    },
                    Donut => {
                        data = [0, 1, 0, 1, 1, 0, 1, 0];
                        update_shift_register(data);
                        rprintln!("Donut...");
                        DelayMs::delay_ms(2000);//wait until donut finished
                        data = [0, 0, 0, 0, 0, 0, 0, 0];
                        update_shift_register(data);//reset shift register
                    },
                    _ => { },
                }
            },
        }
    }

    pub fn update_shift_register(data: Data) {
        ShiftRegisterPins::latch_low();

        //Send data to SER
        for byte in data.into_iter().rev() {
            if byte == 1 {
                ShiftRegisterPins::data_low();
            } else {
                ShiftRegisterPins::data_high();
            }

            ShiftRegisterPins::clock_high();
            DelayMs::delay_ms(5);//5ms delay
            ShiftRegisterPins::clock_low();
        }

        ShiftRegisterPins::latch_high();
    }
}

pub enum EchoStatus {
    IDLE,
    DONE,
}

#[derive(Debug)]
pub enum Command {
    Forward,
    Reverse,
    RightTurn,
    LeftTurn,
    Brake,
    Stop,
    Donut,
}

pub enum ServoDirection {
    Right,
    Left,
}

#[derive(Debug)]
pub enum UltrasonicPosition {
    Right,
    Left,
    Middle
}

