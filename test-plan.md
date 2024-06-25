Temperature Monitor Hardware Testing Proposal
=============================================

Objective
---------

Automatically detect failures in a microcontroller-driven temperature sensor.
Enable rapid updates to the Temperature Monitor and test rig with reasonable
confidence that obvious defects will be detected.

The most likely sources of defects
   - Updates to microcontroller firmware
   - Updates to libraries the microcontroller firmware depends on
   - Updates to the test rig software/firmware
   - Updates to the test rig hardware
   - Faulty wiring/connectors

Qualifying the sensor itself is out of scope.

Assumptions
-----------

The test rig should be as simple and inexpensive as possible while exercising
the range of behavior of the device under test (DUT).

The test rig is in an air-conditioned office environment well within the
range of the sensor.

The sensor is not subtly defective, e.g.
   - The sensor is poorly calibrated
   - The sensor generates errors for no apparent reason, but only rarely

Test Rig
--------

   - Microcontroller
   - Visible light photodiode
   - Temperature sensor of a different model than the DUT
   - Low-voltage appliance fuse used as a heater
   - Transistor circuit for controlling power to heater
   - Circuit board with pogo pins for monitoring the DUT outputs
   - A fire resistant surface (almost certainly not needed)
   - A computer with at least two USB-A ports

Concept of Operations
---------------------

By applying a little heat to the sensor, it should be possible to observe the
temperature readings ramp up, and then by withdrawing the heat, it should be
possible to see the readings ramp down.

Elements of the test rig function as follows
   - The fuse is used to apply heat directly to the DUT sensor
   - The test microcontroller is able to sense the temperature of the heater
   - By using a fuse as the heating element, the risk of starting a fire is
     reduced, but fuses are typically not terribly accurate, so there remains a
     risk of overheating and damaging the sensor
   - The test microcontroller is able to monitor the signals of the temperature
     sensor
   - Due to the low thermal mass of the system, allowing the system to cool
     passively is fast enough
   - The temperature threshold is initially set hotter a few degrees than the
      ambient temperature and noise margin of the sensor by 
   - The test microcontroller is able to detect light from the DUT
    microcontroller's LED
   - The test microcontroller is able to communicate over USB-Serial to the
     computer
   - The test microcontroller is able to monitor the signals of the DUT
     microcontroller

The testing proceeds as follows
1. The computer commands the test microcontroller to report the status of the
DUT LED. The LED will be off initially.

2. Until the test microcontroller reports the DUT LED has turned ON or the
threshold would be below a reasonable lower limit, the computer repeatedly
commands test microcontroller to decrease the threshold by sending a signal to
the down button input of the DUT.

3. Until the test microcontroller reports the DUT LED has turned OFF or the
threshold would be above the reasonable upper limit, the computer repeatedly
commands the test microcontroller to increase the threshold by sending a
singal to the up button input of the DUT.

4. The computer commands the test microcontroller to increase the threshold by
a few degrees and commands the microcontroller to verify that the LED is still
OFF.

5. The computer commands the test microcontroller to apply power to the heater
for a small fixed time interval and report the status of the LED, and its own
temperature measurement of the heater. The computer monitors the serial
output of both the test microcontroller and the DUT looking for a warming
trend. If the warming trend is detected and the LED turns on at approximately
the nominal threshold temperature, no more heat is applied.

6. The computer
monitors the reported temperature looking for a cooling trend. If all the
previous conditions were satisfied, the test PASSES.

7. If at any point, the DUT stops responding, reports a temperature out of a
reasonable range, or lights the LED contradicting the threshold, the test FAILS.


Testing Requirements
--------------------

**RIG-AMB1**: Test rig should work across reasonable ambient temperature range

**RIG-AMB2**: Test rig should fail self-test if ambient is out of range

**RIG-PWR1**: Test rig should fail self-test if input power is out of range 

**RIG-PWR2**: Test rig should monitor that the heater is powered for the designated
time (self-monitoring)

**RIG-PWR3**: Test rig should monitor that the heater temperature never exceeds
50 ℃ and thus is safe to touch

**RIG-PWR4**: Test rig should be able to increase the heater temperature at least
10 ℃ over the ambient temperature

**RIG-LED1**: Test rig should sense DUT status LED output pin

**RIG-LED2**: Test rig should sense DUT status LED illumination

**RIG-I2C1**: Test rig should monitor all data transmitted to temperature sensor
and verify it matches fixed expected steam

**RIG-I2C2**: Test rig should monitor all data transmitted to temperature sensor
and verify it matches the expected transmission rate

**RIG-T1**: Test rig should monitor all data transmitted by the DUT temperature
sensor and verify that temperature remains within the valid range

**RIG-T2**: The test rig should verify that serial output of the DUT approximately
matches the temperature sensor of the test rig

**RIG-T3**: Test rig should monitor all data transmitted by the DUT temperature
sensor and verify that temperature readings are reasonably stable

**RIG-BTN1**: The test rig should simulate bounce on the push button contacts and
verify that the simulated button presses are interpreted correctly

**RIG-RAMP1**: Test rig should monitor that the ramp increases past the threshold
and decrease over an interval corresponding to the heater

**RIG-RAMP2**: Test rig should monitor that the temperature after the ramp flattens
out to close enough to starting conditions

**RIG-RAMP3**: Test rig should monitor that the ramp is sufficiently smooth

Notes
-----

Initially, requirements related to monitoring signals from the DUT
microcontroller and the DUT temperature sensor can be skipped because it is
unlikely that the serial monitor output or LED output would be correct if the
signals were incorrect.

The requirements are high-level and omit precise specifications in the interest
of brevity. The DUT code should reference the requirement IDs here and document
the choices made.

Consider tests that give a rough idea of how quickly the temperature sensor
reacts to temperature changes. Vary the duty cycle of the heater and look for
proportionally lower change in temperature, and ripple in temperature readings
corresponding to the duty cycle, up to the frequency response of the sensor
(might be limited by the thermal mass of the heater).

Consider a non-contact temperature sensor to monitor the heater so that if
the heater malfunctions, at least one of the sensors will remain unharmed.

Consider a PID controller to hold the temperature at a given point to see
if the DUT temperature reading are stable.

Consider a manual test with a frozen gel pack and a bit of metal in boiling
water to see if the system works across a broader temperature range.

Consider how to test that the microcontroller panic handler works as intended.
