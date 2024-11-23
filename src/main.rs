//!rtic rover with bluetooth-usart control
//!ultrasonic sensor for obstacle avoidance
//!runs in auto and manual modes

#![no_main]
#![no_std]
#![deny(warnings)]
#![deny(missing_docs)]
#![feature(type_alias_impl_trait)]
#![allow(clippy::needless_if)]

use panic_rtt_target as _;
use rtt_target::{rprint, rprintln, rtt_init_print};
use rtic_monotonics::systick::{ExtU32, Systick};
use obstacle_avoiding_rover::{
    pac, clocks, led, usart1, pwm_mod, MOVING_FORWARD,
    input_capture::InputCapture, 
    pins::{GPIOAPins, GPIOBPins, ShiftRegisterPins}, EchoStatus::{self, IDLE, DONE}, delay::{DelayMs, DelayUs},
    Command::{self, Forward, Reverse, RightTurn, LeftTurn, Brake, Stop, Donut}, UltrasonicPosition::{self, Right, Left, Middle}, 
    functions::drive_motors
};

use micromath::F32Ext;

const D_STOP: u32 = 20;

#[rtic::app(device = pac, peripherals = true, dispatchers = [USART2, TIM2])]
mod app {
    use super::*;

    #[shared]
    struct Shared {
        command: Option<Command>,
        auto: bool,
        ov_cnt: u32,//overcount
        distance: Option<u32>,
        ic: InputCapture,
    }

    #[local]
    struct Local {
        led: led::Led,
        usart: usart1::Usart1,
        trigger: GPIOBPins,
        echo_status: EchoStatus,
        pwm: pwm_mod::Pwm,
        t1: u32,
        t2: u32,
        ultrasonic_pos: UltrasonicPosition,
    }

    #[init]
    fn init(cx: init::Context) -> (Shared, Local) {
        //Initialise clocks
        let mut clocks = clocks::Clocks::new(cx.device.RCC, cx.device.FLASH);
        clocks.configure();

        //Initialize the systick interrupt & obtain the token to prove that we did
        let systick_mono_token = rtic_monotonics::create_systick_token!();
        Systick::start(cx.core.SYST, 72_000_000, systick_mono_token); // default STM32F103

        //Enable pins
        //Usart Echo & Pwm
        let gpioa_pins = GPIOAPins::new(&clocks, cx.device.GPIOA);
        gpioa_pins.enable_echo_pin();//enable echo pin
        gpioa_pins.enable_usart_pins();
        gpioa_pins.enable_pwm_pins();

        //Ultrasonic pins
        let mut gpiob_pins = GPIOBPins::new(&clocks, cx.device.GPIOB);
        gpiob_pins.enable_trigger_pin();//enable trigger pin
        let trigger = gpiob_pins;//take over gpiob pins handle

        //Input capture
        let ic = InputCapture::configure(&clocks, cx.device.TIM1);
        ic.enable_cc1ie_interrupt();
        ic.enable_update_interrupt();

        //Led handle
        let led = led::Led::new(&clocks, cx.device.GPIOC);

        //Usart handle
        let mut usart = usart1::Usart1::config(&clocks, cx.device.USART1);
        usart.enable_interrupt();//enable interrupt

        //Configure delay
        DelayMs::configure(&clocks, cx.device.TIM4);

        DelayUs::configure(&clocks, cx.device.TIM3);

        //Pwm handle
        let mut pwm = pwm_mod::Pwm::new(cx.device.TIM2);
        pwm.configure(&clocks);
        pwm.enable();

        pwm.set_servo_duty(15);//initialize servo at Middle pos
        pwm.set_motor_duty(100);//motors to run at full speed

        //Shift Register pins configuration
        ShiftRegisterPins::configure();
        ShiftRegisterPins::latch_low();
        ShiftRegisterPins::clock_low();
        ShiftRegisterPins::data_low();

        control::spawn().unwrap();

        rtt_init_print!();
        rprintln!("init");

        (
            Shared {
                command: None,
                auto: false,
                ov_cnt: 0,
                distance: None,
                ic,
            },

            Local {
                led,
                usart,
                trigger,
                echo_status: IDLE,
                pwm,
                t1: 0,
                t2: 0,
                ultrasonic_pos: Middle,
            },
        )
    }

    #[idle]
    fn idle(_: idle::Context) -> ! {
        loop {
            cortex_m::asm::nop();
        }
    }

    #[task(binds = USART1, local = [usart, led], shared = [auto, command], priority = 3)]
    fn receive_command(cx: receive_command::Context) {
        rprintln!("command task started");
        let auto = cx.shared.auto;
        let command = cx.shared.command;
        let usart = cx.local.usart;
        let led = cx.local.led;

        usart.disable_interrupt();//disable interrupts until finished

        //read and loopback
        let byte = usart.receive();
        
        (auto, command).lock(|auto, command| {
            match byte {
                0x41 => {
                    if *auto {
                        *auto = false;
                        led.off();//indication led
                        *command = Some(Brake);//turning from auto, brake to stop
                    } else {
                        *auto = true;//change to auto
                        led.on();
                    }
                },
                _ => {
                    if *auto {
                        //take no command if in auto mode
                    } else {
                        match byte {
                            //set commands according to received
                            //byte
                            0x42 => *command = Some(Forward),
                            0x43 => *command = Some(Reverse),
                            0x44 => *command = Some(RightTurn),
                            0x45 => *command = Some(LeftTurn),
                            0x46 => *command = Some(Brake),
                            0x47 => *command = Some(Stop),
                            0x48 => *command = Some(Donut),
                            _ => {},
                        }
                    }
                },
            }
        });

        usart.transmit(byte);//loop back command byte 
        usart.enable_interrupt();//enable interrupts after finished
    }

