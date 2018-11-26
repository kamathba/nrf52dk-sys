SEARCH_DIR(.)

GROUP(-lgcc -lc -lnosys)

/* Linker script for the nRF52832. */
MEMORY
{
  /* Flash and RAM is offset to leave room for the S132 soft device. */
  FLASH (rx) : ORIGIN = 0x1f000, LENGTH = 0x61000
  RAM (rwx) :  ORIGIN = 0x20001fc0, LENGTH = 0xe040
}

SECTIONS
{
    .heap (COPY):
    {
        PROVIDE(end = .);
        KEEP(*(.heap*))
    } > RAM
} INSERT AFTER .bss;

SECTIONS
{
  .fs_data :
  {
    PROVIDE(__start_fs_data = .);
    KEEP(*(.fs_data))
    PROVIDE(__stop_fs_data = .);
  } > RAM
} INSERT AFTER .data;

SECTIONS
{
  .pwr_mgmt_data :
  {
    PROVIDE(__start_pwr_mgmt_data = .);
    KEEP(*(SORT(.pwr_mgmt_data*)))
    PROVIDE(__stop_pwr_mgmt_data = .);
  } > FLASH
} INSERT AFTER .text


INCLUDE "link.x"

__data_start__ = __sdata;
__data_end__ = __edata;
