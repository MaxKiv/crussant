ToDo
====

General
--- 
[x] update README
[ ] update connections jpeg

Main task
--- 
[x] quickly jump to a fallible main to be able to use ? operator

Blink task
--- 
[ ] use PWM peripheral to dim the LED

Sensor task
--- 
[x] Generate a random sample
[x] add hdc1080 driver
[x] add ccs881 driver
[x] share i2c bus between drivers
[ ] Write an embedded-hal 1.0.0 async capable driver for the humidity sensor?

Display task
---
[x] Log received samples
[x] Print something to waveshare using claudio's driver
[x] Get & show sensor measurement
[ ] Look into embedded_graphics & claudio's dashboard
[ ] Look into the display driver: why does updating the display block the executor?

Sleep
--- 
[ ] Add light sleep, only measure once every 30 seconds
[ ] Add sleep, only measure once every 30 seconds
[ ] Save boot or sleep count in rtc fast memory
[ ] Add deep sleep, only measure once every 30 seconds

Clock
---
[x] Inject walltime on compilation
[ ] Save clock in RTC fast memory so it survives deep sleep
[ ] Get clock time from the web, see claudio

BootCount
---
[ ] Track bootcount in non-volatile memory