    #[task(local = [pwm, ultrasonic_pos], shared = [auto, command, distance], priority = 1)]
    async fn control(cx: control::Context) {
        rprintln!("control task started");
        let mut auto = cx.shared.auto;
        let mut command = cx.shared.command;
        let mut distance = cx.shared.distance;
        let pwm = cx.local.pwm;
        let us_pos = cx.local.ultrasonic_pos;

        let mut dr = 0;//distance in the right direction
        let mut dl = 0;//distance in the left direction

        drive_motors(&Stop);//start from stop position
        
        loop {
            auto.lock(|auto| {
                if *auto {
                    if trigger::spawn().is_err() {} 
                    (distance).lock(|distance| {
                        while let Some(d) = distance {
                            rprintln!("{}", d);

                            match us_pos {
                                Right => {
                                    dr = distance.unwrap();//get dr
                                    rprintln!("right distance: {}", dr);
                                    pwm.set_servo_duty(25);//position us left
                                    rprintln!("moving us to the left");
                                    DelayMs::delay_ms(1000);
                                    *us_pos = Left;
                                },
                                Left => {
                                    dl = distance.unwrap();//get dl
                                    rprintln!("left distance: {}", dl);
                                    pwm.set_servo_duty(15);//return us postion to middle
                                    rprintln!("moving us to the middle");
                                    DelayMs::delay_ms(1000);
                                    *us_pos = Middle;
                                    //compare dr & dl; take required action
                                    if (dr > D_STOP) | (dl > D_STOP) {
                                        if dr > dl {
                                            rprintln!("turn rover right");
                                            drive_motors(&RightTurn);//...turn motors right
                                        } else {
                                            rprintln!("turn rover left");
                                            drive_motors(&LeftTurn);//...turn motors left
                                        }
                                    } else {
                                        rprintln!("reverse");
                                        drive_motors(&Reverse);//...reverse motors
                                        rprintln!("move rover right");
                                        drive_motors(&RightTurn);//...turn motors right
                                    }

                                    DelayMs::delay_ms(500);//delay a little
                                },
                                Middle => {
                                    if distance.unwrap() <= D_STOP {
                                        rprintln!("distance < {}", D_STOP);
                                        drive_motors(&Brake);//...brake motors
                                        pwm.set_servo_duty(5);//position ultrasonic to the right
                                        rprintln!("moving us to the right");
                                        DelayMs::delay_ms(1000);
                                        *us_pos = Right;
                                    } else {
                                        rprintln!("distance > {}", D_STOP);
                                        unsafe {
                                            if !MOVING_FORWARD {
                                                drive_motors(&Forward);//...drive motors forward
                                                MOVING_FORWARD = true;
                                            }
                                        }
                                    }
                                },
                            }
                            *distance = None;
                        }
                    });                    
                } else {
                    //manual
                    command.lock(|command| {
                        while let Some(c) = command {
                            rprintln!("driving motor {:?}", c );

                            drive_motors(c);

                            *command = None;//update command to None
                        }
                    });
                }
            });
        }
    }


    #[task(local = [trigger], shared = [distance], priority = 2)]
    async fn trigger(cx: trigger::Context) {
        rprintln!("trigger task started");
        let trigger = cx.local.trigger;
        let mut _distance = cx.shared.distance;

        trigger.trigger_low();
        DelayUs::delay_us(2);

        trigger.trigger_high();
        DelayUs::delay_us(10);

        trigger.trigger_low();

        Systick::delay(200.millis()).await;
    }

    #[task(binds = TIM1_UP, shared = [ic, ov_cnt], priority = 4)]
    fn overflow(cx: overflow::Context) {
        let ic = cx.shared.ic;
        let ov_cnt = cx.shared.ov_cnt;

        (ic, ov_cnt).lock(|ic, ov_cnt| {
            *ov_cnt += 1;//increment overflow flag

            if *ov_cnt == u32::MAX {//this may clear ov_cnt before its used. => if in Done state ??
                *ov_cnt = 0;
            }

            ic.clear_overflow();
        });
    }

    #[task(binds = TIM1_CC, shared = [ic, ov_cnt, distance], local = [echo_status, t1, t2], priority = 3)]
    fn time_capture(cx: time_capture::Context) {
        rprint!("time:\t");
        let mut ic = cx.shared.ic;
        let status = cx.local.echo_status;
        let t1 = cx.local.t1;
        let t2 = cx.local.t2;
        let mut ov_cnt = cx.shared.ov_cnt;
        let mut distance = cx.shared.distance;

        (distance).lock(|distance| {
            match status {
                IDLE => {
                    rprintln!("idle");
                    ic.lock(|ic| {
                        *t1 = u32::from(ic.read_ccr());//capture t1
                        ic.switch_polarity();//toggle polarity
                        ic.enable();
                    });
                    ov_cnt.lock(|ov_cnt| *ov_cnt = 0);//clear overcapture
                    *status = DONE;//update status to done
                },
                DONE => {
                    rprintln!("done");
                    ic.lock(|ic| {
                        *t2 = u32::from(ic.read_ccr());//capture t2
                        ic.switch_polarity();//toggle polarity
                        ic.disable();
                    });
                    *status = IDLE;//update status to idle
                    let mut OF = 0;

                    ov_cnt.lock(|ov_cnt| OF = *ov_cnt );
                    let T = *t2 + OF*(65535) - *t1;
                    let d = (T as f32)*(0.034/2.);
                    *distance = Some(d.floor() as u32);
                    rprintln!("{}", distance.unwrap());
                },
            }
        });
    }
}





