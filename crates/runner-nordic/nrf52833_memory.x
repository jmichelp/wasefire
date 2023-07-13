/* nrf52833-dk */

__stack_size = 0x08000;
__store_size = 0x08000;

MEMORY
{
  FLASH : ORIGIN = 0x00000000, LENGTH = 512K - __store_size
  RAM   : ORIGIN = 0x20000000 + __stack_size, LENGTH = 128K - __stack_size
}

_stack_start = ORIGIN(RAM);
__eheap = ORIGIN(RAM) + LENGTH(RAM);
__sstore = ORIGIN(FLASH) + LENGTH(FLASH);
__estore = __sstore + __store_size;
