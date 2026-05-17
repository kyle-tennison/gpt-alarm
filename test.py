import Jetson.GPIO as GPIO
import time

PIN_NO = 7 # buzzer
# PIN_NO = 15 # light

# Use physical 40-pin board numbering
GPIO.setmode(GPIO.BOARD)

# Initialize physical Pin 7 (GPIO09) as an output
GPIO.setup(PIN_NO, GPIO.OUT)

# Set the pin High (3.3V)
GPIO.output(PIN_NO, GPIO.HIGH)

time.sleep(1)

# Set the pin Low (0V)
GPIO.output(PIN_NO, GPIO.LOW)

# Release the pin back to its default state
GPIO.cleanup()