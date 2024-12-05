# Obstacle Avoiding Rover
## Table of Contents
- [Introduction](#intro)
- [Requirements](#reqs)
- [Commands](#commands)
- [Electrical Schematic](#schematic)
- [PCB](#pcb)
- [Code](#code)
- [Results](#results)
- [Recommendations](#recomm)

<a id="intro"></a>
## Introduction

This is a simple project that uses Rust's [RTIC](https://rtic.rs/2/book/en/) framework to create a 4WD rover that can avoid obstacles. It is implemented on the STM32F103 board. In orer to reduce the number of control pins used by the microcontroller in motor control, a shift register is used together with two L293D motor drivers. An HC-SR04 ultrasonic sensor module mounted on a servo motor for actuation detects the obstacles. Remote communication with the system is achieved using an HC-06 Bluetooth module.

`Probe-rs` is used for debugging and flashing of the application program. You can learn more about it [here](https://probe.rs/).

<a id="reqs"></a>
## Requirements
- 1 x STM32F103 board
- 1 x STLINK for programming the microcontroller board
- 1 x HC-SR04 Ultrasonic Sensor
- 1 x Mounting Bracket for the HC-SR04 Ultrasonic Sensor
- 1 x SG90 Servo Motor
- 2 x L293D Motor Driver chips
- 1 x 74HC565N Shift Register
- 2 x Li-ion Batteries. Any other sufficient supply can work. I personally used 2 x 7800 mAh 4.2V Li-ion batteries.
- 1 x Adjustable 5A Buck Voltage Regulator Module 
- 1 x 4 Motor 4WD Rover Chassis
- 8 x F - F Dupont Connector Wires
- 14 x M - M Dupont Connector Wires
- 1 x Battery Holder
- 1 x Diode
- 1 x SPDT Switch
- 7 x Screw Terminal Block Connectors 
- 3 x 16 DIP IC Sockets
- 4 x PCB Standoff Separators
- [Serial Bluetooth Terminal App](https://play.google.com/store/apps/details?id=de.kai_morich.serial_bluetooth_terminal&hl=en&pli=1)

<a id="commands"></a>
## Commands

| Command | Description |
|---------|-------------|
| A	| Auto/Manual	|
| B	| Forward	|
| C	| Reverse	|
| D	| Right Turn	|
| E	| Left Turn	|
| F	| Brake		|
| G	| Stop 		|
| H	| Donut		|

To send these remote commands we'd have to set up the serial Bluetooth App. 

Navigate to settings and under `Newline` select `None`.

<p align="center">
  <img alt="command-setup1" src="https://github.com/user-attachments/assets/234c036f-3960-4e1d-9882-cd95389b322d" width="185" height="400">
</p>

We will also allow local echo to help us visulaize the commands we are sending.

<p align="center">
  <img alt="command-setup2" src="https://github.com/user-attachments/assets/d7626d8a-df83-45af-b6fa-3e9864dcc33e" width="185" height="400">
</p>

Edit the macro rows at the bottom to include the commands discussed above.

<p align="center">
  <img alt="macro-setup" src="https://github.com/user-attachments/assets/8407048a-e463-4635-93e2-ce2748800bd0" width="185" height="400">
</p>

<a id="schematic"></a>
## Electrical Schematic

The below schematic was prepared using Kicad.

<p align="center">
  <img alt="electrical-schematic" src="https://github.com/user-attachments/assets/519f7154-acd1-49a9-b63b-990da9e78a59" width="600" height="600">
</p>

// schematic files

<a id="pcb"></a>
## PCB

The PCB was also produced from the above schematic using Kicad.
<p align="center">
  <img alt="pcb" src="https://github.com/user-attachments/assets/dc0ef893-e01c-43ff-99ba-b0b4506cf72c" width="400" height="300">
</p>

<p align="center">
  <img alt="pcb-3d1" src="https://github.com/user-attachments/assets/f5ce5877-eb29-4cec-9fa2-5dec548f7bd5" width="400" height="300">
</p>

<p align="center">
  <img alt="pcb-3d2" src="https://github.com/user-attachments/assets/978d3c4e-fb1c-47c5-92da-66fb3bcdd326" width="400" height="300">
</p>

// pcb files

// gerber files

<a id="code"></a>
## Code

The application code can be found [here](https://github.com/ian-ndeda/obstacle-avoiding-rover/blob/main/src/main.rs).

<a id="results"></a>
## Results

Below is the final set-up of the entire thing.


<p align="center">
  <img alt="setup1" src="https://github.com/user-attachments/assets/473adee6-2f33-4f78-ad40-8c40e810ccc5" width="356" height="200">
</p>

<p align="center">
  <img alt="setup2" src="https://github.com/user-attachments/assets/80a16392-25ef-45ce-a39f-6c01fb4c9603" width="356" height="200">
</p>

<p align="center">
  <img alt="setup3" src="https://github.com/user-attachments/assets/adc9c5d7-32b7-412a-b592-eb0103a0b2b2" width="356" height="200">
</p>

After flashing the program into the STM32F103 the console will be as shown below every time a command is given.

>:exclamation: Note that the program is in debug mode. You therefore cannot flash it in `--release` mode.


<p align="center">
  <img alt="console-probe" src="https://github.com/user-attachments/assets/57e12db0-d576-4821-97e0-ecb1c16efb24" width="500" height="500">
</p>

A little demonstration of the project.

<p align="center">
  <img alt="rover-demo" src="https://github.com/user-attachments/assets/aaf069bd-b3e1-4881-ad45-ce7413774aa2" width="200" height="356">
</p>

A little celebration is in order.

<p align="center">
  <img alt="rover-donut" src="https://github.com/user-attachments/assets/b2419794-e499-470f-9c48-1af0dd6c33f6" width="200" height="200">
</p>

<a id="recomm"></a>
## Recommendations

Possible improvements:
- Program some speed control into the application code. One could also explore hardware solutions.
- The HRS04 ultrasonic sensor is quite accurate. It however has some challanges. 
	- First, it's readings are affected by temperature and humidity. One could solve this by adding a temperature and humidity sensor to modulate the distance values for this.
	- The HRS04 can also give wrong reckonings when the obstacle is at an angle to its ine of sight or is made of 'fuzzy' material.  Other distance measuring methods can be exploited to correct or this. A possible candidate is the VL53L0X Time-of-Flight Sensor. 
- Create a handheld console for remote control insted of using the serial app.
- Mount [this](https://github.com/ian-ndeda/self-aligning-sat-dish/blob/main/README.md) self-aligning dish on to the rover and connect them electrically.  # Obstacle Avoiding Rover
## Table of Contents
- [Introduction](#intro)
- [Requirements](#reqs)
- [Commands](#commands)
- [Electrical Schematic](#schematic)
- [PCB](#pcb)
- [Code](#code)
- [Results](#results)
- [Recommendations](#rcomm)

<a id="intro"></a>
## Introduction

This is a simple project that uses Rust's [RTIC](https://rtic.rs/2/book/en/) framework to create a 4WD rover that can avoid obstacles. It is implemented on the STM32F103 board. In orer to reduce the number of control pins used by the microcontroller in motor control, a shift register is used together with two L293D motor drivers. An HC-SR04 ultrasonic sensor module mounted on a servo motor for actuation detects the obstacles. Remote communication with the system is achieved using an HC-06 Bluetooth module.

`Probe-rs` is used for debugging and flashing of the application program. You can learn more about it [here](https://probe.rs/).

<a id="reqs"></a>
## Requirements
- 1 x STM32F103 board
- 1 x STLINK for programming the microcontroller board
- 1 x HC-SR04 Ultrasonic Sensor
- 1 x Mounting Bracket for the HC-SR04 Ultrasonic Sensor
- 1 x SG90 Servo Motor
- 2 x L293D Motor Driver chips
- 1 x 74HC565N Shift Register
- 2 x Li-ion Batteries. Any other sufficient supply can work. I personally used 2 x 7800 mAh 4.2V Li-ion batteries.
- 1 x Adjustable 5A Buck Voltage Regulator Module 
- 1 x 4 Motor 4WD Rover Chassis
- 8 x F - F Dupont Connector Wires
- 14 x M - M Dupont Connector Wires
- 1 x Battery Holder
- 1 x Diode
- 1 x SPDT Switch
- 7 x Screw Terminal Block Connectors 
- 3 x 16 DIP IC Sockets
- 4 x PCB Standoff Separators
- [Serial Bluetooth Terminal App](https://play.google.com/store/apps/details?id=de.kai_morich.serial_bluetooth_terminal&hl=en&pli=1)

<a id="commands"></a>
## Commands

| Command | Description |
|---------|-------------|
| A	| Auto/Manual	|
| B	| Forward	|
| C	| Reverse	|
| D	| Right Turn	|
| E	| Left Turn	|
| F	| Brake		|
| G	| Stop 		|
| H	| Donut		|

To send these remote commands we'd have to set up the serial Bluetooth App. 

Navigate to settings and under `Newline` select `None`.

<p align="center">
  <img alt="command-setup1" src="https://github.com/user-attachments/assets/234c036f-3960-4e1d-9882-cd95389b322d" width="185" height="400">
</p>

We will also allow local echo to help us visulaize the commands we are sending.

<p align="center">
  <img alt="command-setup2" src="https://github.com/user-attachments/assets/d7626d8a-df83-45af-b6fa-3e9864dcc33e" width="185" height="400">
</p>

Edit the macro rows at the bottom to include the commands discussed above.

<p align="center">
  <img alt="macro-setup" src="https://github.com/user-attachments/assets/8407048a-e463-4635-93e2-ce2748800bd0" width="185" height="400">
</p>

<a id="schematic"></a>
## Electrical Schematic

The below schematic was prepared using Kicad.

<p align="center">
  <img alt="electrical-schematic" src="https://github.com/user-attachments/assets/519f7154-acd1-49a9-b63b-990da9e78a59" width="600" height="600">
</p>

[Here](https://drive.google.com/file/d/1DhmXBWXqDYcyfgzvpXEiL7pExS6LLZpt/view?usp=sharing) are the schematic files. 

<a id="pcb"></a>
## PCB

The PCB was also produced from the above schematic using Kicad.
<p align="center">
  <img alt="pcb" src="https://github.com/user-attachments/assets/dc0ef893-e01c-43ff-99ba-b0b4506cf72c" width="400" height="300">
</p>

<p align="center">
  <img alt="pcb-3d1" src="https://github.com/user-attachments/assets/f5ce5877-eb29-4cec-9fa2-5dec548f7bd5" width="400" height="300">
</p>

<p align="center">
  <img alt="pcb-3d2" src="https://github.com/user-attachments/assets/978d3c4e-fb1c-47c5-92da-66fb3bcdd326" width="400" height="300">
</p>

The PCB and gerber files are [here](https://drive.google.com/file/d/1szpY48ynfYvcWlL3J9hRI3lrKwgi_NbI/view?usp=sharing) and [here](https://drive.google.com/file/d/1gogWauziaE12DxzuEed2dPWoAJ4hTE9W/view?usp=sharing) respectively.

<a id="code"></a>
## Code

The application code can be found [here](https://github.com/ian-ndeda/obstacle-avoiding-rover/blob/main/src/main.rs).

<a id="results"></a>
## Results

Below is the final set-up of the entire thing.


<p align="center">
  <img alt="setup1" src="https://github.com/user-attachments/assets/473adee6-2f33-4f78-ad40-8c40e810ccc5" width="356" height="200">
</p>

<p align="center">
  <img alt="setup2" src="https://github.com/user-attachments/assets/80a16392-25ef-45ce-a39f-6c01fb4c9603" width="356" height="200">
</p>

<p align="center">
  <img alt="setup3" src="https://github.com/user-attachments/assets/adc9c5d7-32b7-412a-b592-eb0103a0b2b2" width="356" height="200">
</p>

After flashing the program into the STM32F103 the console will be as shown below every time a command is given.

>:exclamation: Note that the program is in debug mode. You therefore cannot flash it in `--release` mode.


<p align="center">
  <img alt="console-probe" src="https://github.com/user-attachments/assets/57e12db0-d576-4821-97e0-ecb1c16efb24" width="500" height="500">
</p>

A little demonstration of the project.

<p align="center">
  <img alt="rover-demo" src="https://github.com/user-attachments/assets/aaf069bd-b3e1-4881-ad45-ce7413774aa2" width="250" height="445">
</p>

A little celebration is in order.

<p align="center">
  <img alt="rover-donut" src="https://github.com/user-attachments/assets/b2419794-e499-470f-9c48-1af0dd6c33f6" width="250" height="250">
</p>

<a id="recomm"></a>
## Recommendations

Possible improvements:
- Program some speed control into the application code. One could also explore hardware solutions.
- The HRS04 ultrasonic sensor is quite accurate. It however has some challanges. 
	- First, it's readings are affected by temperature and humidity. One could solve this by adding a temperature and humidity sensor to modulate the distance values for this.
	- The HRS04 can also give wrong reckonings when the obstacle is at an angle to its ine of sight or is made of 'fuzzy' material.  Other distance measuring methods can be exploited to correct or this. A possible candidate is the VL53L0X Time-of-Flight Sensor. 
- Create a handheld console for remote control insted of using the serial app.
- Mount [this](https://github.com/ian-ndeda/self-aligning-sat-dish/blob/main/README.md) self-aligning dish on to the rover and connect them electrically.  
