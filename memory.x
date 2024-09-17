/* Linker script for the STM32F103C8T6 */
/* cortex-m-rt will use this to produce link.x */
MEMORY
{
  FLASH : ORIGIN = 0x08000000, LENGTH = 64K
  RAM : ORIGIN = 0x20000000, LENGTH = 20K
}

